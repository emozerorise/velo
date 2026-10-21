import { vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import { getCurrentWindow, type Window } from '@tauri-apps/api/window';
import { useChatStore } from '@/stores/chatStore';
import { useDictationStore } from '@/stores/dictationStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { TestNode } from './mountView';
import { setLocale } from '@/composables/useI18n';

export async function flushDictation() { await nextTick(); await nextTick(); await nextTick(); }
export async function dictationHarness() {
  vi.clearAllMocks(); setActivePinia(createPinia()); setLocale('en');
  const focusState = { focused: true, handler: null as null | ((event: { payload: boolean }) => void) };
  vi.mocked(getCurrentWindow).mockReturnValue({
    isFocused: async () => focusState.focused,
    onFocusChanged: async (handler: (event: { payload: boolean }) => void) => { focusState.handler = handler; return () => { focusState.handler = null; }; },
  } as unknown as Window);
  let nextId = 0;
  vi.stubGlobal('crypto', { getRandomValues: (bytes: Uint8Array) => bytes.fill(++nextId) });
  vi.stubGlobal('window', Object.assign(new EventTarget(), { innerWidth: 640, innerHeight: 400, devicePixelRatio: 2, matchMedia: () => new EventTarget() }));
  vi.stubGlobal('document', Object.assign(new EventTarget(), { activeElement: null, visibilityState: 'visible' }));
  vi.stubGlobal('HTMLElement', TestNode); vi.stubGlobal('HTMLInputElement', class extends TestNode {}); vi.stubGlobal('HTMLTextAreaElement', class extends TestNode {});
  vi.stubGlobal('requestAnimationFrame', () => 1); vi.stubGlobal('cancelAnimationFrame', () => {});
  const events = new Map<string, (event: TauriEvent<unknown>) => void>();
  vi.mocked(listen).mockImplementation(async (name, handler) => { events.set(name, handler); return () => events.delete(name); });
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const values = args as Record<string, unknown>;
    if (command === 'chat_status') return { path: values.path, provider: 'ollama', model: 'synthetic', base_url: 'http://127.0.0.1:49995', has_transcript: true, configured: true, remote: false, has_key: false };
    if (command === 'chat_send') return { path: values.path, request_id: values.requestId, sources: [], coverage: { transcript: 'full', summary: 'absent', omitted_pairs: 0 } };
    return undefined;
  });
  const player = usePlayerStore();
  player.mediaInfo = { path: '/a.wav', file_name: 'a.wav', duration: 60, audio_codec: 'pcm', video_codec: null, width: 0, height: 0, fps: 0, hwdec_current: null };
  const workspace = useWorkspaceStore(); workspace.chatOpen = true;
  const chat = useChatStore(); await chat.initListeners(); await chat.prepare('/a.wav');
  const dictation = useDictationStore(); await dictation.initListeners();
  const starts = () => vi.mocked(invoke).mock.calls.filter(([name]) => name === 'dictation_start');
  const identity = () => { const args = starts().at(-1)![1] as { path: string; sessionId: string }; return { path: args.path, session_id: args.sessionId }; };
  const event = (name: string, payload: unknown) => events.get(name)!({ event: name, id: 0, payload });
  return { player, workspace, chat, dictation, events, starts, identity, event,
    state: (phase: string, elapsed_ms = 0) => event('velo://dictation-state', { ...identity(), phase, elapsed_ms, paused_playback: phase !== 'requesting_permission' }),
    ready: (text = 'ข้อความจากไมค์') => event('velo://dictation-ready', { ...identity(), text }),
    focus: (focused: boolean) => { focusState.focused = focused; focusState.handler?.({ payload: focused }); },
    cleanup: () => { dictation.cleanupListeners(); chat.cleanupListeners(); vi.unstubAllGlobals(); },
  };
}
