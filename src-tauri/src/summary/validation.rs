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
                    .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())))
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

/// Numeric identity is unchanged by leading zeroes; never infer a missing source.
pub fn canonical_references(markdown: &str, labels: &BTreeSet<&str>) -> String {
    let mut result = markdown.to_owned();
    for label in note_labels(markdown) {
        let Some(digits) = label.strip_prefix('S') else {
            continue;
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let Some(index) = digits.parse::<usize>().ok().and_then(|n| n.checked_sub(1)) else {
            continue;
        };
        let canonical = chunk::source_id(index);
        if labels.contains(canonical.as_str()) {
            result = result.replace(&format!("[{label}]"), &format!("[{canonical}]"));
        }
    }
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
