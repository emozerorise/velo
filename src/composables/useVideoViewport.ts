import { onScopeDispose, watch, type Ref } from 'vue';
import { nextVideoViewportRevision, playerService } from '@/services/playerService';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import type { VideoViewportInput } from '@/types/workspace';

export function useVideoViewport(element: Ref<HTMLElement | null>, visible: () => boolean): void {
  const workspace = useWorkspaceStore();
  let disposed = false;
  let pending: number | null = null;
  let latestRevision = 0;
  let rect: VideoViewportInput['rect'] = { x: 0, y: 0, width: 1, height: 1 };
  let detachScale = () => {};

  function send(show: boolean) {
    const bounds = element.value?.getBoundingClientRect();
    if (bounds && bounds.width > 0 && bounds.height > 0) {
      rect = { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height };
    }
    const revision = nextVideoViewportRevision();
    latestRevision = revision;
    void playerService.setVideoViewport({
      rect, css_width: window.innerWidth, css_height: window.innerHeight,
      visible: show && !!element.value && !!bounds && bounds.width > 0 && bounds.height > 0,
      revision,
    }).then(() => {
      if (!disposed && revision === latestRevision) workspace.viewportError = false;
    }).catch(() => {
      if (!disposed && revision === latestRevision) workspace.viewportError = true;
    });
  }
  function schedule() {
    if (disposed || pending !== null) return;
    pending = requestAnimationFrame(() => {
      pending = null;
      if (!disposed) send(visible());
    });
  }
  function watchScale() {
    detachScale();
    const query = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
    const changed = () => { schedule(); watchScale(); };
    query.addEventListener('change', changed);
    detachScale = () => query.removeEventListener('change', changed);
  }
  const observer = new ResizeObserver(schedule);
  watch(element, (node) => {
    observer.disconnect();
    if (node) observer.observe(node);
    schedule();
  }, { immediate: true, flush: 'post' });
  watch(visible, schedule, { flush: 'post' });
  window.addEventListener('resize', schedule);
  window.addEventListener('scroll', schedule, true);
  window.visualViewport?.addEventListener('resize', schedule);
  window.visualViewport?.addEventListener('scroll', schedule);
  watchScale();

  onScopeDispose(() => {
    disposed = true;
    if (pending !== null) cancelAnimationFrame(pending);
    observer.disconnect();
    detachScale();
    window.removeEventListener('resize', schedule);
    window.removeEventListener('scroll', schedule, true);
    window.visualViewport?.removeEventListener('resize', schedule);
    window.visualViewport?.removeEventListener('scroll', schedule);
    send(false);
  });
}
