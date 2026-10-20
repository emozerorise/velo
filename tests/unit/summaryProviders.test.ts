import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, getActivePinia, setActivePinia } from 'pinia';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import SettingsModal from '@/components/settings/SettingsModal.vue';
import SummaryTab from '@/components/transcript/SummaryTab.vue';
import { useSummaryStore } from '@/stores/summaryStore';
import { usePlayerStore } from '@/stores/playerStore';
import { setLocale } from '@/composables/useI18n';
import { useSettingsStore } from '@/stores/settingsStore';
import { selectSummaryProvider } from '@/utils/summaryProviders';
import { providerHost, providerIsRemote } from '@/utils/providerLocation';

describe('summary provider presets', () => {
  beforeEach(() => setActivePinia(createPinia()));

  it('selects DeepSeek without carrying a local model to its API', () => {
    const current = useSettingsStore().settings.summary;
    current.language = 'th';
    current.instructions = 'Keep action items first';
    const selected = selectSummaryProvider(current, 'deepseek');
    expect(selected).toEqual({
      provider: 'deepseek',
      base_url: 'https://api.deepseek.com',
      model: 'deepseek-flash',
      context_tokens: 32768,
      language: 'th',
      instructions: 'Keep action items first',
    });
    expect(current.provider).toBe('ollama');
    expect(providerIsRemote(selected.base_url)).toBe(true);
    expect(providerHost(selected.base_url)).toBe('api.deepseek.com');
  });

  it('does not reset a saved DeepSeek address or model on reselect', () => {
    const current = {
      ...useSettingsStore().settings.summary,
      provider: 'deepseek',
      base_url: 'https://api.deepseek.com/v1',
      model: 'deepseek-v4-pro',
    };
    expect(selectSummaryProvider(current, 'deepseek')).toEqual(current);
  });

  it('restores a usable local model when leaving DeepSeek', () => {
    const current = {
      ...useSettingsStore().settings.summary,
      provider: 'deepseek',
      base_url: 'https://api.deepseek.com',
      model: 'deepseek-flash',
    };
    expect(selectSummaryProvider(current, 'ollama')).toMatchObject({
      provider: 'ollama',
      base_url: 'http://localhost:11434',
      model: 'qwen3:30b-instruct',
    });
    expect(selectSummaryProvider(current, 'openai')).toMatchObject({
      provider: 'openai',
      base_url: 'https://api.openai.com/v1',
      model: '',
    });
  });

  it('preserves the existing model when switching the legacy presets', () => {
    const current = {
      ...useSettingsStore().settings.summary,
      model: 'custom-model',
    };
    expect(selectSummaryProvider(current, 'openai')).toMatchObject({
      provider: 'openai',
      base_url: 'https://api.openai.com/v1',
      model: 'custom-model',
    });
  });

  it.each(['en', 'th'])(
    'offers DeepSeek and shows its destination before a request (%s)',
    async (locale) => {
      setLocale(locale);
      const store = useSettingsStore();
      store.openSettings('ai');
      store.settings.summary = {
        ...store.settings.summary,
        provider: 'deepseek',
        base_url: 'https://api.deepseek.com',
        model: 'deepseek-flash',
      };
      const context: { teleports?: Record<string, string> } = {};
      await renderToString(createSSRApp(SettingsModal).use(getActivePinia()!), context);
      const html = context.teleports?.body ?? '';
      expect(html).toContain('value="deepseek" selected>DeepSeek</option>');
      expect(html).toContain(
        locale === 'th'
          ? 'บทถอดเสียงจะถูกส่งไปที่ api.deepseek.com'
          : 'Transcripts are sent to api.deepseek.com.',
      );
      expect(html).toContain('type="password"');
    },
  );

  it('does not tell a DeepSeek user to install or pull an Ollama model', async () => {
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
    store.status = {
      provider: 'deepseek',
      base_url: 'https://api.deepseek.com',
      model: '',
      configured: false,
      remote: true,
      has_key: false,
    };
    const app = () => createSSRApp(SummaryTab).use(getActivePinia()!);
    const setup = await renderToString(app());
    expect(setup).not.toContain('ollama pull');
    expect(setup).not.toContain('a model you run yourself');

    store.status.configured = true;
    store.status.model = 'unknown-model';
    store.error = 'The model "unknown-model" is not available on this server';
    const missingModel = await renderToString(app());
    expect(missingModel).not.toContain('ollama pull');
    expect(missingModel).toContain('The provider does not offer unknown-model.');
    store.error = 'Could not reach the model server';
    const unreachable = await renderToString(app());
    expect(unreachable).not.toContain('ollama serve');
    expect(unreachable).toContain('Check your internet connection');
  });
});
