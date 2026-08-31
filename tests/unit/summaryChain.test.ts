import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

/** Handlers registered through the mocked `listen`, so a test can fire the
 *  backend events the chain is built on. */
const handlers = new Map<string, (event: { payload: unknown }) => void>();
const invoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invoke(command, args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (name: string, handler: (event: { payload: unknown }) => void) => {
    handlers.set(name, handler);
    return Promise.resolve(() => handlers.delete(name));
  },
}));

import { useSummaryStore } from '@/stores/summaryStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import type { MediaInfo } from '@/types/player';
import type { Transcript } from '@/types/transcript';

const PATH = '/meetings/standup.mp4';

function emit(name: string, payload: unknown) {
  handlers.get(name)?.({ payload });
}

/** Lets the promise chains inside the store's event handlers settle. */
function settle() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function called(command: string) {
  return invoke.mock.calls.some(([name]) => name === command);
}

function transcript(segments = 3): Transcript {
  return {
    path: PATH,
    language: 'th',
    model: 'ggml-large-v3-turbo-q5_0.bin',
    prompt: '',
    created_at: 0,
    segments: Array.from({ length: segments }, (_, i) => ({
      start: i,
      end: i + 1,
      text: 'พูดอะไรสักอย่าง',
    })),
  };
}

describe('the transcribe-and-summarise chain', () => {
  beforeEach(async () => {
    // The chain runs through the settings store, which touches the document
    // when it persists. These tests run headless, so give it just enough.
    globalThis.document = {
      documentElement: { lang: '', classList: { add() {}, remove() {}, toggle() {} } },
    } as unknown as Document;

    setActivePinia(createPinia());
    handlers.clear();
    invoke.mockReset();
    invoke.mockImplementation(async (command: string) => {
      switch (command) {
        case 'summary_status':
          return {
            provider: 'ollama',
            base_url: 'http://localhost:11434',
            model: 'qwen3:8b',
            configured: true,
            remote: false,
          };
        case 'summary_probe':
          return ['qwen3:8b'];
        default:
          return null;
      }
    });

    usePlayerStore().mediaInfo = { path: PATH } as unknown as MediaInfo;
    await useSummaryStore().initListeners();
    invoke.mockClear();
  });

  it('stops before transcribing when the model server is unreachable', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'summary_probe') throw 'Could not reach the model server';
      return null;
    });

    const store = useSummaryStore();
    await store.transcribeAndSummarise();

    expect(called('transcript_generate')).toBe(false);
    expect(store.isRunning).toBe(false);
    expect(store.error).toContain('Could not reach');
  });

  it('summarises once the transcript it started lands', async () => {
    const store = useSummaryStore();
    await store.transcribeAndSummarise();

    expect(called('transcript_generate')).toBe(true);
    expect(called('summary_generate')).toBe(false);
    expect(store.stage).toBe('extracting');

    emit('velo://transcript-ready', transcript());
    await settle();

    expect(called('summary_generate')).toBe(true);
  });

  it('does not summarise a transcript that was cancelled part way', async () => {
    const store = useSummaryStore();
    const transcripts = useTranscriptStore();

    await store.transcribeAndSummarise();
    transcripts.stage = 'transcribing';
    await settle();
    expect(store.stage).toBe('transcribing');

    await store.cancel();
    expect(called('transcript_cancel')).toBe(true);

    // Whatever arrives afterwards must not restart the chain.
    emit('velo://transcript-ready', transcript());
    await settle();

    expect(called('summary_generate')).toBe(false);
    expect(store.isRunning).toBe(false);
  });

  it('retries the summary alone, never the transcription', async () => {
    const store = useSummaryStore();
    useTranscriptStore().transcript = transcript();

    await store.transcribeAndSummarise();

    expect(called('summary_generate')).toBe(true);
    expect(called('transcript_generate')).toBe(false);
    expect(store.stage).not.toBeNull();
  });

  it('keeps quiet about the error a cancel causes', async () => {
    const store = useSummaryStore();
    useTranscriptStore().transcript = transcript();

    await store.transcribeAndSummarise();
    emit('velo://summary-progress', { path: PATH, stage: 'reducing', done: 0, total: 1 });
    await store.cancel();

    emit('velo://summary-error', { path: PATH, message: 'Cancelled' });

    expect(store.error).toBeNull();
    expect(store.isRunning).toBe(false);
  });

  it('reports a failure that was not asked for', async () => {
    const store = useSummaryStore();
    useTranscriptStore().transcript = transcript();

    await store.transcribeAndSummarise();
    emit('velo://summary-error', { path: PATH, message: 'The provider failed to answer' });

    expect(store.error).toBe('The provider failed to answer');
    expect(store.isRunning).toBe(false);
  });

  it('streams the final pass and keeps the saved summary', async () => {
    const store = useSummaryStore();
    useTranscriptStore().transcript = transcript();

    await store.transcribeAndSummarise();
    emit('velo://summary-delta', { path: PATH, text: '## ภาพรวม\n' });
    emit('velo://summary-delta', { path: PATH, text: 'ทีมตกลงเลื่อน deploy' });
    expect(store.streamed).toContain('ทีมตกลงเลื่อน');

    emit('velo://summary-ready', {
      path: PATH,
      model: 'qwen3:8b',
      language: 'th',
      markdown: '## ภาพรวม\nทีมตกลงเลื่อน deploy [00:12]',
      created_at: 1,
      source_segments: 3,
    });

    expect(store.hasSummary).toBe(true);
    expect(store.streamed).toBe('');
    expect(store.isRunning).toBe(false);
    expect(store.isStale).toBe(false);
  });

  it('marks a summary stale once the transcript is redone', async () => {
    const store = useSummaryStore();
    const transcripts = useTranscriptStore();
    transcripts.transcript = transcript(3);

    emit('velo://summary-ready', {
      path: PATH,
      model: 'qwen3:8b',
      language: 'th',
      markdown: 'สรุป',
      created_at: 1,
      source_segments: 3,
    });
    expect(store.isStale).toBe(false);

    transcripts.transcript = transcript(9);
    expect(store.isStale).toBe(true);
  });

  it('numbers the steps of a chained run', async () => {
    const store = useSummaryStore();
    const transcripts = useTranscriptStore();

    await store.transcribeAndSummarise();
    expect(store.step).toBe(2);

    transcripts.stage = 'transcribing';
    await settle();
    expect(store.step).toBe(3);

    emit('velo://summary-progress', { path: PATH, stage: 'mapping', done: 0, total: 4 });
    expect(store.step).toBe(4);
    expect(store.progress).toBe(0);
  });
});
