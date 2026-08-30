import { onMounted, onUnmounted, watch } from 'vue';
import { usePlayerStore } from '@/stores/playerStore';
import { useSettingsStore } from '@/stores/settingsStore';
import type { HistoryItem } from '@/services/settingsService';

/** Below this there is nothing worth coming back to. */
const MIN_RESUME_SECONDS = 15;
/** Resuming into the closing seconds is worse than starting over. */
const END_MARGIN_SECONDS = 30;
/** How often a position is written while playback runs. */
const WRITE_INTERVAL_MS = 5000;

/**
 * The position to store for a file, given where playback stopped.
 * Zero means "start from the beginning next time".
 */
export function resumePositionFor(position: number, duration: number): number {
  if (duration <= 0 || position < MIN_RESUME_SECONDS) return 0;
  if (position >= duration - END_MARGIN_SECONDS) return 0;
  return position;
}

/**
 * Remembers where each file was left off, so reopening it picks up there.
 *
 * The player emits a time update ten times a second; writing that often would
 * hammer the disk, so positions are throttled and additionally flushed at the
 * moments that matter -- pausing, stopping, switching files, closing the app.
 */
export function usePlaybackHistory() {
  const player = usePlayerStore();
  const settings = useSettingsStore();

  /**
   * The most recent snapshot of the file currently playing. Held so that a
   * file switch can still store the *outgoing* file's position: by the time
   * the new media info arrives, `player.currentTime` already belongs to it.
   */
  let pending: HistoryItem | null = null;
  let lastWrite = 0;

  function snapshot(): HistoryItem | null {
    const media = player.mediaInfo;
    if (!media || player.duration <= 0) return null;

    return {
      path: media.path,
      file_name: media.file_name,
      last_position: resumePositionFor(player.currentTime, player.duration),
      duration: player.duration,
      last_played_timestamp: Math.floor(Date.now() / 1000),
    };
  }

  function flush(item: HistoryItem | null) {
    if (!item) return;
    lastWrite = Date.now();
    settings.recordPlayback(item);
  }

  watch(
    () => player.currentTime,
    () => {
      pending = snapshot();
      if (player.state !== 'playing') return;
      if (Date.now() - lastWrite < WRITE_INTERVAL_MS) return;
      flush(pending);
    }
  );

  // Leaving playback is the point a position is most worth keeping, and it
  // falls between throttled writes.
  watch(
    () => player.state,
    (state, previous) => {
      if (previous === 'playing' && state !== 'playing') {
        flush(pending);
      }
    }
  );

  // Store the outgoing file before `pending` is replaced by the new one.
  watch(
    () => player.mediaInfo?.path,
    (path, previousPath) => {
      if (previousPath && path !== previousPath) {
        flush(pending);
      }
      pending = null;
    }
  );

  // Best effort on quit: an in-flight IPC call may not survive teardown, so
  // at most one interval's worth of progress is lost.
  function handleUnload() {
    flush(snapshot());
  }

  onMounted(() => {
    window.addEventListener('beforeunload', handleUnload);
  });

  onUnmounted(() => {
    window.removeEventListener('beforeunload', handleUnload);
    flush(snapshot());
  });
}
