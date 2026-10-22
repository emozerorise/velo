import { nextTick, onScopeDispose, watch, type Ref } from 'vue';

export function useReadingScroll(viewport: Ref<HTMLElement | null>, reading: () => boolean): void {
  const positions = new Map<boolean, number>();
  let detach = () => {};
  watch(
    viewport,
    (element) => {
      detach();
      if (!element) return;
      const remember = () => positions.set(reading(), element.scrollTop);
      element.addEventListener('scroll', remember, { passive: true });
      detach = () => element.removeEventListener('scroll', remember);
    },
    { immediate: true, flush: 'post' },
  );
  onScopeDispose(() => detach());
  watch(reading, async (current) => {
    const element = viewport.value;
    if (!element) return;
    // A parent width change can clamp scrollTop before this watcher runs.
    // Remember real scroll events instead of sampling the already-new layout.
    await nextTick();
    const saved = positions.get(current);
    if (viewport.value === element && saved !== undefined) element.scrollTop = saved;
  });
}
