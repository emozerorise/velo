//! Reject unsupported labels rather than guessing where a claim belongs.
use super::chunk;
use crate::transcript::TranscriptSegment;
use std::collections::{BTreeMap, BTreeSet};

#[derive(serde::Serialize)]
pub struct Block {
    pub id: usize,
    pub section: String,
    pub text: String,
}

// These are section labels, not a heuristic that exempts arbitrary short claims.
fn organizational_label(line: &str) -> Option<&str> {
    let text = line
        .trim_start_matches(['-', '*', '+', ' '])
        .trim_end_matches(['*', ':', ' ', '：']);
    let text = text.trim_end_matches(':');
    let known = [
        "ภาพรวม",
        "ประเด็นที่คุยกัน",
        "ประเด็นสำคัญ",
        "สิ่งที่ตัดสินใจ",
        "งานที่ต้องทำ",
        "เรื่องที่ยังค้าง",
        "งานที่ตกลงแล้ว",
        "ข้อเสนอ",
        "ข้อเสนอแนะ",
        "Overview",
        "Topics discussed",
        "Key points",
        "Decisions",
        "Action items",
        "Open questions",
        "Agreed tasks",
        "Proposals",
        "Suggestions",
    ];
    known
        .iter()
        .any(|name| text.eq_ignore_ascii_case(name))
        .then_some(text)
}

/// Share the exact units used for citation coverage and semantic auditing.
pub fn blocks(markdown: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut headings: Vec<(usize, &str)> = Vec::new();
    let mut text = String::new();
    for line in markdown.lines().chain(std::iter::once("")) {
        let line = line.trim();
        let bullet = line.starts_with(['-', '*', '+'])
            || line
                .split_once(". ")
                .is_some_and(|(n, _)| n.parse::<u32>().is_ok());
        let label = organizational_label(line);
        let heading = line.starts_with('#') || label.is_some();
        if (line.is_empty() || heading || bullet) && !text.is_empty() {
            result.push(Block {
                id: result.len() + 1,
                section: headings
                    .iter()
                    .map(|(_, name)| *name)
                    .collect::<Vec<_>>()
                    .join(" / "),
                text: text.trim().to_owned(),
            });
            text.clear();
        }
        if heading {
            let level = if label.is_some() {
                3
            } else {
                line.bytes().take_while(|b| *b == b'#').count()
            };
            while headings
                .last()
                .is_some_and(|(previous, _)| *previous >= level)
            {
                headings.pop();
            }
            headings.push((
                level,
                label.unwrap_or_else(|| line.trim_start_matches('#').trim()),
            ));
        } else if !line.is_empty() && line != "-" {
            text.push_str(line);
            text.push('\n');
        }
    }
    result
}

#[cfg(test)]
pub fn source_labels(source: &str) -> BTreeSet<&str> {
    source
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix('[')?;
            let (label, _) = rest.split_once(']')?;
            Some(label)
        })
        .collect()
}

pub fn note_labels(notes: &str) -> BTreeSet<&str> {
    notes
        .split('[')
        .skip(1)
        .filter_map(|rest| {
            let (label, _) = rest.split_once(']')?;
            (label.contains(':')
                || label
                    .strip_prefix('S')
                    .is_some_and(|id| id.as_bytes().first().is_some_and(u8::is_ascii_digit)))
            .then_some(label)
        })
        .collect()
}

pub fn validate(markdown: &str, labels: &BTreeSet<&str>) -> std::result::Result<(), String> {
    if markdown.trim().is_empty() || markdown.contains("[[UNSUPPORTED]]") {
        return Err("no supported summary returned".into());
    }
    for label in note_labels(markdown) {
        if !labels.contains(label) {
            return Err(format!("unsupported citation [{label}]"));
        }
    }
    let blocks = blocks(markdown);
    for block in &blocks {
        if note_labels(&block.text).is_empty() {
            return Err(format!(
                "a factual block has no source citation: {}",
                block.text.chars().take(160).collect::<String>()
            ));
        }
    }
    if blocks.is_empty() {
        return Err("no supported summary returned".into());
    }
    Ok(())
}

