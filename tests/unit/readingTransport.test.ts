import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import ReadingTransport from '@/components/player/ReadingTransport.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { setLocale } from '@/composables/useI18n';
import { mountView } from './helpers/mountView';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
beforeEach(() => { setActivePinia(createPinia()); setLocale('en'); vi.clearAllMocks(); vi.stubGlobal('window', new EventTarget()); });
afterEach(() => vi.unstubAllGlobals());

it.each([false, true])('shows seek icons without numeric captions, retaining labelled five-second actions (%s)', (compact) => {
  const player = usePlayerStore();
  player.duration = 6252;
  const view = mountView(ReadingTransport, { compact }, getActivePinia()!);
  const seek = view.nodes().filter(n => n.tagName === 'BUTTON' && String(n.props['aria-label']).includes('5'));
  expect(seek).toHaveLength(2);
  expect(view.nodes().filter(n => n.tagName === 'SPAN' && n.text === '5')).toHaveLength(0);
  for (const button of seek) {
    expect(button.props.title).toBe(button.props['aria-label']);
    button.trigger('click');
  }
  expect(vi.mocked(invoke).mock.calls.filter(([name]) => name === 'player_seek')).toEqual([
    ['player_seek', { seconds: -5, exact: true }], ['player_seek', { seconds: 5, exact: true }],
  ]);
  expect(vi.mocked(invoke).mock.calls.some(([name]) => name === 'player_play' || name === 'player_toggle_pause')).toBe(false);
  view.unmount();
});
