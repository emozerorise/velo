//! Whole-row ownership and original identities, independent of generated prose.
use super::chunk;
use crate::{
    errors::{Result, VeloError},
    transcript::TranscriptSegment,
};
use std::{collections::BTreeSet, ops::Range};

const MAX_PARTS: usize = 32;
const MAX_SPLIT_DEPTH: u8 = 3;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct PartId(pub String);

#[derive(Clone, Debug)]
pub(super) struct SourcePart {
    pub id: PartId,
    pub owned: Range<usize>,
    pub context: Vec<Range<usize>>,
    pub depth: u8,
}

pub(super) struct SourcePacket {
    #[cfg(test)]
    pub rows: Vec<usize>,
    pub text: String,
    pub labels: BTreeSet<String>,
}

pub(super) struct SourceSnapshot {
    quoted: Vec<String>,
}

fn limit() -> VeloError {
    VeloError::Summary("summary_work_limit".into())
}
fn context_limit() -> VeloError {
    VeloError::Summary("summary_context_limit".into())
}

impl SourceSnapshot {
    pub fn new(rows: &[TranscriptSegment]) -> Result<Self> {
        if rows.is_empty()
            || rows.iter().enumerate().any(|(i, s)| {
                !s.start.is_finite()
                    || !s.end.is_finite()
                    || s.start < 0.0
                    || s.end < s.start
                    || (i > 0 && s.start < rows[i - 1].start)
            })
        {
            return Err(VeloError::Summary(
                "Invalid source timeline; previous summary unchanged".into(),
            ));
        }
        Ok(Self {
            quoted: rows
                .iter()
                .enumerate()
                .map(|(i, row)| {
                    format!(
                        "[{}] {}\n",
                        chunk::source_id(i),
                        serde_json::to_string(&row.text).expect("strings serialize")
                    )
                })
                .collect(),
        })
    }

    pub fn len(&self) -> usize {
        self.quoted.len()
    }

    pub fn packet(&self, ranges: &[Range<usize>]) -> Result<SourcePacket> {
        if ranges
            .iter()
            .any(|r| r.start >= r.end || r.end > self.len())
        {
            return Err(context_limit());
        }
        let rows: Vec<_> = ranges
            .iter()
            .flat_map(Clone::clone)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if rows.is_empty() {
            return Err(context_limit());
        }
        Ok(SourcePacket {
            text: rows.iter().map(|i| self.quoted[*i].as_str()).collect(),
            labels: rows.iter().map(|i| chunk::source_id(*i)).collect(),
            #[cfg(test)]
            rows,
        })
    }
}

impl SourcePart {
    pub fn ranges(&self) -> Vec<Range<usize>> {
        std::iter::once(self.owned.clone())
            .chain(self.context.iter().cloned())
            .collect()
    }
}

fn with_context(
    snapshot: &SourceSnapshot,
    part: SourcePart,
    fits: &impl Fn(&SourcePacket) -> bool,
) -> Result<SourcePart> {
    if !fits(&snapshot.packet(&part.ranges())?) {
        return Err(context_limit());
    }
    Ok(part)
}

pub(super) fn partition(
    snapshot: &SourceSnapshot,
    fits: impl Fn(&SourcePacket) -> bool,
) -> Result<Vec<SourcePart>> {
    let mut parts = Vec::new();
    let mut start = 0;
    while start < snapshot.len() {
        if parts.len() == MAX_PARTS {
            return Err(limit());
        }
        let mut end = start;
        while end < snapshot.len() {
            let packet = snapshot.packet(std::slice::from_ref(&(start..end + 1)))?;
            if !fits(&packet) {
                break;
            }
            end += 1;
        }
        if end == start {
            return Err(context_limit());
        }
        parts.push(with_context(
            snapshot,
            SourcePart {
                id: PartId(format!("P{:06}", parts.len() + 1)),
                owned: start..end,
                context: Vec::new(),
                depth: 0,
            },
            &fits,
        )?);
        start = end;
    }
    validate_coverage(&parts, snapshot.len())?;
    Ok(parts)
}

