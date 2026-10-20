import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { listen, type Event } from '@tauri-apps/api/event';
import { chatService, newRequestId } from '@/services/chatService';
import { useChatStore } from '@/stores/chatStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useSummaryStore } from '@/stores/summaryStore';
import type { ChatReady, ChatStatus } from '@/types/chat';
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('@/services/chatService', () => ({
  chatService: { status: vi.fn(), send: vi.fn(), cancel: vi.fn() },
  newRequestId: vi.fn(),
}));
const sources = [{ start: 0, end: 30 }];
const coverage = {
  transcript: 'full',
  summary: 'absent',
  omitted_pairs: 0,
} as const;
const status = (path: string): ChatStatus => ({
  path,
  has_transcript: true,
  provider: 'ollama',
  base_url: 'http://localhost:11434',
  model: 'synthetic',
  configured: true,
  remote: false,
  has_key: false,
});
let listeners: Map<string, (event: Event<unknown>) => void>;
function receive(name: string, payload: unknown) {
  listeners.get(`velo://chat-${name}`)!({
    event: `velo://chat-${name}`,
    id: 0,
    payload,
  });
}
const lastSend = () => vi.mocked(chatService.send).mock.calls.at(-1)!;
function ready(text = 'Answer [00:05]') {
  const [path, id] = lastSend();
  receive('ready', {
    path,
    request_id: id,
    text,
    sources,
    coverage,
  } satisfies ChatReady);
}