/// Numeric identity and complete supplied ranges are exact, not guessed sources.
pub fn canonical_references(markdown: &str, labels: &BTreeSet<&str>) -> String {
    let mut result = markdown.to_owned();
    for label in note_labels(markdown) {
        let number = |part: &str| -> Option<usize> {
            let digits = part.trim().strip_prefix('S')?;
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            digits.parse().ok().filter(|n| *n > 0)
        };
        let mut parts = label.split(['-', '–', '—']);
        let Some(start) = parts.next().and_then(number) else {
            continue;
        };
        let end = match parts.next() {
            Some(part) => match number(part) {
                Some(end) if parts.next().is_none() => end,
                _ => continue,
            },
            None => start,
        };
        // Bound allocation before iterating a model-supplied numeric range.
        let Some(count) = end.checked_sub(start).and_then(|n| n.checked_add(1)) else {
            continue;
        };
        if count > labels.len() {
            continue;
        }
        let canonical: Vec<_> = (start..=end).map(|n| chunk::source_id(n - 1)).collect();
        if canonical.iter().all(|id| labels.contains(id.as_str())) {
            let expanded = canonical
                .iter()
                .map(|id| format!("[{id}]"))
                .collect::<Vec<_>>()
                .join(" ");
            result = result.replace(&format!("[{label}]"), &expanded);
        }
    }
    result
}

/// Wire-only compression of adjacent canonical IDs; no prose or source is lost.
#[cfg(test)]
pub fn compact_references(markdown: &str) -> String {
    fn id(text: &str) -> Option<(usize, &str)> {
        let (label, rest) = text.strip_prefix('[')?.split_once(']')?;
        let n = label.strip_prefix('S')?.parse::<usize>().ok()?;
        (n > 0 && chunk::source_id(n - 1) == label).then_some((n, rest))
    }
    let mut result = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(index) = rest.find('[') {
        result.push_str(&rest[..index]);
        rest = &rest[index..];
        let Some((start, after)) = id(rest) else {
            result.push('[');
            rest = &rest[1..];
            continue;
        };
        let mut end = start;
        let mut remaining = after;
        loop {
            let next = remaining.trim_start_matches([' ', '\t']);
            match id(next) {
                Some((n, after)) if end.checked_add(1) == Some(n) => {
                    end = n;
                    remaining = after;
                }
                _ => break,
            }
        }
        if end > start {
            result.push_str(&format!(
                "[{}-{}]",
                chunk::source_id(start - 1),
                chunk::source_id(end - 1)
            ));
        } else {
            result.push_str(&rest[..rest.len() - after.len()]);
        }
        rest = remaining;
    }
    result.push_str(rest);
    result
}

/// Lookup, not rounding or similarity: only an existing source ID can become a clock label.
pub fn render_references(markdown: &str, segments: &[TranscriptSegment]) -> String {
    let times: BTreeMap<_, _> = segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (chunk::source_id(index), chunk::stamp(segment.start)))
        .collect();
    let mut rendered = String::new();
    let mut rest = markdown;
    while let Some((before, after)) = rest.split_once('[') {
        rendered.push_str(before);
        let Some((label, remaining)) = after.split_once(']') else {
            rendered.push('[');
            rendered.push_str(after);
            return rendered;
        };
        rendered.push('[');
        rendered.push_str(times.get(label).map(String::as_str).unwrap_or(label));
        rendered.push(']');
        rest = remaining;
    }
    rendered.push_str(rest);
    rendered
}

