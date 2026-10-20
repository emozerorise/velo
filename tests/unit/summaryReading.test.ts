import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import SummaryLine from '@/components/transcript/SummaryLine.vue';
import { groupSummarySections, parseInline, parseSummary } from '@/utils/summaryMarkdown';
import { usePlayerStore } from '@/stores/playerStore';
import { useSummaryStore } from '@/stores/summaryStore';
import { setLocale } from '@/composables/useI18n';
import { useSettingsStore } from '@/stores/settingsStore';

describe('summary reading sections', () => {
  it('treats headings matching JavaScript property names as ordinary model text', () => {
    expect(groupSummarySections('## constructor\nText')[0].kind).toBe('other');
  });
  it.each([
    ['Overview', 'Topics discussed', 'Decisions', 'Action items', 'Open questions'],
    ['ภาพรวม', 'ประเด็นที่คุยกัน', 'สิ่งที่ตัดสินใจ', 'งานที่ต้องทำ', 'เรื่องที่ยังค้าง'],
  ])('prioritises outcomes without losing the original blocks (%s)', (...titles) => {
    const markdown = titles
      .map((title, i) => `## ${title}\n- Point ${i} [00:10]\n  - Detail ${i}`)
      .join('\n');
    const sections = groupSummarySections(markdown);
    expect(sections.map((section) => section.kind)).toEqual([
      'overview',
      'decisions',
      'actions',
      'topics',
      'questions',
    ]);
    expect(sections.flatMap((section) => section.nodes)).toHaveLength(10);
    expect(sections[1].nodes).toEqual(parseSummary('- Point 2 [00:10]\n  - Detail 2'));
  });

  it('keeps preambles, custom headings, duplicate headings and empty sections', () => {
    const sections = groupSummarySections(
      'Preamble\n## Custom\nText\n## Custom\n- Other\n## Decisions\n-',
    );
    expect(sections.map(({ title }) => title)).toEqual([null, 'Decisions', 'Custom', 'Custom']);
    expect(sections.flatMap(({ nodes }) => nodes)).toEqual([
      ...parseSummary('Preamble'),
      ...parseSummary('Text'),
      ...parseSummary('- Other'),
    ]);
    expect(sections[1].nodes).toEqual([]);
    expect(groupSummarySections('')).toEqual([]);
  });
});

describe('summary reading presentation', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
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
    useSummaryStore().summary = {
      path: '/sample.mp3',
      model: 'test-model',
      language: 'en',
      created_at: 0,
      source_segments: 0,
      markdown:
        '## Overview\nA short overview.\n## Topics discussed\n- Context [00:10]\n## Decisions\n- Ship it [00:20]\n## Action items\n- **Owner**: follow up [00:30]',
    };
  });

  it('hides citation controls without changing the cached summary', async () => {
    const settings = useSettingsStore();
    settings.settings.general = { ...settings.settings.general, show_summary_timestamps: false };
    const original = useSummaryStore().summary?.markdown;
    const html = await renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
    expect(html).not.toContain('Jump to 00:20');
    expect(html).not.toContain('ข้ามไปที่ 00:20');
    expect(html).toContain('Ship it');
    expect(useSummaryStore().summary?.markdown).toBe(original);
  });

  it.each(['en', 'th'])('offers timestamp and prompt controls (%s)', async (locale) => {
    setLocale(locale);
    const html = await renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
    expect(html).toContain(locale === 'th' ? 'แสดงเวลา' : 'Show timestamps');
    expect(html).toContain(locale === 'th' ? 'แก้ prompt' : 'Edit prompt');
  });

  it.each(['en', 'th'])(
    'renders labelled controls and collapsible detail, with no request (%s)',
    async (locale) => {
      setLocale(locale);
      const store = useSummaryStore();
      const original = store.summary?.markdown;
      const html = await renderToString(createSSRApp(SummaryTab).use(getActivePinia()!));
      expect(html).toContain('<details');
      expect(html).not.toContain('<details open');
      expect(html.indexOf('Ship it')).toBeLessThan(html.indexOf('Context'));
      expect(html).toContain(locale === 'th' ? 'ขยายพื้นที่อ่าน' : 'Expand reading area');
      expect(html).toContain(locale === 'th' ? 'คัดลอกบทสรุป' : 'Copy summary');
      expect(html).toContain('test-model');
      expect(store.summary?.markdown).toBe(original);
      expect(store.isRunning).toBe(false);
    },
  );

  it('separates all citation buttons from the prose and keeps bold text and safe escaping', async () => {
    setLocale('en');
    const html = await renderToString(
      createSSRApp(SummaryLine, {
        parts: parseInline('Before [00:10] **owner** after [01:02:03] <script>bad()</script>'),
      }),
    );
    expect(html).toContain('aria-label="Jump to 00:10"');
    expect(html).toContain('aria-label="Jump to 01:02:03"');
    expect(html.indexOf('after')).toBeLessThan(html.indexOf('<button'));
    expect(html).toContain('<strong');
    expect(html).toContain('&lt;script&gt;');
    expect(html).not.toContain('<script>');
  });
});
