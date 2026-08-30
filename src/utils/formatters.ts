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

/**
 * Formats a byte count for download progress. Whole numbers below 10 MB would
 * be too coarse to look like progress, so small sizes keep a decimal.
 */
export function formatBytes(bytes: number): string {
  if (!bytes || isNaN(bytes) || bytes < 0) {
    return '0 MB';
  }

  const mb = bytes / (1024 * 1024);
  if (mb < 1) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  if (mb < 10) {
    return `${mb.toFixed(1)} MB`;
  }
  return `${Math.round(mb)} MB`;
}
