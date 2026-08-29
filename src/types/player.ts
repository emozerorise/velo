export type PlaybackState =
  | 'idle'
  | 'loading'
  | 'playing'
  | 'paused'
  | 'stopped'
  | 'ended'
  | 'error';

export interface Track {
  id: number;
  type: 'video' | 'audio' | 'sub';
  src_id: number;
  title: string | null;
  lang: string | null;
  codec: string | null;
  selected: boolean;
  default: boolean;
}

export interface MediaInfo {
  path: string;
  file_name: string;
  duration: number;
  width: number;
  height: number;
  video_codec: string | null;
  audio_codec: string | null;
  fps: number;
  hwdec_current: string | null;
}

export interface TimeUpdate {
  current_time: number;
  duration: number;
  percent: number;
}
