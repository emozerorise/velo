import { invoke } from '@tauri-apps/api/core';
import type { Summary, SummaryStatus } from '@/types/summary';

export const summaryService = {
  async status(): Promise<SummaryStatus> {
    return invoke('summary_status');
  },

  /** The models the provider actually has. Doubles as a reachability check. */
  async probe(): Promise<string[]> {
    return invoke('summary_probe');
  },

  async setApiKey(key: string): Promise<void> {
    return invoke('summary_set_api_key', { key });
  },

  async clearApiKey(): Promise<void> {
    return invoke('summary_clear_api_key');
  },

  async get(path: string): Promise<Summary | null> {
    return invoke('summary_get', { path });
  },

  async generate(path: string): Promise<void> {
    return invoke('summary_generate', { path });
  },

  async cancel(): Promise<void> {
    return invoke('summary_cancel');
  },

  async remove(path: string): Promise<void> {
    return invoke('summary_delete', { path });
  },
};
