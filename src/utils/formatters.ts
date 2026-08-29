/**
 * Formats seconds into HH:MM:SS or MM:SS
 */
export function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds) || seconds < 0) {
    return '00:00';
  }

  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);

  const mm = m.toString().padStart(2, '0');
  const ss = s.toString().padStart(2, '0');

  if (h > 0) {
    const hh = h.toString().padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  }

  return `${mm}:${ss}`;
}

/**
 * Formats a track label for UI display
 */
export function formatTrackLabel(track: {
  id: number;
  title: string | null;
  lang: string | null;
  codec: string | null;
  default: boolean;
}): string {
  const parts: string[] = [];

  if (track.title) {
    parts.push(track.title);
  } else if (track.lang) {
    parts.push(track.lang.toUpperCase());
  } else {
    parts.push(`Track ${track.id}`);
  }

  if (track.codec) {
    parts.push(`(${track.codec.toUpperCase()})`);
  }

  if (track.default) {
    parts.push('[Default]');
  }

  return parts.join(' ');
}
