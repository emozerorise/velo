import { invoke } from '@tauri-apps/api/core';
import type { Transcript, TranscriptEngineStatus } from '@/types/transcript';

export const transcriptService = {
  async engineStatus(): Promise<TranscriptEngineStatus> {
    return invoke('transcript_engine_status');
  },

  async get(path: string): Promise<Transcript | null> {
    return invoke('transcript_get', { path });
  },

  async generate(path: string, language: string, prompt: string): Promise<void> {
    return invoke('transcript_generate', { path, language, prompt });
  },

  async cancel(): Promise<void> {
    return invoke('transcript_cancel');
  },

  async remove(path: string): Promise<void> {
    return invoke('transcript_delete', { path });
  },
};
