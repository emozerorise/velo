import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { updateService, type UpdateOffer, type UpdateDownloadEvent } from '@/services/updateService';
import { settingsService } from '@/services/settingsService';
import { resumePositionFor } from '@/composables/usePlaybackHistory';
import { useSettingsStore } from './settingsStore';
import { useTranscriptStore } from './transcriptStore';
import { useSummaryStore } from './summaryStore';
import { usePlayerStore } from './playerStore';
import type { UpdateSettings } from '@/types/settings';

type Phase = 'idle' | 'checking' | 'available' | 'downloading' | 'downloaded'
  | 'installing' | 'ready' | 'restarting' | 'error';
const DAY = 24 * 60 * 60 * 1000;

export const useUpdateStore = defineStore('updates', () => {
  const settings = useSettingsStore();
  const manualOnly = updateService.manualOnly;
  const transcripts = useTranscriptStore();
  const summary = useSummaryStore();
  const player = usePlayerStore();
  const phase = ref<Phase>('idle');
  const currentVersion = ref('');
  const version = ref('');
  const notes = ref('');
  const error = ref<string | null>(null);
  const downloaded = ref(0);
  const total = ref(0);
  const dismissed = ref(false);
  let offer: UpdateOffer | null = null;
  let startupTimer: ReturnType<typeof setTimeout> | undefined;
  let interval: ReturnType<typeof setInterval> | undefined;

  const workBusy = computed(() => transcripts.isRunning || transcripts.isDownloading || summary.isRunning);
  const blocking = computed(() => phase.value === 'installing' || phase.value === 'restarting');
  const operating = computed(() => blocking.value || phase.value === 'checking' || phase.value === 'downloading');
  const noticeVisible = computed(() => !dismissed.value && [
    'available', 'downloading', 'downloaded', 'installing', 'ready', 'restarting',
  ].includes(phase.value));
  const progress = computed(() => total.value > 0 ? Math.min(1, downloaded.value / total.value) : -1);

  async function persist(patch: Partial<UpdateSettings>) {
    const saved = await settings.saveSettings({
      ...settings.settings,
      updates: { ...settings.settings.updates, ...patch },
    });
    if (!saved) throw new Error('Could not save update preferences');
  }

  function message(e: unknown): string {
    return e instanceof Error ? e.message : String(e);
  }

  async function checkForUpdates(manual = false) {
    if (manualOnly) return;
    if (operating.value || ['downloaded', 'ready'].includes(phase.value)) return;
    const preferences = settings.settings.updates;
    const elapsed = Date.now() - preferences.last_checked_at;
    if (!manual && (!preferences.auto_check || (elapsed >= 0 && elapsed < DAY))) return;
    phase.value = 'checking';
    error.value = null;
    try {
      // Count attempts too: an offline machine must not retry every launch.
      await persist({ last_checked_at: Date.now() });
      await offer?.close().catch(() => {});
      offer = null;
      const found = await updateService.check();
      if (!found || (!manual && found.version === preferences.skipped_version)) {
        await found?.close();
        phase.value = 'idle';
        version.value = '';
        notes.value = '';
        return;
      }
      offer = found;
      version.value = found.version;
      notes.value = found.body ?? '';
      dismissed.value = false;
      phase.value = 'available';
    } catch (e) {
      error.value = message(e);
      phase.value = 'error';
    }
  }

  function onDownload(event: UpdateDownloadEvent) {
    if (event.event === 'Started') total.value = event.data.contentLength ?? 0;
    if (event.event === 'Progress') downloaded.value += event.data.chunkLength;
  }

  async function downloadAndInstall() {
    if (!offer || workBusy.value || !['available', 'downloaded'].includes(phase.value)) return;
    error.value = null;
    dismissed.value = false;
    try {
      if (phase.value !== 'downloaded') {
        phase.value = 'downloading';
        downloaded.value = 0;
        total.value = 0;
        await offer.download(onDownload, { timeout: 600_000 });
        phase.value = 'downloaded';
      }
      // A long download gives the user time to start another meeting job.
      if (workBusy.value) return;
      phase.value = 'installing';
      await offer.install({ restartAfterInstall: false });
      phase.value = 'ready';
    } catch (e) {
      error.value = message(e);
      // The plugin keeps verified bytes when installation fails, so a
      // permissions error can be retried without another large download.
      phase.value = phase.value === 'installing' ? 'downloaded' : 'available';
    }
  }

  async function restart() {
    if (phase.value !== 'ready' || workBusy.value) return;
    phase.value = 'restarting';
    error.value = null;
    try {
      const media = player.mediaInfo;
      if (settings.settings.general.remember_playback_position && media && player.duration > 0) {
        await settingsService.recordHistory({
          path: media.path,
          file_name: media.file_name,
          last_position: resumePositionFor(player.currentTime, player.duration),
          duration: player.duration,
          last_played_timestamp: Math.floor(Date.now() / 1000),
        });
      }
      if (workBusy.value) { phase.value = 'ready'; return; }
      await updateService.relaunch();
    } catch (e) {
      error.value = message(e);
      phase.value = 'ready';
    }
  }

  async function skipVersion() {
    if (phase.value !== 'available') return;
    phase.value = 'checking';
    try {
      await persist({ skipped_version: version.value });
      await offer?.close();
      offer = null;
      dismissed.value = true;
      phase.value = 'idle';
    } catch (e) {
      error.value = message(e);
      phase.value = 'available';
    }
  }

  function stop() {
    clearTimeout(startupTimer);
    clearInterval(interval);
  }

  async function start() {
    stop();
    try { currentVersion.value = await updateService.currentVersion(); }
    catch (e) { error.value = message(e); }
    if (manualOnly) return;
    startupTimer = setTimeout(() => { void checkForUpdates(); }, 3000);
    interval = setInterval(() => { void checkForUpdates(); }, 60 * 60 * 1000);
  }

  return {
    phase, currentVersion, version, notes, error, downloaded, total, dismissed, manualOnly,
    workBusy, blocking, operating, noticeVisible, progress,
    start, stop, checkForUpdates, downloadAndInstall, restart, skipVersion,
  };
});
