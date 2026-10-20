import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useToast } from '@/composables/useToast';
import { mountComposable } from './helpers/mountComposable';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));

describe('keyboard shortcut ownership', () => {
  const unmounts: (() => void)[] = [];
  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal('window', new EventTarget());
    vi.stubGlobal('HTMLInputElement', class {});
    vi.stubGlobal('HTMLTextAreaElement', class {});
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });
  afterEach(() => {
    unmounts.splice(0).forEach((unmount) => unmount());
    vi.runAllTimers();
    useToast().toasts.value = [];
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
  const press = (code: string, shiftKey = false) => {
    const event = Object.assign(new Event('keydown', { cancelable: true }), {
      code,
      key: code,
      shiftKey,
    });
    window.dispatchEvent(event);
  };
  const mountOwnerAndButtons = () => {
    unmounts.push(
      mountComposable(() => {
        useKeyboardShortcuts({ register: true });
      }),
    );
    unmounts.push(
      mountComposable(() => {
        useKeyboardShortcuts();
      }),
    );
    unmounts.push(
      mountComposable(() => {
        useKeyboardShortcuts();
      }),
    );
  };

  it.each([
    ['ArrowRight', 5],
    ['ArrowLeft', -5],
  ] as const)(
    'dispatches one %s seek even when controls and playlist also use actions',
    (code, seconds) => {
      mountOwnerAndButtons();
      press(code);
      expect(invoke).toHaveBeenCalledTimes(1);
      expect(invoke).toHaveBeenCalledWith('player_seek', { seconds, exact: true });
      expect(useToast().toasts.value).toHaveLength(1);
    },
  );
  it('keeps repeated forward/backward feedback in one slot without swallowing real seeks', () => {
    mountOwnerAndButtons();
    press('ArrowRight');
    press('ArrowRight');
    press('ArrowLeft', true);
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ['player_seek', { seconds: 5, exact: true }],
      ['player_seek', { seconds: 5, exact: true }],
      ['player_seek', { seconds: -30, exact: true }],
    ]);
    expect(useToast().toasts.value).toHaveLength(1);
  });
  it('does not multiply listeners when auto-hidden controls remount', () => {
    mountOwnerAndButtons();
    unmounts.pop()!();
    unmounts.push(
      mountComposable(() => {
        useKeyboardShortcuts();
      }),
    );
    press('ArrowRight');
    expect(invoke).toHaveBeenCalledTimes(1);
  });
  it('removes the owner listener on unmount', () => {
    mountOwnerAndButtons();
    unmounts.shift()!();
    press('ArrowRight');
    expect(invoke).not.toHaveBeenCalled();
  });
});
