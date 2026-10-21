import { computed, reactive, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import { chatService, newRequestId } from '@/services/chatService';
import { useSettingsStore } from './settingsStore';
import { useSummaryStore } from './summaryStore';
import { utf8Length } from '@/utils/chatContent';
import type {
  ChatCoverage,
  ChatDelta,
  ChatError,
  ChatErrorCode,
  ChatHistoryMessage,
  ChatMessage,
  ChatReady,
  ChatSource,
  ChatStatus,
} from '@/types/chat';

type Thread = {
  messages: ChatMessage[];
  draft: string;
  error: ChatErrorCode | null;
  lastAttempt: Attempt | null;
};
type Attempt = {
  userId: string;
  text: string;
  history: ChatHistoryMessage[];
  omitted: number;
};
type Active = {
  id: string;
  path: string;
  assistantId: string;
  attempt: Attempt;
  sequence: number;
  gap: boolean;
};
const CODES = new Set<ChatErrorCode>([
  'not_configured',
  'missing_transcript',
  'auth',
  'connection',
  'rate_limit',
  'provider',
  'context',
  'timeout',
  'empty',
  'incomplete',
  'busy',
  'cancelled',
  'invalid_input',
]);
function errorCode(error: unknown): ChatErrorCode {
  if (
    error &&
    typeof error === 'object' &&
    'code' in error &&
    CODES.has(error.code as ChatErrorCode)
  )
    return error.code as ChatErrorCode;
  return 'provider';
}
function lastMatch<T>(
  items: T[],
  predicate: (item: T) => boolean,
): T | undefined {
  for (let i = items.length - 1; i >= 0; i--)
    if (predicate(items[i])) return items[i];
}

export const useChatStore = defineStore('chat', () => {
  const settings = useSettingsStore();
  const summaries = useSummaryStore();
  const threads = reactive(new Map<string, Thread>());
  const aliases = new Map<string, string>();
  const path = ref<string | null>(null);
  const status = ref<ChatStatus | null>(null);
  const preparing = ref(false);
  const active = ref<Active | null>(null);
  const listenersReady = ref(false);
  const dictationBusy = ref(false);
  let prepareGeneration = 0;
  let listenerGeneration = 0;
  let requestedPath: string | null = null;
  let unsubscribers: (() => void)[] = [];
  function getThread(key: string): Thread {
    if (!threads.has(key))
      threads.set(key, {
        messages: [],
        draft: '',
        error: null,
        lastAttempt: null,
      });
    return threads.get(key)!;
  }
  const thread = computed(() => (path.value ? getThread(path.value) : null));
  const messages = computed(() => thread.value?.messages ?? []);
  const draft = computed(() => thread.value?.draft ?? '');
  const isRunning = computed(() => active.value !== null);
  const error = computed(() => thread.value?.error ?? null);
  const draftTooLong = computed(() => utf8Length(draft.value) > 8192);
  const latestAnswer = computed(() =>
    lastMatch(messages.value, (m) => m.role === 'assistant'),
  );
  const coverage = computed<ChatCoverage | null>(
    () => latestAnswer.value?.coverage ?? null,
  );
  const sources = computed<ChatSource[]>(
    () => latestAnswer.value?.sources ?? [],
  );
  const clientOmittedPairs = computed(
    () => latestAnswer.value?.clientOmittedPairs ?? 0,
  );
  const canSend = computed(
    () =>
      !!path.value &&
      listenersReady.value &&
      !preparing.value &&
      !!status.value?.has_transcript &&
      status.value.configured &&
      (status.value.provider !== 'deepseek' || status.value.has_key) &&
      !summaries.isRunning &&
      !isRunning.value &&
      !dictationBusy.value &&
      !!draft.value.trim() &&
      !draftTooLong.value,
  );

  function setDraft(text: string) {
    if (thread.value) thread.value.draft = text;
  }
  function appendDraft(text: string): boolean {
    if (!thread.value) return false;
    const combined = [draft.value, text].filter(Boolean).join('\n');
    if (utf8Length(combined) > 8192) return false;
    thread.value.draft = combined;
    return true;
  }
  function assistant(turn: Active) {
    return getThread(turn.path).messages.find(
      (m) => m.id === turn.assistantId,
    )!;
  }
  function clearSentDraft(turn: Active) {
    const t = getThread(turn.path);
    if (t.draft === turn.attempt.text) t.draft = '';
  }
  function fail(turn: Active, code: ChatErrorCode) {
    const t = getThread(turn.path);
    assistant(turn).status = code === 'cancelled' ? 'cancelled' : 'failed';
    t.error = code;
    if (!t.draft) t.draft = turn.attempt.text;
    if (active.value?.id === turn.id) active.value = null;
  }
  async function cancel() {
    const turn = active.value;
    if (!turn) return;
    fail(turn, 'cancelled');
    try {
      await chatService.cancel(turn.id);
    } catch {
      /* Late events are already isolated locally. */
    }
  }
  async function prepare(mediaPath: string) {
    const generation = ++prepareGeneration;
    if (requestedPath !== mediaPath) void cancel();
    requestedPath = mediaPath || null;
    preparing.value = !!mediaPath;
    if (!mediaPath) {
      path.value = null;
      status.value = null;
      return;
    }
    const known =
      aliases.get(mediaPath) ?? (threads.has(mediaPath) ? mediaPath : null);
    path.value = known;
    status.value = null;
    try {
      const next = await chatService.status(mediaPath);
      if (generation !== prepareGeneration) return;
      aliases.set(mediaPath, next.path);
      const provisional = threads.get(mediaPath);
      const canonical = getThread(next.path);
      if (next.path !== mediaPath && provisional) {
        // An error-state draft may have been typed before canonical identity
        // was available. Preserve both drafts rather than overwriting either.
        if (provisional.draft)
          canonical.draft = [canonical.draft, provisional.draft]
            .filter(Boolean)
            .join('\n');
        threads.delete(mediaPath);
      }
      path.value = next.path;
      status.value = next;
    } catch (failure) {
      if (generation !== prepareGeneration) return;
      path.value = known ?? mediaPath;
      getThread(path.value).error = errorCode(failure);
    } finally {
      if (generation === prepareGeneration) preparing.value = false;
    }
  }
  function completedPairs(t: Thread): ChatHistoryMessage[][] {
    return t.messages
      .filter((m) => m.role === 'user' && m.status === 'complete')
      .flatMap((user) => {
        const reply = lastMatch(
          t.messages,
          (m) =>
            m.role === 'assistant' &&
            m.replyTo === user.id &&
            m.status === 'complete',
        );
        return reply
          ? [
              [
                { role: 'user', content: user.text },
                { role: 'assistant', content: reply.text },
              ] as ChatHistoryMessage[],
            ]
          : [];
      });
  }
  async function start(attempt: Attempt) {
    if (!path.value || isRunning.value) return;
    const id = newRequestId();
    const turn: Active = {
      id,
      path: path.value,
      assistantId: `assistant-${id}`,
      attempt,
      sequence: 0,
      gap: false,
    };
    const t = getThread(turn.path);
    t.error = null;
    t.lastAttempt = attempt;
    t.messages.push({
      id: turn.assistantId,
      role: 'assistant',
      text: '',
      status: 'streaming',
      replyTo: attempt.userId,
      clientOmittedPairs: attempt.omitted,
    });
    active.value = turn;
    try {
      const started = await chatService.send(
        turn.path,
        id,
        attempt.text,
        attempt.history,
      );
      if (
        active.value?.id !== id ||
        started.path !== turn.path ||
        started.request_id !== id
      )
        return;
      assistant(turn).sources = started.sources;
      assistant(turn).coverage = started.coverage;
      clearSentDraft(turn);
    } catch (failure) {
      if (active.value?.id === id) fail(turn, errorCode(failure));
    }
  }
  async function send() {
    if (!canSend.value || !thread.value) return;
    const t = thread.value;
    const pairs = completedPairs(t);
    const text = draft.value;
    const userId = `user-${newRequestId()}`;
    t.messages.push({ id: userId, role: 'user', text, status: 'complete' });
    await start({
      userId,
      text,
      history: pairs.slice(-6).flat(),
      omitted: Math.max(0, pairs.length - 6),
    });
  }
  async function retry() {
    const attempt = thread.value?.lastAttempt;
    if (
      !attempt ||
      isRunning.value ||
      dictationBusy.value ||
      summaries.isRunning ||
      !status.value?.configured ||
      !status.value.has_transcript ||
      (status.value.provider === 'deepseek' && !status.value.has_key) ||
      preparing.value ||
      !listenersReady.value
    )
      return;
    await start(attempt);
  }
  function matches(payload: {
    path: string;
    request_id: string;
  }): Active | null {
    const turn = active.value;
    return turn && payload.path === turn.path && payload.request_id === turn.id
      ? turn
      : null;
  }
  function delta(payload: ChatDelta) {
    const turn = matches(payload);
    if (
      !turn ||
      !Number.isSafeInteger(payload.sequence) ||
      payload.sequence <= turn.sequence ||
      turn.gap
    )
      return;
    if (payload.sequence !== turn.sequence + 1) {
      turn.gap = true;
      return;
    }
    turn.sequence = payload.sequence;
    assistant(turn).text += payload.text;
  }
  function ready(payload: ChatReady) {
    const turn = matches(payload);
    if (!turn) return;
    Object.assign(assistant(turn), {
      text: payload.text,
      status: 'complete',
      sources: payload.sources,
      coverage: payload.coverage,
    });
    clearSentDraft(turn);
    getThread(turn.path).error = null;
    active.value = null;
  }
  function failure(payload: ChatError) {
    const turn = matches(payload);
    if (turn) fail(turn, CODES.has(payload.code) ? payload.code : 'provider');
  }
  function cleanupListeners() {
    listenerGeneration++;
    listenersReady.value = false;
    unsubscribers.forEach((fn) => fn());
    unsubscribers = [];
  }
  async function initListeners() {
    cleanupListeners();
    const generation = listenerGeneration;
    const subscribed = await Promise.allSettled([
      listen<ChatDelta>('velo://chat-delta', (event) => delta(event.payload)),
      listen<ChatReady>('velo://chat-ready', (event) => ready(event.payload)),
      listen<ChatError>('velo://chat-error', (event) => failure(event.payload)),
    ]);
    const readySubscriptions = subscribed.flatMap((result) =>
      result.status === 'fulfilled' ? [result.value] : [],
    );
    if (
      generation !== listenerGeneration ||
      subscribed.some((result) => result.status === 'rejected')
    ) {
      readySubscriptions.forEach((fn) => fn());
      return;
    }
    unsubscribers = readySubscriptions;
    listenersReady.value = true;
  }
  watch(
    () =>
      JSON.stringify([
        settings.settings.summary.provider,
        settings.settings.summary.base_url,
        settings.settings.summary.model,
        settings.settings.summary.context_tokens,
        settings.settings.summary.language,
        summaries.credentialRevision,
      ]),
    () => {
      void cancel();
      if (requestedPath) void prepare(requestedPath);
    },
  );
  return {
    path,
    status,
    preparing,
    messages,
    draft,
    isRunning,
    errorCode: error,
    draftTooLong,
    canSend,
    dictationBusy,
    coverage,
    sources,
    clientOmittedPairs,
    setDraft,
    appendDraft,
    prepare,
    send,
    cancel,
    retry,
    initListeners,
    cleanupListeners,
  };
});
