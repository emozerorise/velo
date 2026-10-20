import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { chatService } from '@/services/chatService';
import { useChatStore } from '@/stores/chatStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { setLocale } from '@/composables/useI18n';
import InlineChat from '@/components/chat/InlineChat.vue';
import ChatComposer from '@/components/chat/ChatComposer.vue';
import ChatMessage from '@/components/chat/ChatMessage.vue';
import SummaryPromptEditor from '@/components/transcript/SummaryPromptEditor.vue';
import { mountView, TestNode } from './helpers/mountView';
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock('@/services/chatService', () => ({
  chatService: {
    status: vi.fn(),
    send: vi.fn(),
    cancel: vi.fn().mockResolvedValue(undefined),
  },
  newRequestId: () => 'synthetic-request',
}));
describe('inline chat handlers', () => {
  beforeEach(async () => {
    setActivePinia(createPinia());
    setLocale('en');
    vi.clearAllMocks();
    vi.stubGlobal(
      'window',
      Object.assign(new EventTarget(), { innerWidth: 640, innerHeight: 400 }),
    );
    vi.stubGlobal('document', { activeElement: null });
    vi.stubGlobal('HTMLElement', TestNode);
    vi.stubGlobal('HTMLInputElement', class extends TestNode {});
    vi.stubGlobal('HTMLTextAreaElement', class extends TestNode {});
    vi.mocked(listen).mockResolvedValue(() => {});
    vi.mocked(chatService.status).mockResolvedValue({
      path: '/clip.wav',
      provider: 'ollama',
      model: 'synthetic',
      base_url: 'http://localhost:11434',
      has_transcript: true,
      remote: false,
      configured: true,
      has_key: false,
    });
    vi.mocked(chatService.send).mockResolvedValue({
      path: '/clip.wav',
      request_id: 'synthetic-request',
      sources: [],
      coverage: { transcript: 'full', summary: 'absent', omitted_pairs: 0 },
    });
    usePlayerStore().mediaInfo = {
      path: '/clip.wav',
      file_name: 'clip.wav',
      duration: 60,
      audio_codec: 'pcm',
      video_codec: null,
      width: 0,
      height: 0,
      fps: 0,
      hwdec_current: null,
    };
    await useChatStore().initListeners();
    await useChatStore().prepare('/clip.wav');
  });
  afterEach(() => {
    useChatStore().cleanupListeners();
    vi.unstubAllGlobals();
  });
  it.each(['en', 'th'])(
    'starter_cannot_silently_replace_draft (%s)',
    async (locale) => {
      setLocale(locale);
      const chat = useChatStore();
      chat.setDraft('My current draft');
      const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
      view.find('starter-actions').trigger('click');
      await nextTick();
      expect(chat.draft).toBe('My current draft');
      expect(chatService.send).not.toHaveBeenCalled();
      view.find('keep-draft').trigger('click');
      await nextTick();
      expect(chat.draft).toBe('My current draft');
      view.find('starter-prompt').trigger('click');
      await nextTick();
      view.find('replace-draft').trigger('click');
      await nextTick();
      expect(chat.draft).not.toBe('My current draft');
      expect(chatService.send).not.toHaveBeenCalled();
      view.unmount();
    },
  );
  it('IME and Shift+Enter never Send; ordinary Enter sends once', async () => {
    const chat = useChatStore();
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    const field = view.find('chat-draft');
    field.value = 'ถามเกี่ยวกับเพลง';
    field.trigger('input', { target: field });
    await nextTick();
    expect(chat.draft).toBe('ถามเกี่ยวกับเพลง');
    field.trigger('compositionstart');
    field.trigger(
      'keydown',
      Object.assign(new Event('keydown', { cancelable: true }), {
        key: 'Enter',
        shiftKey: false,
        isComposing: true,
      }),
    );
    field.trigger('compositionend');
    field.trigger(
      'keydown',
      Object.assign(new Event('keydown', { cancelable: true }), {
        key: 'Enter',
        shiftKey: true,
      }),
    );
    expect(chatService.send).not.toHaveBeenCalled();
    field.trigger(
      'keydown',
      Object.assign(new Event('keydown', { cancelable: true }), {
        key: 'Enter',
        shiftKey: false,
        isComposing: false,
      }),
    );
    await nextTick();
    expect(chatService.send).toHaveBeenCalledOnce();
    view.unmount();
  });
  it('disabled composer cannot Send by click or keyboard', () => {
    const send = vi.fn();
    const view = mountView(
      ChatComposer,
      {
        modelValue: 'Question',
        disabled: true,
        sending: false,
        remoteHost: null,
        onSend: send,
      },
      getActivePinia()!,
    );
    view.find('chat-send').trigger('click');
    view.find('chat-draft').trigger(
      'keydown',
      Object.assign(new Event('keydown', { cancelable: true }), {
        key: 'Enter',
      }),
    );
    expect(send).not.toHaveBeenCalled();
    view.unmount();
  });
  it('save_prompt_does_not_generate and initial draft does not change saved instructions before submit', async () => {
    const settings = useSettingsStore();
    settings.settings.summary.instructions = 'Saved old prompt';
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: 'Suggested draft' },
      getActivePinia()!,
    );
    const field = view.nodes().find((n) => n.tagName === 'TEXTAREA')!;
    expect(field.value).toBe('Suggested draft');
    expect(settings.settings.summary.instructions).toBe('Saved old prompt');
    const form = view.nodes().find((n) => n.tagName === 'FORM')!;
    form.trigger('submit');
    await nextTick();
    await Promise.resolve();
    await vi.waitFor(() =>
      expect(settings.settings.summary.instructions).toBe('Suggested draft'),
    );
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual([
      'settings_save',
    ]);
    view.unmount();
  });
  it('late_prompt_draft_does_not_overwrite_an_open_editor', async () => {
    const view = mountView(
      SummaryPromptEditor,
      { open: true, initialDraft: 'First suggestion' },
      getActivePinia()!,
    );
    const field = view.nodes().find((node) => node.tagName === 'TEXTAREA')!;
    field.value = 'User edits';
    field.trigger('input');
    await nextTick();
    view.updateProps({ initialDraft: 'Late suggestion' });
    await nextTick();
    expect(field.value).toBe('User edits');
    view.unmount();
  });
  it('prompt action proposes a draft without saving or generating', () => {
    const review = vi.fn();
    const view = mountView(
      ChatMessage,
      {
        message: {
          id: 'prompt',
          role: 'assistant',
          status: 'complete',
          text: '```prompt\nOnly agreed actions\n```',
        },
        onPrompt: review,
      },
      getActivePinia()!,
    );
    view.find('use-chat-prompt').trigger('click');
    expect(review).toHaveBeenCalledWith('Only agreed actions');
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
  it('stream updates do not move a reader away from older content', async () => {
    const scroll = Object.assign(new TestNode('DIV'), {
      scrollHeight: 1200,
      clientHeight: 300,
      scrollTop: 0,
    });
    const view = mountView(
      InlineChat,
      { visible: true, scrollElement: scroll },
      getActivePinia()!,
    );
    await nextTick();
    scroll.scrollTop = 100;
    scroll.dispatchEvent(new Event('scroll'));
    const field = view.find('chat-draft');
    field.value = 'Question';
    field.trigger('input', { target: field });
    await nextTick();
    view.find('chat-send').trigger('click');
    await nextTick();
    await nextTick();
    expect(scroll.scrollTop).toBe(100);
    expect(view.find('chat-latest')).toBeDefined();
    view.find('chat-latest').trigger('click');
    expect(scroll.scrollTop).toBe(1200);
    view.unmount();
  });
});
