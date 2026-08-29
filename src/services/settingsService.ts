import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '@/types/settings';

export interface HistoryItem {
  path: string;
  file_name: string;
  last_position: number;
  duration: number;
  last_played_timestamp: number;
}

export interface HistoryData {
  recent_files: HistoryItem[];
  resume_positions: Record<string, number>;
}

export const settingsService = {
  async getAll(): Promise<AppSettings> {
    return invoke('settings_get_all');
  },

  async save(settings: AppSettings): Promise<void> {
    return invoke('settings_save', { settings });
  },

  async getHistory(): Promise<HistoryData> {
    return invoke('history_get');
  },

  async recordHistory(item: HistoryItem): Promise<void> {
    return invoke('history_record', { item });
  },
};
