import { invoke } from '@tauri-apps/api/core';

export const playerService = {
  async loadFile(path: string, startTime?: number): Promise<void> {
    return invoke('player_load_file', { path, startTime });
  },

  async play(): Promise<void> {
    return invoke('player_play');
  },

  async pause(): Promise<void> {
    return invoke('player_pause');
  },

  async togglePause(): Promise<void> {
    return invoke('player_toggle_pause');
  },

  async stop(): Promise<void> {
    return invoke('player_stop');
  },

  async seek(seconds: number, exact = true): Promise<void> {
    return invoke('player_seek', { seconds, exact });
  },

  async seekAbsolute(seconds: number): Promise<void> {
    return invoke('player_seek_absolute', { seconds });
  },

  async setVolume(volume: number): Promise<void> {
    return invoke('player_set_volume', { volume });
  },

  async setMute(muted: boolean): Promise<void> {
    return invoke('player_set_mute', { muted });
  },

  async setSpeed(speed: number): Promise<void> {
    return invoke('player_set_speed', { speed });
  },

  async setAspectRatio(ratio: string): Promise<void> {
    return invoke('player_set_aspect_ratio', { ratio });
  },

  async selectAudioTrack(id: number): Promise<void> {
    return invoke('player_select_audio_track', { id });
  },

  async selectSubtitleTrack(id: number): Promise<void> {
    return invoke('player_select_subtitle_track', { id });
  },

  async addSubtitleFile(path: string): Promise<void> {
    return invoke('player_add_subtitle_file', { path });
  },

  async setSubtitleDelay(seconds: number): Promise<void> {
    return invoke('player_set_subtitle_delay', { seconds });
  },

  async setAudioDelay(seconds: number): Promise<void> {
    return invoke('player_set_audio_delay', { seconds });
  },
};
