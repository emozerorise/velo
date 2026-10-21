import { dictationHarness, flushDictation } from './helpers/dictationHarness';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: vi.fn() }));

describe('reviewed microphone drafts', () => {
  let f: Awaited<ReturnType<typeof dictationHarness>>;
  beforeEach(async () => { f = await dictationHarness(); });
  afterEach(() => f.cleanup());
  it('registers listeners without requesting consent or capture', () => {
    expect(f.dictation.phase).toBe('idle'); expect(f.starts()).toHaveLength(0);
    expect(f.events.has('velo://dictation-ready')).toBe(true);
  });
  it('mic_click_requests_once with camelCase IPC and snake_case event identity', async () => {
    await Promise.all([f.dictation.start(), f.dictation.start()]);
    expect(f.starts()).toHaveLength(1);
    const args = f.starts()[0][1] as Record<string, unknown>;
    expect(args).toEqual({ path: '/a.wav', sessionId: expect.stringMatching(/^[0-9a-f]{32}$/) });
    f.state('recording', 1000); await flushDictation();
    expect(f.dictation.phase).toBe('recording'); expect(f.dictation.elapsedMs).toBe(1000);
  });
  it('edited_draft_append_is_reviewed and never sends', async () => {
    f.chat.setDraft('เดิม'); await f.dictation.start(); f.ready();
    expect(f.chat.draft).toBe('เดิม'); expect(f.dictation.phase).toBe('review');
    f.chat.setDraft('แก้ไขใหม่'); expect(f.dictation.insert()).toBe(true);
    expect(f.chat.draft).toBe('แก้ไขใหม่\nข้อความจากไมค์'); expect(f.dictation.phase).toBe('idle');
    expect(invoke).not.toHaveBeenCalledWith('chat_send', expect.anything());
  });
  it('discard_keeps_current_draft', async () => {
    await f.dictation.start(); f.ready(); f.chat.setDraft('เขียนใหม่'); f.dictation.discard();
    expect(f.chat.draft).toBe('เขียนใหม่'); expect(f.dictation.proposal).toBeNull();
  });
  it('oversized insertion preserves both draft and editable proposal', async () => {
    await f.dictation.start(); f.ready('ก'.repeat(2000)); f.chat.setDraft('ข'.repeat(1000));
    expect(f.dictation.insert()).toBe(false); expect(f.chat.draft).toBe('ข'.repeat(1000)); expect(f.dictation.proposal).toBe('ก'.repeat(2000));
    expect(f.dictation.errorCode).toBe('too_long'); f.dictation.proposal = 'สั้น';
    expect(f.dictation.insert()).toBe(true); expect(f.chat.draft.endsWith('\nสั้น')).toBe(true);
  });
  it.each(['hide', 'transcript', 'file', 'compact-player'])('hide_or_switch_file_cancels_mic: %s', async (action) => {
    f.chat.setDraft('keep'); await f.dictation.start(); f.state('recording'); await flushDictation();
    const identity = f.identity();
    if (action === 'hide') f.workspace.chatOpen = false;
    if (action === 'transcript') f.workspace.toggleTranscript();
    if (action === 'file') f.player.mediaInfo = { ...f.player.mediaInfo!, path: '/b.wav' };
    if (action === 'compact-player') { f.workspace.compact = true; f.workspace.togglePlayer(); }
    await flushDictation();
    expect(invoke).toHaveBeenCalledWith('dictation_cancel', { sessionId: identity.session_id });
    expect(f.dictation.phase).toBe('idle'); expect(f.chat.draft).toBe('keep');
    f.event('velo://dictation-ready', { ...identity, text: 'late' }); expect(f.dictation.proposal).toBeNull();
  });
  it('permission_dialog_blur_is_not_hidden_recording', async () => {
    await f.dictation.start(); f.focus(false); await flushDictation();
    expect(f.dictation.phase).toBe('requesting_permission');
    expect(invoke).not.toHaveBeenCalledWith('dictation_cancel', expect.anything());
    f.state('recording'); await flushDictation();
    expect(f.dictation.phase).toBe('idle'); expect(invoke).toHaveBeenCalledWith('dictation_cancel', expect.anything());
    f.focus(true); expect(f.starts()).toHaveLength(1);
  });
  it('actual blur after permission cancels transcription without changing typed content', async () => {
    await f.dictation.start(); f.state('transcribing'); await flushDictation(); f.chat.setDraft('still editing');
    f.focus(false); await flushDictation(); expect(f.dictation.phase).toBe('idle'); expect(f.chat.draft).toBe('still editing');
  });
  it('late_result_ignored after cancellation and after a new session', async () => {
    await f.dictation.start(); const old = f.identity(); await f.dictation.cancel(); await f.dictation.start();
    f.event('velo://dictation-ready', { ...old, text: 'wrong generation' }); expect(f.dictation.proposal).toBeNull();
    f.ready('new generation'); expect(f.dictation.proposal).toBe('new generation');
  });
  it('whisper_missing_leaves_text_chat_enabled', async () => {
    vi.mocked(invoke).mockRejectedValueOnce('missing_model'); f.chat.setDraft('typed question');
    await f.dictation.start(); expect(f.dictation.errorCode).toBe('missing_model'); expect(f.chat.canSend).toBe(true);
    expect(f.chat.draft).toBe('typed question');
  });
  it('no media never starts capture', async () => {
    f.player.mediaInfo = null; await f.dictation.start(); expect(f.starts()).toHaveLength(0);
  });
  it('Stop and Cancel preserve IDs and do not send or resume media', async () => {
    await f.dictation.start(); f.state('recording'); await flushDictation(); const id = f.identity().session_id;
    await f.dictation.stop(); expect(invoke).toHaveBeenCalledWith('dictation_stop', { sessionId: id });
    await f.dictation.cancel(); expect(invoke).toHaveBeenCalledWith('dictation_cancel', { sessionId: id });
    expect(invoke).not.toHaveBeenCalledWith('chat_send', expect.anything()); expect(invoke).not.toHaveBeenCalledWith('player_play');
  });
});
