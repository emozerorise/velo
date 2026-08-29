import { describe, it, expect } from 'vitest';
import { formatTime, formatTrackLabel } from '@/utils/formatters';

describe('formatTime', () => {
  it('formats 0 seconds as 00:00', () => {
    expect(formatTime(0)).toBe('00:00');
  });

  it('formats under a minute correctly', () => {
    expect(formatTime(45)).toBe('00:45');
  });

  it('formats minutes and seconds correctly', () => {
    expect(formatTime(125)).toBe('02:05');
  });

  it('formats hours, minutes, and seconds correctly', () => {
    expect(formatTime(3665)).toBe('01:01:05');
  });

  it('handles negative or invalid values gracefully', () => {
    expect(formatTime(-10)).toBe('00:00');
    expect(formatTime(NaN)).toBe('00:00');
  });
});

describe('formatTrackLabel', () => {
  it('formats track with title and codec', () => {
    const track = {
      id: 1,
      title: 'English Commentary',
      lang: 'eng',
      codec: 'aac',
      default: true,
    };
    expect(formatTrackLabel(track)).toBe('English Commentary (AAC) [Default]');
  });

  it('falls back to language if no title', () => {
    const track = {
      id: 2,
      title: null,
      lang: 'jpn',
      codec: 'flac',
      default: false,
    };
    expect(formatTrackLabel(track)).toBe('JPN (FLAC)');
  });

  it('falls back to track id if no title or language', () => {
    const track = {
      id: 3,
      title: null,
      lang: null,
      codec: null,
      default: false,
    };
    expect(formatTrackLabel(track)).toBe('Track 3');
  });
});
