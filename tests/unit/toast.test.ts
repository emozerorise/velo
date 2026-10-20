import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useToast } from '@/composables/useToast';

describe('player feedback toasts', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.runAllTimers();
    useToast().toasts.value = [];
    vi.useRealTimers();
  });
  it('updates a seek toast in place and restarts its timeout', () => {
    const { showToast, toasts } = useToast();
    showToast('Forward 5', 1500, 'seek');
    const id = toasts.value[0].id;
    vi.advanceTimersByTime(1000);
    showToast('Backward 30', 1500, 'seek');
    expect(toasts.value).toMatchObject([{ id, text: 'Backward 30' }]);
    expect(toasts.value).toHaveLength(1);
    vi.advanceTimersByTime(500);
    expect(toasts.value).toHaveLength(1);
    vi.advanceTimersByTime(1000);
    expect(toasts.value).toHaveLength(0);
  });
  it('keeps unrelated notifications while refreshing playback feedback', () => {
    const { showToast, toasts } = useToast();
    showToast('Model removed', 3000);
    showToast('Forward 5', 1500, 'seek');
    showToast('Forward 5', 1500, 'seek');
    expect(toasts.value.map((t) => t.text)).toEqual(['Model removed', 'Forward 5']);
    vi.advanceTimersByTime(1500);
    expect(toasts.value.map((t) => t.text)).toEqual(['Model removed']);
  });
});