describe('session-only file chat', () => {
  beforeEach(async () => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    listeners = new Map();
    let id = 0;
    vi.mocked(newRequestId).mockImplementation(() => `request-${++id}`);
    vi.mocked(listen).mockImplementation(async (name, handler) => {
      listeners.set(name, handler);
      return () => listeners.delete(name);
    });
    vi.mocked(chatService.status).mockImplementation(async (path) =>
      status(path),
    );
    vi.mocked(chatService.send).mockImplementation(async (path, id) => ({
      path,
      request_id: id,
      sources,
      coverage,
    }));
    vi.mocked(chatService.cancel).mockResolvedValue(undefined);
    await useChatStore().initListeners();
    await useChatStore().prepare('/a.wav');
  });
  afterEach(() => useChatStore().cleanupListeners());
  async function send() {
    const chat = useChatStore();
    chat.setDraft('Next?');
    await chat.send();
    return chat;
  }

  it('duplicate_delta_ignored', async () => {
    const chat = await send();
    const [path, id] = lastSend();
    const delta = { path, request_id: id, sequence: 1, text: 'คำตอบ' };
    receive('delta', delta);
    receive('delta', delta);
    expect(chat.messages.at(-1)?.text).toBe('คำตอบ');
    expect(chat.isRunning).toBe(true);
  });
  it('sequence_gap_waits_for_authoritative_final', async () => {
    const chat = await send();
    const [path, id] = lastSend();
    receive('delta', { path, request_id: id, sequence: 1, text: 'First' });
    receive('delta', {
      path,
      request_id: id,
      sequence: 3,
      text: 'Missing second',
    });
    expect(chat.messages.at(-1)?.text).toBe('First');
    ready('Complete authoritative answer');
    expect(chat.messages.at(-1)?.text).toBe('Complete authoritative answer');
    expect(chat.isRunning).toBe(false);
  });
  it('old_file_event_not_current_answer', async () => {
    const chat = await send();
    const [path, id] = lastSend();
    await chat.prepare('/b.wav');
    chat.setDraft('New file draft');
    receive('ready', {
      path,
      request_id: id,
      text: 'Old answer',
      sources,
      coverage,
    });
    expect(chat.messages).toEqual([]);
    expect(chat.draft).toBe('New file draft');
    expect(chatService.cancel).toHaveBeenCalledWith(id);
    await chat.prepare('/a.wav');
    expect(chat.messages.at(-1)?.status).toBe('cancelled');
  });
  it('retry_keeps_one_user_message', async () => {
    const chat = await send();
    const [path, id] = lastSend();
    receive('delta', { path, request_id: id, sequence: 1, text: 'Partial' });
    receive('error', {
      path,
      request_id: id,
      code: 'connection',
      message: 'never display this',
      cancelled: false,
    });
    expect(chat.messages.at(-1)?.status).toBe('failed');
    await chat.retry();
    expect(lastSend()[1]).not.toBe(id);
    ready();
    expect(chat.messages.filter((m) => m.role === 'user')).toHaveLength(1);
    expect(
      chat.messages.some((m) => m.text === 'Partial' && m.status === 'failed'),
    ).toBe(true);
  });
  it('provider_change_cancels_inflight', async () => {
    const chat = await send();
    const id = lastSend()[1];
    useSettingsStore().settings.summary.model = 'new model';
    await nextTick();
    expect(chatService.cancel).toHaveBeenCalledWith(id);
    expect(chat.isRunning).toBe(false);
  });
  it('unrelated_preferences_do_not_cancel_an_inflight_question', async () => {
    const chat = await send();
    const settings = useSettingsStore();
    settings.settings = {
      ...settings.settings,
      summary: { ...settings.settings.summary },
      general: { ...settings.settings.general, theme: 'light' },
    };
    await nextTick();
    expect(chat.isRunning).toBe(true);
    expect(chatService.cancel).not.toHaveBeenCalled();
  });
  it('credential_change_cancels_inflight_without_exposing_the_key', async () => {
    const chat = await send();
    const id = lastSend()[1];
    await useSummaryStore().setApiKey('synthetic-only');
    await nextTick();
    expect(chatService.cancel).toHaveBeenCalledWith(id);
    expect(chat.isRunning).toBe(false);
  });
  it('hide_reopen_keeps_thread', async () => {
    const chat = await send();
    const workspace = useWorkspaceStore();
    workspace.chatOpen = true;
    workspace.chatOpen = false;
    workspace.view = 'transcript';
    await nextTick();
    expect(chat.isRunning).toBe(true);
    expect(chatService.cancel).not.toHaveBeenCalled();
    ready();
    chat.setDraft('Retained draft');
    workspace.view = 'summary';
    workspace.chatOpen = true;
    await nextTick();
    expect(chat.messages).toHaveLength(2);
    expect(chat.draft).toBe('Retained draft');
  });
  it('eight_complete_pairs_report_two_client_omissions', async () => {
    const chat = useChatStore();
    for (let i = 0; i < 8; i++) {
      chat.setDraft(`Question ${i}`);
      await chat.send();
      ready(`Answer ${i}`);
    }
    chat.setDraft('Ninth question');
    await chat.send();
    expect(lastSend()[3]).toHaveLength(12);
    expect(lastSend()[3][0].content).toBe('Question 2');
    expect(chat.clientOmittedPairs).toBe(2);
    expect(chat.messages.filter((m) => m.role === 'user')).toHaveLength(9);
  });
  it('appendDraft preserves concurrent edits and rejects UTF-8 overflow without truncation', () => {
    const chat = useChatStore();
    chat.setDraft('แก้ไขใหม่');
    expect(chat.appendDraft('ข้อความจากไมค์')).toBe(true);
    expect(chat.draft).toBe('แก้ไขใหม่\nข้อความจากไมค์');
    const draft = chat.draft;
    expect(chat.appendDraft('ก'.repeat(3000))).toBe(false);
    expect(chat.draft).toBe(draft);
    chat.setDraft('ก'.repeat(3000));
    expect(chat.draftTooLong).toBe(true);
    expect(chat.canSend).toBe(false);
    expect(chatService.send).not.toHaveBeenCalled();
  });
  it('summary busy or missing transcript disables Send but retains editable text', async () => {
    const chat = useChatStore();
    chat.setDraft('Question');
    useSummaryStore().stage = 'reducing';
    await chat.send();
    expect(chatService.send).not.toHaveBeenCalled();
    expect(chat.draft).toBe('Question');
    useSummaryStore().stage = null;
    vi.mocked(chatService.status).mockResolvedValue({
      ...status('/a.wav'),
      has_transcript: false,
    });
    await chat.prepare('/a.wav');
    await chat.send();
    expect(chatService.send).not.toHaveBeenCalled();
  });
});