pub(crate) fn source_clock(id: &str, segments: &[TranscriptSegment]) -> Option<String> {
    let index = id
        .strip_prefix('S')?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)?;
    if chunk::source_id(index) != id {
        return None;
    }
    Some(chunk::stamp(segments.get(index)?.start))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_compaction_preserves_claim_boundaries_order_and_every_reference() {
        let labels = BTreeSet::from(["S000001", "S000002", "S000003", "S000004"]);
        let raw = "First [S000001] [S000002] [S000003]\n\nOther [S000004]\nThen [S000002] [S000001] [S000001]";
        let compact = compact_references(raw);
        assert_eq!(
            compact,
            "First [S000001-S000003]\n\nOther [S000004]\nThen [S000002] [S000001] [S000001]"
        );
        assert_eq!(canonical_references(&compact, &labels), raw);
        assert_eq!(
            compact_references("First [S000001]\n[S000002] words [S000003]"),
            "First [S000001]\n[S000002] words [S000003]"
        );
        assert_eq!(
            compact_references("[S000001] [S999999] [00:37] [S0] [S000001-S999999]"),
            "[S000001] [S999999] [00:37] [S0] [S000001-S999999]"
        );
    }
    #[test]
    fn source_ranges_expand_only_when_every_id_was_supplied() {
        let labels = BTreeSet::from(["S001839", "S001840", "S001841", "S001842", "S001843"]);
        for range in ["S001839-S001843", "S1839 – S1843", "S1839—S1843"] {
            let result = canonical_references(&format!("ประเด็น [{range}]"), &labels);
            assert_eq!(
                result,
                "ประเด็น [S001839] [S001840] [S001841] [S001842] [S001843]"
            );
            assert!(validate(&result, &labels).is_ok());
        }
        let gaps = BTreeSet::from(["S001839", "S001843"]);
        for text in [
            "ประเด็น [S001839-S001843]",
            "ประเด็น [S001843-S001839]",
            "ประเด็น [S001839-S99999999999999999999999999]",
            "ประเด็น [S001839-S001843] [S001839]",
            "ประเด็น [S001839-] [S001839]",
            "ประเด็น [S001839-S001840-S001843] [S001839]",
            "ประเด็น [S0-S001843] [S001839]",
        ] {
            let result = canonical_references(text, &gaps);
            assert!(validate(&result, &gaps).is_err(), "{text}");
        }
    }

    #[test]
    fn organizational_labels_need_no_fabricated_reference_but_claims_still_do() {
        let labels = BTreeSet::from(["S000001"]);
        for heading in [
            "- งานที่ตกลงแล้ว",
            "**ข้อเสนอ**",
            "- **Action items:**",
            "ประเด็นสำคัญ:",
        ] {
            assert!(
                validate(&format!("{heading}\n  - Actual point [S000001]"), &labels).is_ok(),
                "{heading}"
            );
            assert!(
                validate(heading, &labels).is_err(),
                "a heading alone is not a summary"
            );
        }
        for prose in [
            "- Agreed to release tomorrow",
            "**ผู้รับผิดชอบคือสมชาย**",
            "งานที่ตกลงแล้ว: เพิ่มระบบใหม่",
            "- ผู้รับผิดชอบ: ไม่ระบุ",
        ] {
            assert!(
                validate(&format!("{prose}\n- Another point [S000001]"), &labels).is_err(),
                "{prose}"
            );
        }
        assert!(validate("- งานที่ตกลงแล้ว\n  - Invalid [S000099]", &labels).is_err());
    }
    #[test]
    fn only_existing_ids_resolve_to_their_own_source_time() {
        let segments = vec![
            TranscriptSegment {
                start: 30.9,
                end: 37.38,
                text: "Source".into(),
            },
            TranscriptSegment {
                start: 37.38,
                end: 42.78,
                text: "More".into(),
            },
        ];
        let labels = source_labels("[S000001] Source\n[S000002] More");
        let reviewed = "Point [S000002]\n\n- Other [S000001]";
        assert!(validate(reviewed, &labels).is_ok());
        assert_eq!(
            render_references(reviewed, &segments),
            "Point [00:37]\n\n- Other [00:30]"
        );
        for invalid in [
            "Point [S000099]",
            "Point [00:31]",
            "Point [S000001] [S000099]",
        ] {
            assert!(validate(invalid, &labels).is_err(), "{invalid}");
        }
    }
    #[test]
    fn checks_every_block_and_only_exact_supplied_labels() {
        let labels = source_labels("[00:30] Source\n[1:02:03] More");
        assert!(validate(
            "## Summary\nSupported [00:30]\ncontinued\n\n- More [1:02:03]",
            &labels
        )
        .is_ok());
        for text in [
            "Wrong [00:31]",
            "Rounded [00:37]",
            "Invalid [00:60]",
            "Alias [0:00:30]",
            "Range [00:30–00:37]",
            "## Empty",
            "## Head\nUncited\n\n- Cited [00:30]",
            "[[UNSUPPORTED]]",
        ] {
            assert!(validate(text, &labels).is_err(), "{text}");
        }
    }
}
