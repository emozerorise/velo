import { dictationHarness, flushDictation } from './helpers/dictationHarness';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getActivePinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { setLocale } from '@/composables/useI18n';
import InlineChat from '@/components/chat/InlineChat.vue';
import { mountView } from './helpers/mountView';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: vi.fn() }));

describe('inline microphone controls', () => {
  let f: Awaited<ReturnType<typeof dictationHarness>>;
  beforeEach(async () => { f = await dictationHarness(); }); afterEach(() => f.cleanup());
  it.each(['en', 'th'])('mic_click_requests_once and second_click_stops_not_sends (%s)', async (locale) => {
    setLocale(locale); const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    view.find('dictation-toggle').trigger('click'); await flushDictation(); expect(f.starts()).toHaveLength(1);
    f.state('recording', 2000); await flushDictation();
    expect(view.find('dictation-toggle').props['aria-label']).toBe(locale === 'th' ? 'หยุดบันทึกคำถาม' : 'Stop recording question');
    view.find('dictation-toggle').trigger('click'); await flushDictation();
    expect(invoke).toHaveBeenCalledWith('dictation_stop', { sessionId: f.identity().session_id });
    expect(invoke).not.toHaveBeenCalledWith('chat_send', expect.anything()); view.unmount();
  });
  it('keyboard review handlers append current edits and return focus to composer', async () => {
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    view.find('dictation-toggle').trigger('click'); await flushDictation(); f.ready(); await flushDictation();
    const field = view.find('chat-draft'); field.value = 'แก้ไขใหม่'; field.trigger('input', { target: field });
    view.find('dictation-insert').trigger('click'); await flushDictation();
    expect(f.chat.draft).toBe('แก้ไขใหม่\nข้อความจากไมค์'); expect(field.focusCount).toBeGreaterThan(0);
    expect(invoke).not.toHaveBeenCalledWith('chat_send', expect.anything()); view.unmount();
  });
  it('ime_enter_during_recording_never_sends while ordinary editing remains enabled', async () => {
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!);
    view.find('dictation-toggle').trigger('click'); await flushDictation(); f.state('recording'); await flushDictation();
    const field = view.find('chat-draft'); field.value = 'typed while recording'; field.trigger('input', { target: field }); await flushDictation();
    field.trigger('compositionstart'); field.trigger('keydown', Object.assign(new Event('keydown', { cancelable: true }), { key: 'Enter', isComposing: true })); field.trigger('compositionend');
    field.trigger('keydown', Object.assign(new Event('keydown', { cancelable: true }), { key: 'Enter', isComposing: false }));
    expect(view.find('chat-send').props.disabled).toBe(true); expect(field.props.disabled).not.toBe(true);
    expect(invoke).not.toHaveBeenCalledWith('chat_send', expect.anything()); view.unmount();
  });
  it('Discard and Cancel keep drafts and focus the composer', async () => {
    const view = mountView(InlineChat, { visible: true }, getActivePinia()!); f.chat.setDraft('keep this');
    view.find('dictation-toggle').trigger('click'); await flushDictation(); f.ready(); await flushDictation();
    view.find('dictation-discard').trigger('click'); await flushDictation(); expect(f.chat.draft).toBe('keep this');
    view.find('dictation-toggle').trigger('click'); await flushDictation(); view.find('dictation-cancel').trigger('click'); await flushDictation();
    expect(f.chat.draft).toBe('keep this'); expect(view.find('chat-draft').focusCount).toBeGreaterThan(0); view.unmount();
  });
});
