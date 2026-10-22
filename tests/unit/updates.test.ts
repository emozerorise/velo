import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import UpdateNotice from '@/components/updates/UpdateNotice.vue';
import UpdateSection from '@/components/updates/UpdateSection.vue';
import { setLocale } from '@/composables/useI18n';

const { check, relaunch, invoke } = vi.hoisted(() => ({
  check: vi.fn(), relaunch: vi.fn(), invoke: vi.fn(),
}));
vi.mock('@tauri-apps/plugin-updater', () => ({ check }));
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.6.0' }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

import { useUpdateStore } from '@/stores/updateStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { usePlayerStore } from '@/stores/playerStore';

function offer() {
  return {
    version: '0.7.0', body: 'New features',
    download: vi.fn(async (progress: (event: unknown) => void) => {
      progress({ event: 'Started', data: {} });
      progress({ event: 'Progress', data: { chunkLength: 200 } });
      progress({ event: 'Finished' });
    }),
    install: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  };
}

describe('in-app updates', () => {
  beforeEach(() => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'macos');
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-10-20T10:00:00Z'));
    globalThis.document = {
      documentElement: { lang: '', classList: { add() {}, remove() {} } },
    } as unknown as Document;
    setActivePinia(createPinia());
    check.mockReset().mockResolvedValue(null);
    relaunch.mockReset().mockResolvedValue(undefined);
    invoke.mockReset().mockResolvedValue(null);
  });
  afterEach(() => {
    useUpdateStore().stop();
    vi.useRealTimers();
    vi.unstubAllEnvs();
  });

  it('never schedules or invokes in-app updates on Windows, including manual requests', async () => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'windows');
    const store = useUpdateStore();
    await store.start();
    await vi.advanceTimersByTimeAsync(2 * 86400000);
    await store.checkForUpdates(true);
    expect(check).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
    expect(store.currentVersion).toBe('0.6.0');
  });

  it.each(['en', 'th'])('shows a manual Windows download instead of unsupported update controls (%s)', async language => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'windows');
    setLocale(language);
    const html = await renderToString(createSSRApp(UpdateSection));
    expect(html).toContain('https://github.com/emozerorise/velo/releases/latest');
    expect(html).toContain(language === 'th' ? 'ดาวน์โหลดรุ่นล่าสุด' : 'Download latest version');
    expect(html).not.toContain('type="checkbox"');
  });

  it('limits automatic checks to once per day across store instances', async () => {
    const settings = useSettingsStore();
    await useUpdateStore().checkForUpdates();
    expect(check).toHaveBeenCalledTimes(1);
    expect(settings.settings.updates.last_checked_at).toBe(Date.now());
    const saved = JSON.parse(JSON.stringify(settings.settings));
    setActivePinia(createPinia());
    useSettingsStore().settings = saved;
    await useUpdateStore().checkForUpdates();
    expect(check).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(24 * 60 * 60 * 1000);
    await useUpdateStore().checkForUpdates();
    expect(check).toHaveBeenCalledTimes(2);
  });

  it('honours opt-out and lets manual checks bypass opt-out and throttling', async () => {
    useSettingsStore().settings.updates.auto_check = false;
    const store = useUpdateStore();
    await store.checkForUpdates();
    expect(check).not.toHaveBeenCalled();
    await store.checkForUpdates(true);
    expect(check).toHaveBeenCalledTimes(1);
    await store.checkForUpdates(true);
    expect(check).toHaveBeenCalledTimes(2);
  });

  it('handles a saved timestamp in the future after a clock change', async () => {
    useSettingsStore().settings.updates.last_checked_at = Date.now() + 86400000;
    await useUpdateStore().checkForUpdates();
    expect(check).toHaveBeenCalledTimes(1);
  });

  it('delays startup checks and stops its timers', async () => {
    const store = useUpdateStore();
    await store.start();
    expect(check).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(3000);
    expect(check).toHaveBeenCalledTimes(1);
    store.stop();
    await vi.advanceTimersByTimeAsync(2 * 86400000);
    expect(check).toHaveBeenCalledTimes(1);
  });

  it('does not download until clicked, skips versions persistently, and permits manual override', async () => {
    const update = offer();
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates();
    expect(store.phase).toBe('available');
    expect(store.noticeVisible).toBe(true);
    expect(update.download).not.toHaveBeenCalled();
    await store.skipVersion();
    expect(useSettingsStore().settings.updates.skipped_version).toBe('0.7.0');
    expect(store.noticeVisible).toBe(false);
    vi.advanceTimersByTime(86400000);
    await store.checkForUpdates();
    expect(store.noticeVisible).toBe(false);
    await store.checkForUpdates(true);
    expect(store.noticeVisible).toBe(true);
  });

  it('prevents concurrent checks', async () => {
    let resolve!: (value: null) => void;
    check.mockReturnValue(new Promise<null>((done) => { resolve = done; }));
    const store = useUpdateStore();
    const pending = store.checkForUpdates(true);
    await store.checkForUpdates(true);
    await vi.advanceTimersByTimeAsync(0);
    expect(check).toHaveBeenCalledTimes(1);
    resolve(null);
    await pending;
  });

  it('keeps check failures retryable without opening a notice', async () => {
    check.mockRejectedValueOnce(new Error('offline'));
    const store = useUpdateStore();
    await store.checkForUpdates();
    expect(store.error).toContain('offline');
    expect(store.noticeVisible).toBe(false);
    await store.checkForUpdates(true);
    expect(store.phase).toBe('idle');
    expect(store.error).toBe(null);
  });

  it('blocks installation during transcription, model downloads and summaries', async () => {
    const update = offer();
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    const transcripts = useTranscriptStore();
    transcripts.isRunning = true;
    await store.downloadAndInstall();
    transcripts.isRunning = false;
    transcripts.isDownloading = true;
    await store.downloadAndInstall();
    transcripts.isDownloading = false;
    useSummaryStore().stage = 'reducing';
    await store.downloadAndInstall();
    expect(update.download).not.toHaveBeenCalled();
    expect(store.phase).toBe('available');
  });

  it('defers installation if a job starts during download and reuses downloaded bytes', async () => {
    const update = offer();
    update.download.mockImplementation(async () => { useTranscriptStore().isRunning = true; });
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    await store.downloadAndInstall();
    expect(store.phase).toBe('downloaded');
    expect(update.install).not.toHaveBeenCalled();
    useTranscriptStore().isRunning = false;
    await store.downloadAndInstall();
    expect(update.download).toHaveBeenCalledTimes(1);
    expect(store.phase).toBe('ready');
    expect(relaunch).not.toHaveBeenCalled();
  });

  it('handles unknown download lengths and waits for an explicit restart', async () => {
    check.mockResolvedValue(offer());
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    await store.downloadAndInstall();
    expect(store.downloaded).toBe(200);
    expect(store.progress).toBe(-1);
    expect(store.phase).toBe('ready');
    expect(relaunch).not.toHaveBeenCalled();
    useTranscriptStore().isRunning = true;
    await store.restart();
    expect(relaunch).not.toHaveBeenCalled();
    useTranscriptStore().isRunning = false;
    await store.restart();
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it('preserves a retry path when download or installation fails', async () => {
    const update = offer();
    update.download.mockRejectedValueOnce(new Error('network failed'));
    update.install.mockRejectedValueOnce(new Error('permission denied'));
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    await store.downloadAndInstall();
    expect(store.phase).toBe('available');
    expect(store.error).toContain('network failed');
    await store.downloadAndInstall();
    expect(store.phase).toBe('downloaded');
    expect(store.error).toContain('permission denied');
    await store.downloadAndInstall();
    expect(store.phase).toBe('ready');
    expect(update.download).toHaveBeenCalledTimes(2);
  });

  it('waits for playback history persistence before relaunching', async () => {
    check.mockResolvedValue(offer());
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    await store.downloadAndInstall();
    const player = usePlayerStore();
    player.mediaInfo = {
      path: '/meeting.mp3', file_name: 'meeting.mp3', duration: 300,
      width: 0, height: 0, video_codec: null, audio_codec: 'mp3', fps: 0, hwdec_current: null,
    };
    player.duration = 300;
    player.currentTime = 120;
    let persisted!: () => void;
    invoke.mockImplementation(async (command: string) => {
      if (command === 'history_record') await new Promise<void>((done) => { persisted = done; });
    });
    const restarting = store.restart();
    await nextTick();
    expect(relaunch).not.toHaveBeenCalled();
    persisted();
    await restarting;
    expect(invoke).toHaveBeenCalledWith('history_record', expect.objectContaining({
      item: expect.objectContaining({ path: '/meeting.mp3', last_position: 120 }),
    }));
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it('does not close an offer while installation can start during skip persistence', async () => {
    const update = offer();
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    let saved!: () => void;
    invoke.mockReturnValue(new Promise<void>((done) => { saved = done; }));
    const skipping = store.skipVersion();
    await store.downloadAndInstall();
    expect(update.download).not.toHaveBeenCalled();
    saved();
    await skipping;
    expect(store.phase).toBe('idle');
  });

  it('renders an unobtrusive notice, disables busy actions and escapes release notes in both languages', async () => {
    const update = offer();
    update.body = '<script>alert(1)</script>';
    check.mockResolvedValue(update);
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    useTranscriptStore().isRunning = true;
    for (const language of ['en', 'th']) {
      setLocale(language);
      const notice = await renderToString(createSSRApp(UpdateNotice));
      expect(notice).toContain('0.7.0');
      expect(notice).toMatch(/<button[^>]*\sdisabled(?:[=>\s])/);
      const settings = await renderToString(createSSRApp(UpdateSection));
      expect(settings).toContain('&lt;script&gt;');
      expect(settings).not.toContain('<script>');
      store.dismissed = true;
      expect(await renderToString(createSSRApp(UpdateNotice))).not.toContain('<aside');
      store.dismissed = false;
    }
    setLocale('en');
  });

  it('disables manual checks until startup has loaded saved settings and the current version', async () => {
    const store = useUpdateStore();
    const pending = await renderToString(createSSRApp(UpdateSection));
    expect(pending).toMatch(/<button[^>]*\sdisabled(?:[=>\s])/);
    expect(pending).toMatch(/<input[^>]*\sdisabled(?:[=>\s])/);
    await store.start();
    const ready = await renderToString(createSSRApp(UpdateSection));
    expect(ready).not.toMatch(/<button[^>]*\sdisabled(?:[=>\s])/);
    expect(ready).not.toMatch(/<input[^>]*\sdisabled(?:[=>\s])/);
  });

  it('keeps restart failures retryable and blocks duplicate relaunches', async () => {
    check.mockResolvedValue(offer());
    const store = useUpdateStore();
    await store.checkForUpdates(true);
    await store.downloadAndInstall();
    relaunch.mockRejectedValueOnce(new Error('relaunch failed'));
    await store.restart();
    expect(store.phase).toBe('ready');
    expect(store.error).toContain('relaunch failed');
    let complete!: () => void;
    relaunch.mockReturnValue(new Promise<void>((done) => { complete = done; }));
    const restart = store.restart();
    expect(store.blocking).toBe(true);
    await store.restart();
    expect(relaunch).toHaveBeenCalledTimes(2);
    complete();
    await restart;
  });
});
