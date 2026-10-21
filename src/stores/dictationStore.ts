import { computed, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { dictationService } from '@/services/dictationService';
import { newRequestId } from '@/services/chatService';
import { isVideoMasked } from '@/composables/useVideoMask';
import { useChatStore } from './chatStore';
import { usePlayerStore } from './playerStore';
import { useWorkspaceStore } from './workspaceStore';
import type { DictationErrorCode, DictationFailure, DictationIdentity, DictationPhase, DictationProgress, DictationReady } from '@/types/dictation';

const CODES = new Set<DictationErrorCode>([
  'permission_denied', 'permission_restricted', 'device_unavailable', 'unsupported_format',
  'engine_busy', 'missing_model', 'capture_failed', 'silence', 'transcription_failed', 'cancelled', 'invalid_input',
]);
function code(value: unknown): DictationErrorCode {
  return CODES.has(value as DictationErrorCode) ? value as DictationErrorCode : 'capture_failed';
}
type Session = { id: string; path: string; mediaPath: string };
const RANK = { requesting_permission: 0, recording: 1, transcribing: 2 } as const;

export const useDictationStore = defineStore('dictation', () => {
  const chat = useChatStore();
  const player = usePlayerStore();
  const workspace = useWorkspaceStore();
  const phase = ref<DictationPhase>('idle');
  const proposal = ref<string | null>(null);
  const elapsedMs = ref(0);
  const pausedPlayback = ref(false);
  const errorCode = ref<DictationErrorCode | null>(null);
  const listenersReady = ref(false);
  const active = computed(() => phase.value !== 'idle' && phase.value !== 'review');
  const canStart = computed(() => listenersReady.value && phase.value === 'idle' &&
    !!player.mediaInfo && !!chat.path && !chat.preparing && workspace.chatVisible && !isVideoMasked.value && !chat.isRunning);
  let session: Session | null = null;
  let generation = 0;
  let unsubscribers: (() => void)[] = [];

  watch(active, (busy) => { chat.dictationBusy = busy; }, { flush: 'sync' });
  function current(expected: Session | null): expected is Session {
    return !!expected && session === expected && expected.path === chat.path && expected.mediaPath === player.mediaInfo?.path;
  }
  function matches(payload: DictationIdentity): boolean {
    return current(session) && session.id === payload.session_id && session.path === payload.path;
  }
  function reset() {
    session = null; phase.value = 'idle'; proposal.value = null; elapsedMs.value = 0; pausedPlayback.value = false;
  }
  async function cancel() {
    const pending = active.value ? session : null;
    reset(); errorCode.value = null;
    if (pending) {
      try { await dictationService.cancel(pending.id); } catch { /* Ignore late IPC errors after local invalidation. */ }
    }
  }
  async function start() {
    if (!canStart.value) return;
    const next = { id: newRequestId(), path: chat.path!, mediaPath: player.mediaInfo!.path };
    session = next; errorCode.value = null; elapsedMs.value = 0; pausedPlayback.value = false;
    phase.value = 'requesting_permission';
    try { await dictationService.start(next.path, next.id); }
    catch (failure) {
      if (current(next) && active.value) { reset(); errorCode.value = code(failure); }
    }
  }
  async function stop() {
    const pending = session;
    if (!current(pending) || phase.value !== 'recording') return;
    phase.value = 'transcribing';
    try { await dictationService.stop(pending.id); }
    catch (failure) {
      if (current(pending) && active.value) { await cancel(); errorCode.value = code(failure); }
    }
  }
  function discard() { reset(); errorCode.value = null; }
  function insert(): boolean {
    if (phase.value !== 'review' || !current(session) || !proposal.value?.trim()) return false;
    if (!chat.appendDraft(proposal.value)) { errorCode.value = 'too_long'; return false; }
    discard(); return true;
  }
  async function progress(payload: DictationProgress) {
    if (!matches(payload) || !active.value || !(payload.phase in RANK)) return;
    const pending = session;
    // The OS consent dialog may briefly blur the window. Only accept actual
    // capture/transcription after re-reading focus, never a remembered grant.
    if (payload.phase !== 'requesting_permission') {
      let focused = false;
      try { focused = await getCurrentWindow().isFocused(); } catch { /* Fail closed. */ }
      if (!current(pending) || !active.value) return;
      if (!focused || !workspace.chatVisible || isVideoMasked.value) { await cancel(); return; }
    }
    const rank = RANK[phase.value as keyof typeof RANK];
    if (rank === undefined || RANK[payload.phase] < rank) return;
    phase.value = payload.phase;
    elapsedMs.value = Math.min(60000, Math.max(elapsedMs.value, payload.elapsed_ms));
    pausedPlayback.value = payload.paused_playback;
  }
  function ready(payload: DictationReady) {
    if (!matches(payload) || !active.value) return;
    proposal.value = payload.text; phase.value = 'review'; errorCode.value = null;
  }
  function failure(payload: DictationFailure) {
    if (!matches(payload) || !active.value) return;
    reset(); errorCode.value = payload.cancelled ? null : code(payload.code);
  }
  function cleanupListeners() {
    generation++; listenersReady.value = false;
    unsubscribers.forEach((fn) => fn()); unsubscribers = [];
    void cancel();
  }
  async function initListeners() {
    cleanupListeners();
    const revision = generation;
    const visibility = () => { if (document.visibilityState === 'hidden' && active.value) void cancel(); };
    document.addEventListener('visibilitychange', visibility);
    const subscribed = await Promise.allSettled([
      listen<DictationProgress>('velo://dictation-state', (e) => { void progress(e.payload); }),
      listen<DictationReady>('velo://dictation-ready', (e) => ready(e.payload)),
      listen<DictationFailure>('velo://dictation-error', (e) => failure(e.payload)),
      getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (!focused && active.value && phase.value !== 'requesting_permission') void cancel();
      }),
    ]);
    const handles = subscribed.flatMap((result) => result.status === 'fulfilled' ? [result.value] : []);
    handles.push(() => document.removeEventListener('visibilitychange', visibility));
    if (revision !== generation || subscribed.some((result) => result.status === 'rejected')) { handles.forEach((fn) => fn()); return; }
    unsubscribers = handles; listenersReady.value = true;
  }
  watch(() => [player.mediaInfo?.path, chat.path], () => { void cancel(); }, { flush: 'sync' });
  watch(() => workspace.chatVisible && !isVideoMasked.value, (visible) => {
    if (!visible && active.value) void cancel();
  }, { flush: 'sync' });
  return { phase, proposal, elapsedMs, pausedPlayback, errorCode, active, canStart, start, stop, cancel, insert, discard, initListeners, cleanupListeners };
});
