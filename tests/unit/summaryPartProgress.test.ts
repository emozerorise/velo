import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { invoke } from '@tauri-apps/api/core';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import SummaryProgress from '@/components/transcript/SummaryProgress.vue';
import { useSummaryStore } from '@/stores/summaryStore';
import { usePlayerStore } from '@/stores/playerStore';
import { setLocale } from '@/composables/useI18n';
import type { ChainStage } from '@/types/summary';

const { handlers } = vi.hoisted(() => ({ handlers: new Map<string, (event: {payload: unknown}) => void>() }));
vi.mock('@tauri-apps/api/event', () => ({listen: vi.fn(async (name, cb) => { handlers.set(name, cb); return () => handlers.delete(name); })}));
vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn().mockResolvedValue(undefined)}));
const stages = ['writing_parts', 'checking_consistency', 'replacing_parts', 'assembling_summary'];
const previous = {path: '/voice.aac', model: 'model', language: 'th', markdown: 'Old complete [00:10]', created_at: 1, source_segments: 2};
beforeEach(() => { setActivePinia(createPinia()); handlers.clear(); vi.clearAllMocks(); usePlayerStore().mediaInfo = {path: previous.path, file_name: 'voice.aac', duration: 100, video_codec: null, audio_codec: 'aac', width: 0, height: 0, fps: 0, hwdec_current: null}; });
it.each(stages)('keeps %s running, indeterminate, step four and cancellable', async stage => {
  const store = useSummaryStore();
  await store.initListeners();
  store.summary = {...previous}; store.chained = true;
  handlers.get('velo://summary-progress')!({payload: {path: previous.path, stage, done: 2, total: 4}});
  expect(store.summary?.markdown).toBe(previous.markdown);
  expect(store.progress).toBe(-1); expect(store.step).toBe(4); expect(store.isRunning).toBe(true);
  await store.cancel();
  expect(invoke).toHaveBeenCalledWith('summary_cancel');
  handlers.get('velo://summary-error')!({payload: {path: previous.path, message: 'Cancelled'}});
  expect(store.error).toBeNull(); expect(store.summary?.markdown).toBe(previous.markdown);
});
it('shows excerpt writing and one combination without an AI review step', async () => {
  setLocale('en');
  const html = await renderToString(createSSRApp(SummaryProgress, {label: 'Combining excerpts', stage: 'assembling_summary', progress: -1, formatLabel: 'Meeting', streamed: ''}));
  expect(html).toContain('Combining excerpt summaries');
  expect(html).toContain('Summarizing excerpts');
  expect(html).not.toContain('Checking source consistency');
  expect(html).not.toContain('aria-valuenow');
});
it('ignores old-path events and accepts a committed ready result after late cancellation', async () => {
  const store = useSummaryStore(); await store.initListeners(); store.summary = {...previous};
  handlers.get('velo://summary-ready')!({payload: {...previous, path: '/other.aac', markdown: 'Wrong file'}});
  expect(store.summary?.markdown).toBe(previous.markdown);
  handlers.get('velo://summary-progress')!({payload: {path: '/other.aac', stage: 'writing_parts', done: 0, total: 1}});
  expect(store.stage).toBeNull();
  store.stage = 'writing_parts' as ChainStage; await store.cancel();
  handlers.get('velo://summary-ready')!({payload: {...previous, markdown: 'Committed'}});
  expect(store.summary?.markdown).toBe('Committed'); expect(store.isRunning).toBe(false);
});
it.each(['en', 'th'])('localizes bounded failures without claiming a previous result on first failure (%s)', async locale => {
  setLocale(locale); const store = useSummaryStore();
  for (const error of ['summary_work_limit', 'summary_review_unstable', 'summary_review_invalid', 'summary_layout_unsupported', 'summary_part_layout_invalid', 'summary_no_supported_content']) {
    store.error = error;
    let html = await renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
    expect(html).not.toContain(error); expect(html).not.toContain('กำลังแสดงฉบับเดิม');
    store.summary = {...previous};
    html = await renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
    expect(html).toContain(locale === 'th' ? 'สรุปใหม่ไม่สำเร็จ กำลังแสดงฉบับเดิม' : 'Showing the previous summary');
    store.summary = null;
  }
});
