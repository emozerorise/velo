//! Splitting a transcript into pieces a model can hold at once.

use crate::transcript::TranscriptSegment;

/// Deliberately pessimistic. Thai runs close to two characters per token in
/// the tokenizers involved; English is cheaper, so this over-reserves rather
/// than overflowing the window.
const CHARS_PER_TOKEN: usize = 2;

/// Half the window is left for the prompt and the answer.
const INPUT_SHARE: f64 = 0.5;

/// Segments repeated at the head of the next chunk, so a topic that spans a
/// boundary is summarised with its lead-in rather than starting mid-thought.
const OVERLAP_SEGMENTS: usize = 2;

/// A window big enough to be worth a request even if the setting is tiny.
const MIN_BUDGET_CHARS: usize = 2_000;

pub struct Chunk {
    /// `[mm:ss] text` lines, ready to hand to the model.
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub segments: usize,
}

/// `[mm:ss]`, widening to `[h:mm:ss]` past an hour so a long meeting's
/// citations stay unambiguous.
pub fn stamp(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

pub fn budget_chars(context_tokens: u32) -> usize {
    let budget = (context_tokens as f64 * INPUT_SHARE) as usize * CHARS_PER_TOKEN;
    budget.max(MIN_BUDGET_CHARS)
}

/// Split on segment boundaries only -- never mid-sentence -- filling each
/// chunk up to `budget` characters.
pub fn chunk(segments: &[TranscriptSegment], budget: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut i = 0;

    while i < segments.len() {
        let mut text = String::new();
        let mut j = i;

        while j < segments.len() {
            let segment = &segments[j];
            let line = format!("[{}] {}\n", stamp(segment.start), segment.text.trim());

            // A single segment longer than the budget still gets its own
            // chunk: dropping it would silently lose transcript.
            if !text.is_empty() && text.len() + line.len() > budget {
                break;
            }

            text.push_str(&line);
            j += 1;
        }

        chunks.push(Chunk {
            text,
            start: segments[i].start,
            end: segments[j - 1].end,
            segments: j - i,
        });

        if j >= segments.len() {
            break;
        }

        // Always advance, even when the overlap would reach back past the
        // start of the chunk just emitted.
        i = j.saturating_sub(OVERLAP_SEGMENTS).max(i + 1);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments(count: usize, text: &str) -> Vec<TranscriptSegment> {
        (0..count)
            .map(|i| TranscriptSegment {
                start: i as f64 * 10.0,
                end: i as f64 * 10.0 + 9.0,
                text: text.to_string(),
            })
            .collect()
    }

    #[test]
    fn stamps_minutes_then_hours() {
        assert_eq!(stamp(0.0), "00:00");
        assert_eq!(stamp(72.4), "01:12");
        assert_eq!(stamp(3_723.0), "1:02:03");
    }

    #[test]
    fn one_chunk_when_everything_fits() {
        let chunks = chunk(&segments(5, "hello"), 10_000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].segments, 5);
        assert_eq!(chunks[0].start, 0.0);
        assert_eq!(chunks[0].end, 49.0);
    }

    #[test]
    fn respects_the_budget_and_overlaps() {
        let chunks = chunk(&segments(20, "a fairly long line of speech"), 200);
        assert!(chunks.len() > 1, "expected a split");

        for c in &chunks {
            // Only a chunk holding exactly one segment may exceed the budget.
            assert!(c.text.len() <= 200 || c.segments == 1, "chunk overflowed");
        }

        // Consecutive chunks share their boundary segments.
        assert!(chunks[1].start < chunks[0].end, "chunks did not overlap");
    }

    #[test]
    fn a_segment_larger_than_the_budget_still_ships() {
        let long = "x".repeat(500);
        let chunks = chunk(&segments(3, &long), 100);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.segments == 1));
    }

    #[test]
    fn budget_never_collapses_to_nothing() {
        assert_eq!(budget_chars(0), MIN_BUDGET_CHARS);
        assert_eq!(budget_chars(32_768), 32_768);
    }
}
