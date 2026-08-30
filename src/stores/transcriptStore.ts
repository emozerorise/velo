import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { transcriptService } from '@/services/transcriptService';
import { usePlayerStore } from './playerStore';
import { usePlaylistStore } from './playlistStore';
import { useSettingsStore } from './settingsStore';
import type {
  Transcript,
  TranscriptEngineStatus,
  TranscriptError,
  TranscriptProgress,
  TranscriptStage,
} from '@/types/transcript';

/** Spoken languages worth offering before the full picker exists. This is the
 *  language of the audio, which has nothing to do with the app's own UI
 *  language -- the two are set in different places on purpose. */
export const TRANSCRIPT_LANGUAGES = [
  { code: 'auto', key: 'transcript.lang.auto' },
  { code: 'th', key: 'transcript.lang.th' },
  { code: 'en', key: 'transcript.lang.en' },
] as const;

export const useTranscriptStore = defineStore('transcript', () => {
  const player = usePlayerStore();
  const settings = useSettingsStore();

  const isPanelOpen = ref(false);
  const transcript = ref<Transcript | null>(null);
  const engine = ref<TranscriptEngineStatus | null>(null);
  // Language and vocabulary live in the persisted settings: the terms an
  // organisation says out loud barely change between meetings, so typing them
  // once should be enough.
  const language = computed({
    get: () => settings.settings.transcript.language,
    set: (value: string) => {
      settings.settings.transcript.language = value;
    },
  });

  const prompt = computed({
    get: () => settings.settings.transcript.prompt,
    set: (value: string) => {
      settings.settings.transcript.prompt = value;
    },
  });

  const isRunning = ref(false);
  const stage = ref<TranscriptStage | null>(null);
  const progress = ref(-1);
  const error = ref<string | null>(null);

  let unlistenFunctions: (() => void)[] = [];

  /// The segment covering the playhead. Segments are ordered and
  /// non-overlapping, so a binary search keeps this cheap at 10Hz.
  const activeIndex = computed(() => {
    const segments = transcript.value?.segments;
    if (!segments || segments.length === 0) return -1;

    const t = player.currentTime;
    let lo = 0;
    let hi = segments.length - 1;
    let found = -1;

    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      const seg = segments[mid];
      if (t < seg.start) {
        hi = mid - 1;
      } else {
        found = mid;
        lo = mid + 1;
      }
    }

    return found;
  });

  const hasTranscript = computed(() => (transcript.value?.segments.length ?? 0) > 0);

  async function initListeners() {
    cleanupListeners();

    const u1 = await listen<TranscriptProgress>('velo://transcript-progress', (event) => {
      if (event.payload.path !== currentPath()) return;
      isRunning.value = true;
      stage.value = event.payload.stage;
      progress.value = event.payload.progress;
    });

    const u2 = await listen<Transcript>('velo://transcript-ready', (event) => {
      if (event.payload.path === currentPath()) {
        transcript.value = event.payload;
      }
      resetJob();
    });

    const u3 = await listen<TranscriptError>('velo://transcript-error', (event) => {
      if (event.payload.path === currentPath()) {
        error.value = event.payload.message;
      }
      resetJob();
    });

    unlistenFunctions = [u1, u2, u3];

    try {
      engine.value = await transcriptService.engineStatus();
    } catch (e) {
      console.error('Failed to read transcript engine status:', e);
    }
  }

  function cleanupListeners() {
    unlistenFunctions.forEach((unlisten) => unlisten());
    unlistenFunctions = [];
  }

  function currentPath(): string | null {
    return player.mediaInfo?.path ?? null;
  }

  function resetJob() {
    isRunning.value = false;
    stage.value = null;
    progress.value = -1;
  }

  /// Called when the player swaps files: drop the old transcript and pick up
  /// a cached one for the new path if there is one.
  async function loadForCurrentMedia() {
    transcript.value = null;
    error.value = null;

    const path = currentPath();
    if (!path) return;

    try {
      transcript.value = await transcriptService.get(path);
    } catch (e) {
      console.error('Failed to load transcript:', e);
    }
  }

  async function generate() {
    const path = currentPath();
    if (!path || isRunning.value) return;

    error.value = null;
    isRunning.value = true;
    stage.value = 'extracting';
    progress.value = -1;
    settings.saveSettings(settings.settings);

    try {
      await transcriptService.generate(path, language.value, prompt.value);
    } catch (e) {
      error.value = String(e);
      resetJob();
    }
  }

  async function cancel() {
    try {
      await transcriptService.cancel();
    } catch (e) {
      console.error('Failed to cancel transcription:', e);
    }
  }

  /// Drops the cached transcript and returns the panel to its setup state
  /// rather than re-running immediately -- a second attempt is usually a
  /// second attempt *with different vocabulary*.
  async function discard() {
    const path = currentPath();
    if (!path) return;
    try {
      await transcriptService.remove(path);
    } catch (e) {
      console.error('Failed to clear transcript:', e);
    }
    transcript.value = null;
  }

  function jumpTo(seconds: number) {
    player.seekAbsolute(seconds);
  }

  // Both drawers occupy the right edge, so only one can be up at a time.
  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value;
    if (isPanelOpen.value) {
      usePlaylistStore().isDrawerOpen = false;
    }
  }

  return {
    isPanelOpen,
    transcript,
    engine,
    language,
    prompt,
    isRunning,
    stage,
    progress,
    error,
    activeIndex,
    hasTranscript,
    initListeners,
    cleanupListeners,
    loadForCurrentMedia,
    generate,
    cancel,
    discard,
    jumpTo,
    togglePanel,
  };
});
