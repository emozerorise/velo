import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp, nextTick } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { invoke } from '@tauri-apps/api/core';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { usePlayerStore } from '@/stores/playerStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { open } from '@tauri-apps/plugin-dialog';
import { setLocale } from '@/composables/useI18n';
import WorkspaceRail from '@/components/workspace/WorkspaceRail.vue';
import TranscriptPanel from '@/components/transcript/TranscriptPanel.vue';
import TranscriptTab from '@/components/transcript/TranscriptTab.vue';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import PlayerShelf from '@/components/player/PlayerShelf.vue';
import { ref } from 'vue';
import { useVideoMask } from '@/composables/useVideoMask';
import { mountComposable } from './helpers/mountComposable';
import { mountView, TestNode } from './helpers/mountView';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}));
const render = (component: Parameters<typeof createSSRApp>[0]) =>
  renderToString(createSSRApp(component).use(getActivePinia()!));

function loaded() {
  usePlayerStore().mediaInfo = {
    path: '/meeting.mp4',
    file_name: 'meeting.mp4',
    duration: 60,
    video_codec: 'h264',
    audio_codec: 'aac',
    width: 1280,
    height: 720,
    fps: 30,
    hwdec_current: null,
  };
  useTranscriptStore().transcript = {
    path: '/meeting.mp4',
    model: 'test',
    language: 'en',
    prompt: '',
    created_at: 0,
    segments: [
      { start: 0, end: 20, text: 'Important decision' },
      { start: 20, end: 60, text: 'Other topic' },
    ],
  };
}

