import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp, nextTick, ref } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { invoke } from '@tauri-apps/api/core';
import TranscriptPanel from '@/components/transcript/TranscriptPanel.vue';
import TranscriptTab from '@/components/transcript/TranscriptTab.vue';
import ControlOverlay from '@/components/player/ControlOverlay.vue';
import ReadingSidebar from '@/components/playlist/ReadingSidebar.vue';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { setLocale } from '@/composables/useI18n';
import { mountComposable } from './helpers/mountComposable';
import { useReadingScroll } from '@/composables/useReadingScroll';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('reading workspace compatibility', () => {
  it('exposes file and folder help labels for the compact import controls', async () => {
    const html = await renderToString(createSSRApp(ReadingSidebar, { open: true }).use(getActivePinia()!));
    expect(html).toContain('title="Open file"');
    expect(html).toContain('aria-label="Open folder"');
  });
  beforeEach(() => {
    setActivePinia(createPinia());
    setLocale('en');
    vi.clearAllMocks();
  });

  it('opens reading without requiring a media file or invoking a model', () => {
    const store = useTranscriptStore();
    expect(store.isPanelOpen).toBe(true);
    expect(store.isReadingMode).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('reveals video with the transcript open, and returns without changing playback or content', () => {
    const store = useTranscriptStore();
    const player = usePlayerStore();
    const playlist = usePlaylistStore();
    player.currentTime = 37;
    player.state = 'paused';
    store.transcript = {
      path: '/meeting.mp4',
      language: 'th',
      model: 'test',
      prompt: '',
      created_at: 0,
      segments: [{ start: 0, end: 60, text: 'Existing transcript' }],
    };
    playlist.isDrawerOpen = true;
    expect(store.showVideo).toBeTypeOf('function');
    store.showVideo();
    expect(store.isReadingMode).toBe(false);
    expect(store.isPanelOpen).toBe(true);
    expect(playlist.isDrawerOpen).toBe(false);
    store.isPanelOpen = false;
    expect(store.showReading).toBeTypeOf('function');
    store.showReading();
    expect(store.isReadingMode).toBe(true);
    expect(store.isPanelOpen).toBe(true);
    expect(player.currentTime).toBe(37);
    expect(player.state).toBe('paused');
    expect(store.transcript?.segments[0].text).toBe('Existing transcript');
    expect(invoke).not.toHaveBeenCalled();
  });

  it.each(['en', 'th'])(
    'renders the playlist and labelled tool icons (%s)',
    async (locale) => {
      setLocale(locale);
      const playlist = usePlaylistStore();
      const { useLibraryStore } = await import('@/stores/libraryStore');
      useLibraryStore().snapshot.recordings = [{id:'one',path:'/meeting.mp4',display_name:null,project_id:null,pinned:false,added_at:0,duration:null}];
      playlist.items = [{ id: 'one', path: '/meeting.mp4', fileName: 'meeting.mp4' }];
      playlist.currentIndex = 0;
      const html = await renderToString(createSSRApp(TranscriptPanel).use(getActivePinia()!));
      expect(html).toContain('<aside');
      expect(html).toContain('meeting.mp4');
      expect(html).toContain(locale === 'th' ? 'ตัวเล่น' : 'Player');
      expect(html).toContain(locale === 'th' ? 'เปิดไฟล์' : 'Open file');
      expect(html).not.toContain(
        locale === 'th' ? 'ปิดบทถอดเสียงและสรุป' : 'Close transcript and summary',
      );
    },
  );

  it('reads a cached transcript even when the speech model is no longer installed', async () => {
    const store = useTranscriptStore();
    usePlayerStore().mediaInfo = {
      path: '/meeting.mp4',
      file_name: 'meeting.mp4',
      duration: 60,
      video_codec: null,
      audio_codec: 'aac',
      width: 0,
      height: 0,
      fps: 0,
      hwdec_current: null,
    };
    store.engine = {
      ready: false,
      model_path: null,
      model_name: 'test',
      model_bytes: 10,
      downloading: false,
      model_managed: false,
    };
    store.transcript = {
      path: '/meeting.mp4',
      language: 'th',
      model: 'test',
      prompt: '',
      created_at: 0,
      segments: [{ start: 0, end: 60, text: 'Cached words remain readable' }],
    };
    const html = await renderToString(
      createSSRApp(TranscriptTab, { reading: true }).use(getActivePinia()!),
    );
    expect(html).toContain('Cached words remain readable');
    expect(html).not.toContain('Download speech model');
    expect(invoke).not.toHaveBeenCalled();
  });

  it('asks for a file before offering to download a model in an empty workspace', async () => {
    useTranscriptStore().engine = {
      ready: false,
      model_path: null,
      model_name: 'test',
      model_bytes: 10,
      downloading: false,
      model_managed: false,
    };
    const html = await renderToString(
      createSSRApp(TranscriptTab, { reading: true }).use(getActivePinia()!),
    );
    expect(html).toContain('Open a video or audio file first');
    expect(html).not.toContain('Download speech model');
  });

  it('keeps compact video playback usable without track selectors taking the drawer space', async () => {
    const player = usePlayerStore();
    player.duration = 60;
    player.currentTime = 37;
    const html = await renderToString(
      createSSRApp(ControlOverlay, { compact: true }).use(getActivePinia()!),
    );
    expect(html).toContain('00:37');
    expect(html).toContain('01:00');
    expect(html).toContain('Play (Space)');
    expect(html).not.toContain('Audio Tracks');
    expect(html).not.toContain('Subtitles');
    expect(html).not.toContain('Previous');
  });

  it('restores the reading scroll position after the narrower video view clamps its scroll', async () => {
    const store = useTranscriptStore();
    const element = Object.assign(new EventTarget(), { scrollTop: 300 });
    const viewport = ref(element as unknown as HTMLElement);
    const unmount = mountComposable(() => useReadingScroll(viewport, () => store.isReadingMode));
    element.dispatchEvent(new Event('scroll'));
    store.showVideo();
    // The parent changes the panel width before the child's pre-flush watcher
    // runs, so browser layout can already have clamped the old scroll value.
    viewport.value.scrollTop = 0;
    await nextTick();
    store.showReading();
    await nextTick();
    await nextTick();
    expect(viewport.value.scrollTop).toBe(300);
    unmount();
  });
});