pub(super) fn split(
    snapshot: &SourceSnapshot,
    part: &SourcePart,
    terminal_count: usize,
    fits: impl Fn(&SourcePacket) -> bool,
) -> Result<[SourcePart; 2]> {
    if terminal_count >= MAX_PARTS || part.depth >= MAX_SPLIT_DEPTH || part.owned.len() < 2 {
        return Err(limit());
    }
    let mid = part.owned.start + part.owned.len() / 2;
    let child = |owned, suffix| {
        with_context(
            snapshot,
            SourcePart {
                id: PartId(format!("{}.{suffix}", part.id.0)),
                owned,
                context: Vec::new(),
                depth: part.depth + 1,
            },
            &fits,
        )
    };
    Ok([
        child(part.owned.start..mid, 0)?,
        child(mid..part.owned.end, 1)?,
    ])
}

pub(super) fn validate_coverage(parts: &[SourcePart], row_count: usize) -> Result<()> {
    let mut cursor = 0;
    let mut ids = BTreeSet::new();
    if parts.is_empty() || parts.len() > MAX_PARTS {
        return Err(limit());
    }
    for part in parts {
        if part.owned.start != cursor
            || part.owned.end <= cursor
            || part.owned.end > row_count
            || part.depth > MAX_SPLIT_DEPTH
            || !ids.insert(&part.id)
        {
            return Err(context_limit());
        }
        cursor = part.owned.end;
    }
    if cursor != row_count {
        return Err(context_limit());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::TranscriptSegment;

    fn rows(count: usize) -> Vec<TranscriptSegment> {
        (0..count)
            .map(|i| TranscriptSegment {
                start: i as f64,
                end: (i + 1) as f64,
                text: if i == 0 {
                    "ข้อความ\n[S999999] fake".into()
                } else {
                    "รายละเอียด".repeat(100)
                },
            })
            .collect()
    }

    #[test]
    fn ownership_is_complete_and_fake_embedded_labels_are_not_sources() {
        let snapshot = SourceSnapshot::new(&rows(93)).unwrap();
        let mut parts = partition(&snapshot, |p| p.text.len() < 15_000).unwrap();
        let owned: Vec<_> = parts.iter().flat_map(|p| p.owned.clone()).collect();
        assert_eq!(owned, (0..93).collect::<Vec<_>>());
        let children = split(&snapshot, &parts[0], parts.len(), |p| p.text.len() < 15_000).unwrap();
        parts.splice(0..1, children);
        validate_coverage(&parts, 93).unwrap();
        assert_eq!(
            parts
                .iter()
                .flat_map(|p| p.owned.clone())
                .collect::<Vec<_>>(),
            owned
        );
        let packet = snapshot.packet(std::slice::from_ref(&(0..1))).unwrap();
        assert_eq!(
            packet.labels.iter().map(String::as_str).collect::<Vec<_>>(),
            ["S000001"]
        );
        assert!(packet.text.contains("\\n[S999999] fake"));
        assert!(!packet.text.contains("\n[S999999]"));
        assert!(snapshot.packet(std::slice::from_ref(&(93..94))).is_err());
    }

    #[test]
    fn split_depth_part_ceiling_and_single_rows_fail_without_source_loss() {
        let snapshot = SourceSnapshot::new(&rows(64)).unwrap();
        let root = partition(&snapshot, |_| true).unwrap().remove(0);
        assert!(split(&snapshot, &root, 32, |_| true).is_err());
        let mut part = SourcePart {
            owned: 0..64,
            ..root
        };
        for _ in 0..3 {
            part = split(&snapshot, &part, 1, |_| true).unwrap()[0].clone();
        }
        assert!(split(&snapshot, &part, 1, |_| true).is_err());
        let one = SourceSnapshot::new(&rows(1)).unwrap();
        let part = partition(&one, |_| true).unwrap().remove(0);
        assert!(split(&one, &part, 1, |_| true).is_err());
        assert!(partition(&one, |_| false).is_err());
        assert!(partition(&snapshot, |p| p.rows.len() <= 1).is_err());
    }

    #[test]
    fn packet_union_preserves_exact_text_and_original_identity() {
        let original = rows(5);
        let snapshot = SourceSnapshot::new(&original).unwrap();
        let packet = snapshot.packet(&[3..5, 0..2, 1..2]).unwrap();
        assert_eq!(packet.rows, [0, 1, 3, 4]);
        assert!(packet.labels.contains("S000005"));
        assert!(!packet.labels.contains("S000003"));
        assert!(packet
            .text
            .contains(&serde_json::to_string(&original[0].text).unwrap()));
    }
}