describe('summary-first workspace tools', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    setLocale('en');
    vi.clearAllMocks();
    vi.stubGlobal(
      'window',
      Object.assign(new EventTarget(), {
        innerWidth: 1000,
        innerHeight: 700,
        devicePixelRatio: 2,
        matchMedia: () => new EventTarget(),
      }),
    );
    vi.stubGlobal('requestAnimationFrame', () => 1);
    vi.stubGlobal('cancelAnimationFrame', () => {});
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe() {}
        disconnect() {}
      },
    );
    vi.stubGlobal('HTMLElement', TestNode);
    vi.stubGlobal('HTMLInputElement', class extends TestNode {});
    vi.stubGlobal('HTMLTextAreaElement', class extends TestNode {});
  });
  afterEach(() => vi.unstubAllGlobals());

  it('keeps independent preferences without making playback or AI calls', () => {
    const workspace = useWorkspaceStore();
    expect([workspace.view, workspace.chatOpen, workspace.playerOpen]).toEqual([
      'summary',
      false,
      false,
    ]);
    workspace.toggleTranscript();
    expect(workspace.view).toBe('transcript');
    workspace.toggleChat();
    expect([workspace.view, workspace.chatVisible]).toEqual(['summary', true]);
    workspace.toggleTranscript();
    expect([workspace.chatOpen, workspace.chatVisible]).toEqual([true, false]);
    workspace.toggleTranscript();
    expect(workspace.chatVisible).toBe(true);
    workspace.togglePlayer();
    workspace.togglePlayer();
    expect(workspace.playerOpen).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('new_file_resets_only_visibility', () => {
    loaded();
    const workspace = useWorkspaceStore();
    const transcript = useTranscriptStore().transcript;
    workspace.toggleChat();
    workspace.togglePlayer();
    workspace.toggleTranscript();
    usePlayerStore().currentTime = 37;
    workspace.resetForMedia();
    expect([
      workspace.view,
      workspace.chatOpen,
      workspace.playerOpen,
      workspace.mode,
    ]).toEqual(['summary', false, false, 'reading']);
    expect(usePlayerStore().currentTime).toBe(37);
    expect(useTranscriptStore().transcript).toBe(transcript);
    expect(invoke).not.toHaveBeenCalled();
  });

  it.each(['en', 'th'])(
    'fresh_welcome_never_requests_ai_model_or_mic (%s)',
    async (locale) => {
      setLocale(locale);
      const html = await render(TranscriptPanel);
      expect(html).toContain(
        locale === 'th'
          ? 'เริ่มจากเรื่องที่อยากเข้าใจ'
          : 'Start with something you want to understand',
      );
      expect(html).toContain('data-testid="welcome-open-file"');
      expect(html).not.toContain('role="tablist"');
      expect(html).not.toContain('Download speech model');
      expect(html).not.toContain('No model set up yet');
      expect(invoke).not.toHaveBeenCalled();
      for (const label of locale === 'th'
        ? ['บทถอดเสียง', 'คุยต่อจากสรุป', 'ตัวเล่น']
        : ['Transcript', 'Discuss this summary', 'Player']) {
        expect(html).toContain(`aria-label="${label}"`);
      }
      expect(html.match(/aria-pressed="false"/g)).toHaveLength(3);
    },
  );

  it('rail_space_activates_once', async () => {
    const workspace = useWorkspaceStore();
    const view = mountView(WorkspaceRail, {}, getActivePinia()!);
    const event = new Event('keydown', { cancelable: true });
    Object.defineProperty(event, 'key', { value: ' ' });
    view.find('tool-player').trigger('keydown', event);
    await nextTick();
    expect(event.defaultPrevented).toBe(true);
    expect(workspace.playerOpen).toBe(true);
    view.find('tool-player').trigger('click');
    expect(workspace.playerOpen).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });

  it('chat_from_transcript_returns_to_summary', async () => {
    const workspace = useWorkspaceStore();
    const view = mountView(WorkspaceRail, {}, getActivePinia()!);
    view.find('tool-transcript').trigger('click');
    expect(workspace.view).toBe('transcript');
    view.find('tool-chat').trigger('click');
    await nextTick();
    expect([workspace.view, workspace.chatVisible]).toEqual(['summary', true]);
    expect(view.find('tool-chat').props['aria-pressed']).toBe(true);
    view.unmount();
  });

  it('cached_summary_reads_without_provider', async () => {
    loaded();
    useSummaryStore().status = {
      provider: 'ollama',
      base_url: '',
      model: '',
      configured: false,
      remote: false,
      has_key: false,
    };
    useSummaryStore().summary = {
      path: '/meeting.mp4',
      model: 'test',
      language: 'en',
      markdown: '## Overview\nCached summary remains readable',
      created_at: 0,
      source_segments: 2,
    };
    expect(await render(SummaryTab)).toContain(
      'Cached summary remains readable',
    );
    expect(invoke).not.toHaveBeenCalled();
  });

  it('reading_summary_has_one_heading_with_its_toolbar', async () => {
    loaded();
    useSummaryStore().summary = {
      path: '/meeting.mp4',
      model: 'test',
      language: 'en',
      created_at: 0,
      source_segments: 2,
      markdown: '## Overview\nImportant decision',
    };
    const html = await renderToString(
      createSSRApp(SummaryTab, { reading: true }).use(getActivePinia()!),
    );
    expect(html).toMatch(/<h1[^>]*>Content summary<\/h1>/);
    expect(html).toContain('Copy summary');
    expect(html).toContain('Important decision');
  });

  it('shelf_close_does_not_pause_audio', async () => {
    loaded();
    usePlayerStore().state = 'playing';
    usePlayerStore().currentTime = 37;
    const workspace = useWorkspaceStore();
    workspace.playerOpen = true;
    const view = mountView(PlayerShelf, {}, getActivePinia()!);
    view.find('close-player').trigger('click');
    await nextTick();
    expect(workspace.playerOpen).toBe(false);
    expect([usePlayerStore().state, usePlayerStore().currentTime]).toEqual([
      'playing',
      37,
    ]);
    expect(
      vi
        .mocked(invoke)
        .mock.calls.some(([name]) => /pause|seek|load|stop/.test(name)),
    ).toBe(false);
    view.unmount();
  });

  it('audio_only_has_no_empty_video_hole', async () => {
    loaded();
    usePlayerStore().mediaInfo!.video_codec = null;
    usePlayerStore().mediaInfo!.width = 0;
    usePlayerStore().mediaInfo!.height = 0;
    useWorkspaceStore().playerOpen = true;
    const html = await render(PlayerShelf);
    expect(html).not.toContain('data-testid="video-preview"');
    expect(html).toContain('Play (Space)');
  });

  it('compact_transport_reserves_a_timeline_row_without_wrapping_actions', async () => {
    loaded();
    useWorkspaceStore().playerOpen = true;
    const html = await render(PlayerShelf);
    expect(html).toContain('compact-controls');
    expect(html).toContain('col-span-full row-start-1');
    expect(html).toContain('Seek -5s');
    expect(html).toContain('Seek +5s');
  });

  it('modal_masks_native_preview', async () => {
    loaded();
    useWorkspaceStore().playerOpen = true;
    const open = ref(false);
    const dispose = mountComposable(() => useVideoMask(() => open.value));
    const view = mountView(PlayerShelf, {}, getActivePinia()!);
    expect(view.find('video-preview').props['data-masked']).toBe(false);
    open.value = true;
    await nextTick();
    expect(view.find('video-preview').props['data-masked']).toBe(true);
    open.value = false;
    await nextTick();
    expect(view.find('video-preview').props['data-masked']).toBe(false);
    view.unmount();
    dispose();
  });

  it.each(['en', 'th'])(
    'compact_tools_preserve_draft_and_search (%s)',
    async (locale) => {
      setLocale(locale);
      loaded();
      vi.stubGlobal('document', { activeElement: null });
      Object.assign(window, { innerWidth: 640, innerHeight: 400 });
      const view = mountView(TranscriptPanel, {}, getActivePinia()!);
      const workspace = useWorkspaceStore();
      expect(workspace.compact).toBe(true);
      view.find('tool-transcript').trigger('click');
      await nextTick();
      const search = view.find('transcript-search');
      search.value = 'decision';
      search.trigger('input');
      useTranscriptStore().prompt = 'Retained vocabulary draft';
      view.find('tool-chat').trigger('click');
      view.find('tool-player').trigger('click');
      await nextTick();
      expect([
        workspace.chatOpen,
        workspace.chatVisible,
        workspace.playerOpen,
      ]).toEqual([true, false, true]);
      view.find('tool-chat').trigger('click');
      await nextTick();
      expect([workspace.chatVisible, workspace.playerOpen]).toEqual([
        true,
        false,
      ]);
      view.find('tool-transcript').trigger('click');
      await nextTick();
      expect(search.value).toBe('decision');
      expect(useTranscriptStore().prompt).toBe('Retained vocabulary draft');
      Object.assign(window, { innerWidth: 1440, innerHeight: 1024 });
      window.dispatchEvent(new Event('resize'));
      await nextTick();
      expect(workspace.compact).toBe(false);
      expect(view.find('transcript-search')).toBe(search);
      view.unmount();
    },
  );

  it.each(['transcript', 'chat', 'player'])(
    'escape_restores_own_trigger (%s)',
    async (tool) => {
      const view = mountView(WorkspaceRail, {}, getActivePinia()!);
      const button = view.find(`tool-${tool}`);
      button.trigger('click');
      await nextTick();
      const event = Object.assign(new Event('keydown', { cancelable: true }), {
        key: 'Escape',
      });
      button.trigger('keydown', event);
      await nextTick();
      expect(view.find(`tool-${tool}`).props['aria-pressed']).toBe(false);
      expect(button.focusCount).toBe(1);
      expect(useWorkspaceStore().mode).toBe('reading');
      view.unmount();
    },
  );

  it('focused_button_space_never_reaches_global_player', async () => {
    const dispose = mountComposable(() =>
      useKeyboardShortcuts({ register: true }),
    );
    const view = mountView(WorkspaceRail, {}, getActivePinia()!);
    const event = Object.assign(new Event('keydown', { cancelable: true }), {
      key: ' ',
      code: 'Space',
    });
    view.find('tool-player').trigger('keydown', event);
    if (!event.cancelBubble) window.dispatchEvent(event);
    if (!event.defaultPrevented) view.find('tool-player').trigger('click');
    expect(useWorkspaceStore().playerOpen).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
    dispose();
  });

  it('welcome_open_file_uses_existing_dialog', async () => {
    const view = mountView(TranscriptPanel, {}, getActivePinia()!);
    view.find('welcome-open-file').trigger('click');
    await nextTick();
    expect(open).toHaveBeenCalledOnce();
    expect(open).toHaveBeenCalledWith(
      expect.objectContaining({
        multiple: true,
        filters: [
          expect.objectContaining({
            extensions: expect.arrayContaining(['mp3', 'mp4', 'wav']),
          }),
        ],
      }),
    );
    expect(useSummaryStore().isRunning).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });

  it.each([
    [PlayerShelf, 'empty-player-open-file'],
    [TranscriptTab, 'empty-transcript-open-file'],
  ] as const)(
    'empty tool offers the existing open-file action (%s)',
    async (component, testId) => {
      if (testId === 'empty-player-open-file')
        useWorkspaceStore().playerOpen = true;
      const view = mountView(component, {}, getActivePinia()!);
      view.find(testId).trigger('click');
      await nextTick();
      expect(open).toHaveBeenCalledOnce();
      expect(invoke).not.toHaveBeenCalled();
      view.unmount();
    },
  );

  it('escape_in_search_does_not_close_the_transcript', async () => {
    loaded();
    vi.stubGlobal('document', { activeElement: null });
    const view = mountView(TranscriptPanel, {}, getActivePinia()!);
    useWorkspaceStore().toggleTranscript();
    await nextTick();
    const event = Object.assign(new Event('keydown', { cancelable: true }), {
      key: 'Escape',
    });
    view.find('transcript-search').trigger('keydown', event);
    expect(useWorkspaceStore().view).toBe('transcript');
    view.unmount();
  });

  it('reader_search_scroll_survives_tool_changes', async () => {
    loaded();
    vi.stubGlobal('HTMLElement', TestNode);
    vi.stubGlobal('document', { activeElement: null });
    const view = mountView(TranscriptPanel, {}, getActivePinia()!);
    const workspace = useWorkspaceStore();
    workspace.toggleTranscript();
    await nextTick();
    const search = view.find('transcript-search');
    search.value = 'decision';
    search.trigger('input');
    await nextTick();
    const scroll = view.find('transcript-scroll');
    scroll.scrollTop = 123;
    workspace.toggleChat();
    workspace.togglePlayer();
    await nextTick();
    workspace.toggleTranscript();
    await nextTick();
    expect(view.find('transcript-search')).toBe(search);
    expect(search.value).toBe('decision');
    expect(view.find('transcript-scroll')).toBe(scroll);
    expect(scroll.scrollTop).toBe(123);
    expect(view.nodes().some((node) => node.text === 'Other topic')).toBe(
      false,
    );
    expect(invoke).not.toHaveBeenCalled();
    view.unmount();
  });
});
