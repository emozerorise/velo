import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { summaryService } from '@/services/summaryService';
import { usePlayerStore } from './playerStore';
import { useTranscriptStore } from './transcriptStore';
import type {
  ChainStage,
  Summary,
  SummaryDelta,
  SummaryFailure,
  SummaryProgress,
  SummaryStatus,
} from '@/types/summary';
import type { Transcript } from '@/types/transcript';

/** The language of the summary, which is not the app's UI language. */
export const SUMMARY_LANGUAGES = [
  { code: 'auto', key: 'summary.lang.auto' },
  { code: 'th', key: 'summary.lang.th' },
  { code: 'en', key: 'summary.lang.en' },
] as const;

/** Ordered, so the panel can say "step 3 of 4" without hardcoding it twice. */
const CHAIN_STEPS: ChainStage[] = ['checking', 'extracting', 'transcribing', 'mapping'];

function errorMessage(value: unknown): string {
  if (
    typeof value === 'object' &&
    value !== null &&
    'details' in value &&
    typeof value.details === 'string'
  ) {
    return value.details;
  }
  return String(value);
}

export const useSummaryStore = defineStore('summary', () => {
  const player = usePlayerStore();
  const transcripts = useTranscriptStore();

  const summary = ref<Summary | null>(null);
  const status = ref<SummaryStatus | null>(null);
  const models = ref<string[]>([]);

  const stage = ref<ChainStage | null>(null);
  /** True while a single click is carrying a run through transcription too. */
  const chained = ref(false);
  /** Set the moment the user cancels, so the resulting error stays silent. */
  const cancelling = ref(false);

  /** Final-pass text as it streams, shown before the summary is saved. */
  const streamed = ref('');
  const mapped = ref(0);
  const mapTotal = ref(0);
  const error = ref<string | null>(null);

  let unlistenFunctions: (() => void)[] = [];

  const isRunning = computed(() => stage.value !== null);
  const hasSummary = computed(() => (summary.value?.markdown.length ?? 0) > 0);
  const apiKeyRejected = computed(() => error.value?.includes('API key was rejected') ?? false);

  /** A summary made from a transcript that has since been redone. */
  const isStale = computed(() => {
    const current = transcripts.transcript?.segments.length;
    return (
      summary.value !== null && current !== undefined && summary.value.source_segments !== current
    );
  });

  /** 1-based position in the chained run; 0 when only summarising. */
  const step = computed(() => {
    if (!chained.value || stage.value === null) return 0;
    const at = CHAIN_STEPS.indexOf(stage.value === 'reducing' ? 'mapping' : stage.value);
    return at < 0 ? 0 : at + 1;
  });

  /** 0..1, or -1 where the running stage cannot report a fraction. */
  const progress = computed(() => {
    if (stage.value === 'extracting' || stage.value === 'transcribing') {
      return transcripts.progress;
    }
    if (stage.value === 'mapping' && mapTotal.value > 0) {
      return mapped.value / mapTotal.value;
    }
    return -1;
  });

  function currentPath(): string | null {
    return player.mediaInfo?.path ?? null;
  }

  function resetJob() {
    stage.value = null;
    chained.value = false;
    cancelling.value = false;
    mapped.value = 0;
    mapTotal.value = 0;
  }

  async function initListeners() {
    cleanupListeners();

    const u1 = await listen<SummaryProgress>('velo://summary-progress', (event) => {
      if (event.payload.path !== currentPath()) return;
      stage.value = event.payload.stage;
      mapped.value = event.payload.done;
      mapTotal.value = event.payload.total;
    });

    const u2 = await listen<SummaryDelta>('velo://summary-delta', (event) => {
      if (event.payload.path !== currentPath()) return;
      streamed.value += event.payload.text;
    });

    const u3 = await listen<Summary>('velo://summary-ready', (event) => {
      if (event.payload.path === currentPath()) {
        summary.value = event.payload;
        streamed.value = '';
      }
      resetJob();
    });

    const u4 = await listen<SummaryFailure>('velo://summary-error', (event) => {
      // A cancelled job reports as an error; the user already knows.
      if (event.payload.path === currentPath() && !cancelling.value) {
        error.value = event.payload.message;
      } else {
        // Nothing partial is worth keeping from a run the user stopped.
        streamed.value = '';
      }
      resetJob();
    });

    // The chain's own joint. The transcript is saved before this fires, so
    // the summary command can load it straight from disk.
    const u5 = await listen<Transcript>('velo://transcript-ready', (event) => {
      if (!chained.value || event.payload.path !== currentPath()) return;
      // The transcription stage is over. Clearing it hands the run to the
      // summariser, which otherwise sees a job already in flight and stops.
      stage.value = null;
      void summarise();
    });

    const u6 = await listen('velo://transcript-error', () => {
      // Transcription failed or was cancelled: never summarise a partial.
      if (chained.value) resetJob();
    });

    unlistenFunctions = [u1, u2, u3, u4, u5, u6];
    await refreshStatus();
  }

  function cleanupListeners() {
    unlistenFunctions.forEach((unlisten) => unlisten());
    unlistenFunctions = [];
  }

  // While chained, the transcript stages are this run's stages.
  watch(
    () => transcripts.stage,
    (value) => {
      if (chained.value && value) stage.value = value;
    },
  );

  async function refreshStatus() {
    try {
      status.value = await summaryService.status();
    } catch (e) {
      console.error('Failed to read summary status:', e);
    }
  }

  async function setApiKey(key: string): Promise<boolean> {
    try {
      await summaryService.setApiKey(key);
      await refreshStatus();
      error.value = null;
      return true;
    } catch (e) {
      error.value = errorMessage(e);
      return false;
    }
  }

  async function clearApiKey(): Promise<boolean> {
    try {
      await summaryService.clearApiKey();
      await refreshStatus();
      error.value = null;
      return true;
    } catch (e) {
      error.value = errorMessage(e);
      return false;
    }
  }

  /** Reachability check that doubles as the model picker's source. */
  async function probe(): Promise<boolean> {
    try {
      models.value = await summaryService.probe();
      return true;
    } catch (e) {
      error.value = errorMessage(e);
      return false;
    }
  }

  async function loadForCurrentMedia() {
    summary.value = null;
    streamed.value = '';
    error.value = null;

    const path = currentPath();
    if (!path) return;

    try {
      summary.value = await summaryService.get(path);
    } catch (e) {
      console.error('Failed to load summary:', e);
    }
  }

  async function summarise() {
    const path = currentPath();
    if (!path || isRunning.value) return;

    error.value = null;
    streamed.value = '';
    cancelling.value = false;
    mapped.value = 0;
    mapTotal.value = 0;
    stage.value = 'mapping';

    try {
      await summaryService.generate(path);
    } catch (e) {
      error.value = errorMessage(e);
      resetJob();
    }
  }

  /// Transcribe if needed, then summarise, from one click.
  ///
  /// The preflight is the point: a stopped Ollama is found in seconds,
  /// rather than after an hour of transcription that cannot be used.
  async function transcribeAndSummarise() {
    const path = currentPath();
    if (!path || isRunning.value || transcripts.isRunning) return;

    error.value = null;

    if (transcripts.hasTranscript) {
      await summarise();
      return;
    }

    stage.value = 'checking';
    if (!(await probe())) {
      stage.value = null;
      return;
    }

    chained.value = true;
    stage.value = 'extracting';
    await transcripts.generate();

    // `generate` swallows its own start-up failures, so confirm it took.
    if (!transcripts.isRunning) resetJob();
  }

  async function cancel() {
    cancelling.value = true;
    const running = stage.value;
    chained.value = false;

    try {
      if (running === 'extracting' || running === 'transcribing') {
        await transcripts.cancel();
        resetJob();
      } else if (running === 'mapping' || running === 'reducing') {
        await summaryService.cancel();
      } else {
        resetJob();
      }
    } catch (e) {
      console.error('Failed to cancel:', e);
      resetJob();
    }
  }

  function jumpTo(seconds: number) {
    player.seekAbsolute(seconds);
  }

  /// Drops the stored summary. The transcript is untouched, so writing a new
  /// summary costs one model call rather than another hour of transcription.
  async function discard() {
    const path = currentPath();
    if (!path) return;

    try {
      await summaryService.remove(path);
    } catch (e) {
      console.error('Failed to clear summary:', e);
    }
    summary.value = null;
    streamed.value = '';
  }

  return {
    summary,
    status,
    models,
    stage,
    chained,
    streamed,
    mapped,
    mapTotal,
    error,
    isRunning,
    hasSummary,
    apiKeyRejected,
    isStale,
    step,
    progress,
    initListeners,
    cleanupListeners,
    refreshStatus,
    setApiKey,
    clearApiKey,
    probe,
    loadForCurrentMedia,
    summarise,
    transcribeAndSummarise,
    cancel,
    discard,
    jumpTo,
  };
});
