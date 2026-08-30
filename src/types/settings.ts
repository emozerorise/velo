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

export interface AppSettings {
  version: number;
  general: GeneralSettings;
  video: VideoSettings;
  audio: AudioSettings;
  subtitle: SubtitleSettings;
  transcript: TranscriptSettings;
}
