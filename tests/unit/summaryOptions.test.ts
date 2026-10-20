import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '@/stores/settingsStore';
import { summaryForCopy } from '@/utils/summaryMarkdown';
import SummaryPromptEditor from '@/components/transcript/SummaryPromptEditor.vue';
import { setLocale } from '@/composables/useI18n';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));

describe('summary reading options', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });
  it('copies without citations while preserving headings, nesting and non-time brackets', () => {
    const markdown =
      '## Decisions\n- Keep **owner** [00:05] [1:02:03]\n  - See [Document] at 12:34 [00:08]\n';
    expect(summaryForCopy(markdown, false)).toBe(
      '## Decisions\n- Keep **owner**\n  - See [Document] at 12:34\n',
    );
    expect(summaryForCopy(markdown, true)).toBe(markdown);
  });
  it('persists the hidden-time preference without changing provider or summary instructions', async () => {
    const store = useSettingsStore();
    store.settings.summary.instructions = 'Use the agreed owner';
    const originalSummary = { ...store.settings.summary };
    expect(await store.saveSummaryOptions({ showTimestamps: false })).toBe(true);
    expect(store.settings.general.show_summary_timestamps).toBe(false);
    expect(store.settings.summary).toEqual(originalSummary);
    expect(invoke).toHaveBeenCalledOnce();
    expect(invoke).toHaveBeenCalledWith('settings_save', { settings: store.settings });
  });
  it('saves a custom prompt without starting a summary request', async () => {
    const store = useSettingsStore();
    expect(await store.saveSummaryOptions({ instructions: 'Focus on owners and deadlines' })).toBe(
      true,
    );
    expect(store.settings.summary.instructions).toBe('Focus on owners and deadlines');
    expect(vi.mocked(invoke).mock.calls.map(([command]) => command)).toEqual(['settings_save']);
  });
  it('keeps the saved prompt and view preference when persistence fails', async () => {
    const store = useSettingsStore();
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Disk full'));
    expect(await store.saveSummaryOptions({ instructions: 'Unsaved', showTimestamps: false })).toBe(
      false,
    );
    expect(store.settings.summary.instructions).toBe('');
    expect(store.settings.general.show_summary_timestamps).toBe(true);
    expect(invoke).toHaveBeenCalledOnce();
  });
  it.each(['en', 'th'])(
    'shows an editable prompt, three examples and an explicit save (%s)',
    async (locale) => {
      setLocale(locale);
      useSettingsStore().settings.summary.instructions = '<script>Custom focus</script>';
      const context: { teleports?: Record<string, string> } = {};
      const html = await renderToString(
        createSSRApp(SummaryPromptEditor, { open: true }).use(getActivePinia()!),
        context,
      );
      const body = html + (context.teleports?.body ?? '');
      expect(body).toContain('<textarea');
      expect(body).toContain('&lt;script&gt;Custom focus&lt;/script&gt;');
      expect(body).not.toContain('<script>');
      expect(body).toContain(locale === 'th' ? 'สรุปผู้บริหาร' : 'Executive summary');
      expect(body).toContain(locale === 'th' ? 'เน้นงานที่ต้องทำ' : 'Action-focused');
      expect(body).toContain(locale === 'th' ? 'ประชุมทีมพัฒนา' : 'Development meeting');
      expect(body).toContain(locale === 'th' ? 'บันทึก prompt' : 'Save prompt');
      expect(invoke).not.toHaveBeenCalled();
    },
  );
});
