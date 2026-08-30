import { describe, it, expect } from 'vitest';
import { resumePositionFor } from '@/composables/usePlaybackHistory';

const HOUR = 3600;

describe('resumePositionFor', () => {
  it('remembers a position in the middle of a file', () => {
    expect(resumePositionFor(1200, HOUR)).toBe(1200);
  });

  it('ignores the first few seconds, which are not worth returning to', () => {
    expect(resumePositionFor(3, HOUR)).toBe(0);
    expect(resumePositionFor(14.9, HOUR)).toBe(0);
    expect(resumePositionFor(15, HOUR)).toBe(15);
  });

  it('starts over when playback reached the end', () => {
    // Anything inside the closing margin counts as finished: resuming into
    // the last seconds is worse than replaying from the start.
    expect(resumePositionFor(HOUR, HOUR)).toBe(0);
    expect(resumePositionFor(HOUR - 1, HOUR)).toBe(0);
    expect(resumePositionFor(HOUR - 30, HOUR)).toBe(0);
    expect(resumePositionFor(HOUR - 31, HOUR)).toBe(HOUR - 31);
  });

  it('does not resume when the duration is unknown', () => {
    // Streams and files still loading report zero, which would otherwise make
    // every position look like it was past the end.
    expect(resumePositionFor(500, 0)).toBe(0);
    expect(resumePositionFor(500, -1)).toBe(0);
  });

  it('treats a short clip as finished rather than resumable', () => {
    // A 20 second clip is entirely inside the closing margin.
    expect(resumePositionFor(18, 20)).toBe(0);
  });
});
