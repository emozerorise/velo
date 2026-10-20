import type { SummarySettings } from '@/types/settings';

/** Apply only on a deliberate switch, never while loading saved settings. */
export function selectSummaryProvider(current: SummarySettings, provider: string): SummarySettings {
  if (provider === current.provider) return { ...current };

  if (provider === 'deepseek') {
    return {
      ...current,
      provider,
      base_url: 'https://api.deepseek.com',
      model: 'deepseek-flash',
    };
  }

  // A DeepSeek model cannot be carried back to a local/OpenAI server. Leave
  // model choices alone when switching the two pre-existing presets.
  return {
    ...current,
    provider,
    base_url: provider === 'openai' ? 'https://api.openai.com/v1' : 'http://localhost:11434',
    model:
      current.provider === 'deepseek'
        ? provider === 'openai'
          ? ''
          : 'qwen3:30b-instruct'
        : current.model,
  };
}
