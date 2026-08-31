//! Splitting a transcript into pieces a model can hold at once.

use crate::transcript::TranscriptSegment;

/// Budgets are measured in bytes, because that is what `str::len` counts and
/// a Thai character is three of them. Measured against a real Thai meeting:
/// 32 KB of `[mm:ss] text` lines came to roughly 8,600 tokens, so about 3.8
/// bytes per token. Three is the conservative floor -- English sits nearer
/// four -- and under-filling the window is the safe direction to be wrong in.
const BYTES_PER_TOKEN: usize = 3;

/// Held back for the instructions and the answer itself.
const RESERVED_TOKENS: u32 = 4_096;

/// Segments repeated at the head of the next chunk, so a topic that spans a
/// boundary is summarised with its lead-in rather than starting mid-thought.
const OVERLAP_SEGMENTS: usize = 2;

/// A window big enough to be worth a request even if the setting is tiny.
const MIN_BUDGET_BYTES: usize = 4_000;

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

pub fn budget_bytes(context_tokens: u32) -> usize {
    let input = context_tokens.saturating_sub(RESERVED_TOKENS) as usize;
    (input * BYTES_PER_TOKEN).max(MIN_BUDGET_BYTES)
}

/// Split on segment boundaries only -- never mid-sentence -- filling each
/// chunk up to `budget` bytes.
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
    fn budget_leaves_room_for_the_answer() {
        assert_eq!(budget_bytes(0), MIN_BUDGET_BYTES);
        assert_eq!(budget_bytes(32_768), (32_768 - 4_096) * 3);
    }

    #[test]
    fn a_thai_meeting_is_measured_in_bytes_not_characters() {
        // Three bytes per character, so a budget read as characters would
        // split this three times over.
        let thai = "ประชุมเรื่องกำหนดปล่อยรุ่น".repeat(4);
        assert!(
            thai.chars().count() * 3 <= thai.len() + 2,
            "expected 3-byte characters"
        );

        let chunks = chunk(&segments(40, &thai), budget_bytes(32_768));
        assert_eq!(chunks.len(), 1, "a short meeting should still be one pass");
    }
}
