import { describe, it, expect, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import { usePlayerStore } from '@/stores/playerStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { setLocale } from '@/composables/useI18n';
import { mountView } from './helpers/mountView';

describe('source-aware summary navigation', () => {
  it('regenerates without deleting the previous cache or changing its visible result', async () => {
    setActivePinia(createPinia());
    setLocale('en');
    usePlayerStore().mediaInfo = { path: '/sample.mp3' } as NonNullable<
      ReturnType<typeof usePlayerStore>['mediaInfo']
    >;
    const store = useSummaryStore();
    store.summary = {
      path: '/sample.mp3',
      model: 'old',
      language: 'en',
      created_at: 0,
      source_segments: 1,
      markdown: 'Previous summary',
    };
    const discard = vi.spyOn(store, 'discard').mockResolvedValue();
    const summarise = vi.spyOn(store, 'summarise').mockResolvedValue();
    const view = mountView(SummaryTab, {}, getActivePinia()!);
    view
      .nodes()
      .find(
        (node) =>
          node.tagName === 'BUTTON' &&
          node.children.some((child) => child.text.includes('Summarise again')),
      )!
      .trigger('click');
    await nextTick();
    expect(summarise).toHaveBeenCalledOnce();
    expect(discard).not.toHaveBeenCalled();
    expect(store.summary.markdown).toBe('Previous summary');
    view.unmount();
    vi.restoreAllMocks();
  });
  it('seeks exact starts; other-file, absent, hidden and invented labels cannot seek', async () => {
    setActivePinia(createPinia());
    setLocale('en');
    usePlayerStore().mediaInfo = {
      path: '/sample.mp3',
      file_name: 'sample.mp3',
      duration: 60,
      video_codec: null,
      audio_codec: 'mp3',
      width: 0,
      height: 0,
      fps: 0,
      hwdec_current: null,
    };
    const store = useSummaryStore();
    store.summary = {
      path: '/sample.mp3',
      model: 'test',
      language: 'en',
      created_at: 0,
      source_segments: 1,
      markdown: 'Supported [00:30] invented [00:31]',
    };
    const source = useTranscriptStore();
    source.transcript = {
      path: '/sample.mp3',
      language: 'en',
      model: 'test',
      prompt: '',
      created_at: 0,
      segments: [{ start: 30.9, end: 37.38, text: 'source' }],
    };
    const seek = vi.spyOn(usePlayerStore(), 'seekAbsolute').mockResolvedValue();
    const view = mountView(SummaryTab, {}, getActivePinia()!);
    const button = () =>
      view.nodes().find((node) => node.props['aria-label'] === 'Jump to 00:30');
    expect(button()).toBeUndefined();
    expect(view.nodes().some((node) => node.text.includes('[00:31]'))).toBe(false);
    useSettingsStore().settings.general.show_summary_timestamps = true;
    await nextTick();
    expect(button()).toBeDefined();
    expect(
      view.nodes().some((node) => node.props['aria-label'] === 'Jump to 00:31'),
    ).toBe(false);
    button()!.trigger('click');
    expect(seek).toHaveBeenCalledWith(30.9);
    useSettingsStore().settings.general.show_summary_timestamps = false;
    await nextTick();
    expect(button()).toBeUndefined();
    useSettingsStore().settings.general.show_summary_timestamps = true;
    source.transcript.path = '/other.mp3';
    await nextTick();
    expect(button()).toBeUndefined();
    source.transcript = null;
    await nextTick();
    expect(button()).toBeUndefined();
    view.unmount();
    seek.mockRestore();
  });
});
