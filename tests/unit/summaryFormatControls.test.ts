import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SummaryPromptEditor from '@/components/transcript/SummaryPromptEditor.vue';
import { useSettingsStore } from '@/stores/settingsStore';
import { setLocale } from '@/composables/useI18n';
import { mountView, TestNode } from './helpers/mountView';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('summary format editor controls', () => {
  it.each([false, true])(
    'focuses format selection on normal open, but the draft when reviewing a chat proposal (%s)',
    async (proposal) => {
      const view = mountView(
        SummaryPromptEditor,
        {
          open: true,
          ...(proposal ? { initialDraft: 'Proposed instructions' } : {}),
        },
        getActivePinia()!,
      );
      await nextTick();
      expect((document.activeElement as unknown as TestNode).tagName).toBe(
        proposal ? 'TEXTAREA' : 'SELECT',
      );
      view.unmount();
    },
  );
  beforeEach(() => {
    setActivePinia(createPinia());
    setLocale('en');
    vi.clearAllMocks();
    vi.stubGlobal('window', new EventTarget());
    vi.stubGlobal('document', { activeElement: null });
    vi.stubGlobal('HTMLElement', TestNode);
    vi.stubGlobal('Document', class {});
    vi.stubGlobal('ShadowRoot', class {});
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });
  const field = (view: ReturnType<typeof mountView>, tag: string) =>
    view.nodes().find((node) => node.tagName === tag)!;
  const choose = (view: ReturnType<typeof mountView>, value: string) =>
    field(view, 'SELECT').trigger('change', { target: { value } });

  it('keeps draft edits across format choices; Cancel and reopening restore only saved options', async () => {
    const settings = useSettingsStore();
    settings.settings.summary.instructions = 'Saved instructions';
    const close = vi.fn();
    let view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close },
      getActivePinia()!,
    );
    choose(view, 'custom');
    const textarea = field(view, 'TEXTAREA');
    textarea.value = 'My unsaved structure';
    textarea.trigger('input');
    await nextTick();
    choose(view, 'key_points');
    await nextTick();
    expect(textarea.value).toBe('My unsaved structure');
    expect(settings.settings.summary.format).toBe('meeting');
    expect(settings.settings.summary.instructions).toBe('Saved instructions');
    view
      .nodes()
      .find((node) => node.tagName === 'BUTTON' && node.text === 'Cancel')!
      .trigger('click');
    expect(close).toHaveBeenCalledOnce();
    view.unmount();
    view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close },
      getActivePinia()!,
    );
    await nextTick();
    expect(field(view, 'SELECT').props.value).toBe('meeting');
    expect(field(view, 'TEXTAREA').value).toBe('Saved instructions');
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });

  it('commits format and instructions only after disk success, without starting inference', async () => {
    let finish!: () => void;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = () => resolve(undefined);
        }),
    );
    const settings = useSettingsStore();
    const close = vi.fn();
    const view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close, initialDraft: 'One paragraph' },
      getActivePinia()!,
    );
    choose(view, 'custom');
    field(view, 'FORM').trigger('submit');
    await nextTick();
    expect(settings.settings.summary.format).toBe('meeting');
    expect(settings.settings.summary.instructions).toBe('');
    expect(field(view, 'SELECT').props.disabled).toBe(true);
    expect(close).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith('settings_save', {
      settings: expect.objectContaining({
        summary: expect.objectContaining({
          format: 'custom',
          instructions: 'One paragraph',
        }),
      }),
    });
    finish();
    await vi.waitFor(() => expect(close).toHaveBeenCalledOnce());
    expect(settings.settings.summary.format).toBe('custom');
    expect(settings.settings.summary.instructions).toBe('One paragraph');
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual([
      'settings_save',
    ]);
    view.unmount();
  });

  it('retains the saved format and prompt after a failed save, leaving edits available for retry', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Disk full'));
    const settings = useSettingsStore();
    const close = vi.fn();
    const view = mountView(
      SummaryPromptEditor,
      { open: true, onClose: close, initialDraft: 'New instructions' },
      getActivePinia()!,
    );
    choose(view, 'custom');
    field(view, 'FORM').trigger('submit');
    await vi.waitFor(() =>
      expect(view.nodes().some((node) => node.props.role === 'alert')).toBe(
        true,
      ),
    );
    expect(settings.settings.summary.format).toBe('meeting');
    expect(settings.settings.summary.instructions).toBe('');
    expect(field(view, 'SELECT').props.value).toBe('custom');
    expect(field(view, 'TEXTAREA').value).toBe('New instructions');
    expect(close).not.toHaveBeenCalled();
    view.unmount();
  });

  it('offers key-point examples that do not contradict its format, and free-form examples for custom', async () => {
    const view = mountView(
      SummaryPromptEditor,
      { open: true },
      getActivePinia()!,
    );
    choose(view, 'key_points');
    await nextTick();
    expect(
      view
        .nodes()
        .some(
          (node) =>
            node.tagName === 'BUTTON' && node.text === 'Concise key points',
        ),
    ).toBe(true);
    expect(
      view
        .nodes()
        .some(
          (node) => node.tagName === 'BUTTON' && node.text === 'One paragraph',
        ),
    ).toBe(false);
    choose(view, 'custom');
    await nextTick();
    view
      .nodes()
      .find(
        (node) => node.tagName === 'BUTTON' && node.text === 'One paragraph',
      )!
      .trigger('click');
    await nextTick();
    expect(field(view, 'TEXTAREA').value).toContain('without headings');
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
});
