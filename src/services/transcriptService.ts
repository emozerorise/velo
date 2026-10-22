import type { TranscriptionDevice } from '@/types/settings';
import { invoke } from '@tauri-apps/api/core';
import type { Transcript, TranscriptEngineStatus } from '@/types/transcript';

export const transcriptService = {
  async engineStatus(): Promise<TranscriptEngineStatus> {
    return invoke('transcript_engine_status');
  },

  async downloadModel(): Promise<void> {
    return invoke('transcript_download_model');
  },

  async cancelDownload(): Promise<void> {
    return invoke('transcript_cancel_download');
  },

  /** Resolves with the number of bytes freed. */
  async removeModel(): Promise<number> {
    return invoke('transcript_remove_model');
  },

  async get(path: string): Promise<Transcript | null> {
    return invoke('transcript_get', { path });
  },

  async generate(path: string, language: string, prompt: string, device: TranscriptionDevice = 'auto'): Promise<void> {
    return invoke('transcript_generate', { path, language, prompt, device });
  },

  async cancel(): Promise<void> {
    return invoke('transcript_cancel');
  },

  async remove(path: string): Promise<void> {
    return invoke('transcript_delete', { path });
  },
};
