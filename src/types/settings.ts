export interface GeneralSettings {
  theme: 'dark' | 'light' | 'system';
  language: string;
  remember_playback_position: boolean;
  auto_play_next: boolean;
}

export interface VideoSettings {
  hardware_acceleration: boolean;
  default_aspect_ratio: string;
}

export interface AudioSettings {
  default_volume: number;
  preferred_language: string;
  volume_step: number;
  audio_delay_step: number;
}

export interface SubtitleSettings {
  preferred_language: string;
  auto_load_external: boolean;
  font_size: number;
  subtitle_delay_step: number;
}

export interface TranscriptSettings {
  language: string;
  /** Domain vocabulary fed to whisper as its initial prompt. */
  prompt: string;
}

export interface SummarySettings {
  /** 'ollama' | 'openai' — picks the transport, never inferred from the URL. */
  provider: string;
  base_url: string;
  model: string;
  /** 'th' | 'en' | 'auto' — the summary's language, not the app's. */
  language: string;
  instructions: string;
  /** Drives chunk sizing, and is sent as num_ctx on the Ollama transport. */
  context_tokens: number;
}

export interface AppSettings {
  version: number;
  general: GeneralSettings;
  video: VideoSettings;
  audio: AudioSettings;
  subtitle: SubtitleSettings;
  transcript: TranscriptSettings;
  summary: SummarySettings;
}
