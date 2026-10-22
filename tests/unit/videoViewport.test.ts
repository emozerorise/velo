import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick, ref } from 'vue';
import { mountComposable } from './helpers/mountComposable';
import { invoke } from '@tauri-apps/api/core';
import { useVideoViewport } from '@/composables/useVideoViewport';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import type { VideoViewportInput } from '@/types/workspace';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(1) }));

describe('native viewport observer', () => {
  let frame = 0;
  let frames: Map<number, FrameRequestCallback>;
  let notifyResize: ResizeObserverCallback;
  let disconnected: boolean;
  let screen: EventTarget & { innerWidth: number; innerHeight: number; devicePixelRatio: number; visualViewport: EventTarget; matchMedia: ReturnType<typeof vi.fn> };
  const flush = () => { const pending = [...frames.values()]; frames.clear(); pending.forEach((callback) => callback(0)); };
  const inputs = () => vi.mocked(invoke).mock.calls.map(([, args]) => (args as { input: VideoViewportInput }).input);
  function observe() {
    const element = ref({ getBoundingClientRect: () => ({ x: 100, y: 50, width: 400, height: 200 }) } as HTMLElement);
    const visible = ref(true);
    const scope = { stop: mountComposable(() => useVideoViewport(element, () => visible.value)) };
    return { scope, visible, element };
  }
  beforeEach(() => {
    setActivePinia(createPinia()); vi.clearAllMocks(); vi.mocked(invoke).mockResolvedValue(1);
    frames = new Map(); disconnected = false;
    screen = Object.assign(new EventTarget(), { innerWidth: 1000, innerHeight: 500, devicePixelRatio: 2, visualViewport: new EventTarget(), matchMedia: vi.fn(() => new EventTarget()) });
    vi.stubGlobal('window', screen);
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => { frames.set(++frame, callback); return frame; });
    vi.stubGlobal('cancelAnimationFrame', (id: number) => frames.delete(id));
    vi.stubGlobal('ResizeObserver', class {
      constructor(callback: ResizeObserverCallback) { notifyResize = callback; }
      observe() {} disconnect() { disconnected = true; }
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  it('coalesces_layout_and_scroll_changes_to_one_frame', async () => {
    const { scope } = observe(); await nextTick();
    notifyResize([], {} as ResizeObserver); screen.dispatchEvent(new Event('scroll')); screen.dispatchEvent(new Event('resize'));
    flush(); await nextTick();
    expect(vi.mocked(invoke).mock.calls).toHaveLength(1);
    expect(vi.mocked(invoke).mock.calls[0][0]).toBe('player_set_video_viewport');
    expect(inputs()[0]).toMatchObject({ rect: { x: 100, y: 50, width: 400, height: 200 }, css_width: 1000, css_height: 500, visible: true });
    scope.stop();
  });

  it('observer_disposal_cannot_reopen_surface', async () => {
    const { scope } = observe(); await nextTick();
    const delayed = [...frames.values()][0];
    scope.stop();
    expect(disconnected).toBe(true);
    expect(inputs().at(-1)?.visible).toBe(false);
    const count = inputs().length;
    delayed(0); screen.dispatchEvent(new Event('resize')); notifyResize([], {} as ResizeObserver); flush();
    expect(inputs()).toHaveLength(count);
  });

  it('uses_monotonic_revisions_across_remounts_and_never_changes_playback', async () => {
    const first = observe(); await nextTick(); flush(); first.scope.stop();
    const second = observe(); await nextTick(); flush();
    const revisions = inputs().map((input) => input.revision);
    expect(revisions.every((revision, index) => index === 0 || revision > revisions[index - 1])).toBe(true);
    second.visible.value = false; await nextTick(); flush();
    expect(inputs().at(-1)?.visible).toBe(false);
    expect(vi.mocked(invoke).mock.calls.every(([command]) => command === 'player_set_video_viewport')).toBe(true);
    second.scope.stop();
  });

  it('reports_ipc_failure_and_recovers_on_the_next_layout', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('IPC unavailable'));
    const { scope } = observe(); await nextTick(); flush(); await nextTick();
    await vi.waitFor(() => expect(useWorkspaceStore().viewportError).toBe(true));
    screen.dispatchEvent(new Event('resize')); flush(); await nextTick();
    await vi.waitFor(() => expect(useWorkspaceStore().viewportError).toBe(false));
    scope.stop();
  });
});
