export interface PlaylistItem {
  id: string;
  path: string;
  fileName: string;
  duration?: number;
  lastPosition?: number;
}

export type LoopMode = 'off' | 'all' | 'single';
