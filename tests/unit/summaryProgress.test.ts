import { beforeEach, describe, expect, it } from 'vitest';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import SummaryProgress from '@/components/transcript/SummaryProgress.vue';
import { setLocale } from '@/composables/useI18n';
import type { ChainStage } from '@/types/summary';

describe('summary progress card', () => {
  beforeEach(() => setLocale('en'));
  const render = (stage: ChainStage, progress = -1) => renderToString(createSSRApp(SummaryProgress, {
    stage, progress, label: 'Current stage', formatLabel: 'Meeting summary', streamed: '',
  }));
  it.each([
    ['writing_summary', 'AI is writing your summary'],
    ['checking_references', 'Checking timestamp references'],
    ['extracting_evidence', 'Read source'],
    ['checking_evidence', 'Check evidence'],
    ['composing_summary', 'Arrange summary'],
  ] as const)('marks the actual %s stage without inventing a percentage', async (stage, text) => {
    const html = await render(stage);
    expect(html).toContain('role="progressbar"');
    expect(html).not.toContain('aria-valuenow');
    expect(html.match(/<li[^>]*aria-current="step"[^>]*>.*?<\/li>/)?.[0]).toContain(text);
    expect(html).toContain('Cancel');
  });
  it('uses measured progress when available', async () => {
    expect(await render('transcribing', 0.42)).toContain('aria-valuenow="42"');
  });
});
