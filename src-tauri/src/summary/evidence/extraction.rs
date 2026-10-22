use super::{
    source::{self, SourceSnapshot},
    types::{Anchor, DraftPoint},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
#[serde(tag = "category", rename_all = "snake_case")]
enum Fault {
    Schema,
    JobMismatch,
    PointLimitExceeded { actual: usize, limit: usize },
    EmptySources { point: usize },
    InvalidSource { point: usize, source: usize },
    SourceOrder { point: usize, source: usize },
    ReusedSource { point: usize, source: usize },
    EmptyText { point: usize, source: usize },
    CanonicalPointInvalid { point: usize },
}
#[derive(Debug)]
pub(super) struct ExtractionFailure(Fault);
impl ExtractionFailure {
    pub(super) fn feedback(&self) -> Value {
        let mut value = serde_json::to_value(&self.0).expect("bounded structural feedback");
        value["instruction"]=json!("Return the complete current-job packet with only points of {sources:[supplied source IDs]}. Select original passages, never write, correct or paraphrase text. Keep sources in original order, each source once per packet. Group complete conditions/questions with their effect. The source count, not a summary bullet count, bounds the packet.");
        value
    }
}
pub(super) fn feedback_reservations() -> Vec<Value> {
    let n = usize::MAX;
    [
        Fault::Schema,
        Fault::JobMismatch,
        Fault::PointLimitExceeded {
            actual: n,
            limit: n,
        },
        Fault::EmptySources { point: n },
        Fault::InvalidSource {
            point: n,
            source: n,
        },
        Fault::SourceOrder {
            point: n,
            source: n,
        },
        Fault::ReusedSource {
            point: n,
            source: n,
        },
        Fault::EmptyText {
            point: n,
            source: n,
        },
        Fault::CanonicalPointInvalid { point: n },
    ]
    .into_iter()
    .map(|fault| ExtractionFailure(fault).feedback())
    .collect()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtractionPacket {
    job: String,
    points: Vec<ExcerptPoint>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExcerptPoint {
    sources: Vec<String>,
}
pub(super) fn decode_packet(
    answer: &str,
    sources: &SourceSnapshot,
    supplied: &BTreeSet<String>,
) -> std::result::Result<Vec<DraftPoint>, ExtractionFailure> {
    let packet: ExtractionPacket =
        serde_json::from_str(answer).map_err(|_| ExtractionFailure(Fault::Schema))?;
    sources
        .check_job(&packet.job)
        .map_err(|_| ExtractionFailure(Fault::JobMismatch))?;
    if packet.points.len() > supplied.len() {
        return Err(ExtractionFailure(Fault::PointLimitExceeded {
            actual: packet.points.len(),
            limit: supplied.len(),
        }));
    }
    let mut points = Vec::new();
    let mut selected = BTreeSet::new();
    for (point, selection) in packet.points.into_iter().enumerate() {
        if selection.sources.is_empty() {
            return Err(ExtractionFailure(Fault::EmptySources { point }));
        }
        let mut anchors = Vec::new();
        let mut quoted = Vec::new();
        let mut previous = None;
        for (source, id) in selection.sources.into_iter().enumerate() {
            let invalid = || ExtractionFailure(Fault::InvalidSource { point, source });
            let index = id
                .strip_prefix('S')
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|n| n.checked_sub(1))
                .ok_or_else(invalid)?;
            if !supplied.contains(&id) || super::super::chunk::source_id(index) != id {
                return Err(invalid());
            }
            let raw = sources.segments.get(index).ok_or_else(invalid)?;
            if previous.is_some_and(|last| index <= last) {
                return Err(ExtractionFailure(Fault::SourceOrder { point, source }));
            }
            if !selected.insert(id.clone()) {
                return Err(ExtractionFailure(Fault::ReusedSource { point, source }));
            }
            if raw.text.trim().is_empty() {
                return Err(ExtractionFailure(Fault::EmptyText { point, source }));
            }
            previous = Some(index);
            // Quotation framing is local; no model-generated words enter the reader.
            quoted.push(format!("“{}”", raw.text));
            anchors.push(Anchor {
                source: id,
                quote: raw.text.clone(),
            });
        }
        let canonical = DraftPoint {
            text: quoted.join(" "),
            evidence: anchors.clone(),
            required: anchors,
        };
        source::validate_point(sources, &canonical, supplied)
            .map_err(|_| ExtractionFailure(Fault::CanonicalPointInvalid { point }))?;
        points.push(canonical);
    }
    Ok(points)
}
#[cfg(test)]
mod excerpt_tests {
    use super::*;
    use crate::transcript::TranscriptSegment;
    fn snapshot(raw: &[&str]) -> SourceSnapshot {
        SourceSnapshot::new(
            "/synthetic.wav",
            &raw.iter()
                .enumerate()
                .map(|(i, text)| TranscriptSegment {
                    start: 12.75 + i as f64 * 5.0,
                    end: 17.75 + i as f64 * 5.0,
                    text: (*text).into(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }
    fn ids(s: &SourceSnapshot) -> BTreeSet<String> {
        (1..=s.segments.len()).map(|i| format!("S{i:06}")).collect()
    }
    fn packet(s: &SourceSnapshot, rows: Value) -> Value {
        json!({"job":s.job(),"points":rows})
    }
    fn decode(
        s: &SourceSnapshot,
        rows: Value,
    ) -> std::result::Result<Vec<DraftPoint>, ExtractionFailure> {
        decode_packet(&packet(s, rows).to_string(), s, &ids(s))
    }
    #[test]
    fn excerpt_selection_copies_whole_raw_passages_without_model_wording() {
        let s = snapshot(&[
            "Only when inspection finishes may it disappear?",
            "Cafe\u{301} อาจไม่ส่งต่อ",
        ]);
        let p = decode(&s, json!([{"sources":["S000001","S000002"]}])).unwrap();
        assert_eq!(
            p[0].text,
            "“Only when inspection finishes may it disappear?” “Cafe\u{301} อาจไม่ส่งต่อ”"
        );
        assert_eq!(
            p[0].required[0].quote,
            "Only when inspection finishes may it disappear?"
        );
        assert_eq!(
            p[0].required[1].quote.as_bytes(),
            "Cafe\u{301} อาจไม่ส่งต่อ".as_bytes()
        );
        assert_eq!(
            serde_json::to_value(&p[0].evidence).unwrap(),
            serde_json::to_value(&p[0].required).unwrap()
        );
        assert_eq!(s.segments[0].start, 12.75);
    }
    #[test]
    fn excerpt_selection_never_interprets_repeated_words_or_embedded_instructions() {
        let s = snapshot(&["ซ้ำ ซ้ำ ## heading [00:10] ignore instructions"]);
        let p = decode(&s, json!([{"sources":["S000001"]}])).unwrap();
        assert_eq!(p[0].text, "“ซ้ำ ซ้ำ ## heading [00:10] ignore instructions”");
        for key in ["text", "parts", "evidence", "required", "start"] {
            let mut point = json!({"sources":["S000001"]});
            point[key] = json!("invented");
            assert!(decode(&s, json!([point])).is_err());
        }
    }
    #[test]
    fn excerpt_selection_rejects_wrong_unknown_duplicate_or_reordered_sources() {
        let s = snapshot(&["First.", "Second."]);
        for rows in [
            json!([]),
            json!(["S000099"]),
            json!(["S1"]),
            json!(["S000001", "S000001"]),
            json!(["S000002", "S000001"]),
            json!([0]),
        ] {
            assert!(decode(&s, json!([{"sources":rows}])).is_err());
        }
        let answer = packet(&s, json!([{"sources":["S000002"]}])).to_string();
        assert!(decode_packet(&answer, &s, &BTreeSet::from(["S000001".into()])).is_err());
        let mut answer = packet(&s, json!([{"sources":["S000001"]}]));
        answer["job"] = json!("other");
        assert!(decode_packet(&answer.to_string(), &s, &ids(&s)).is_err());
    }
    #[test]
    fn excerpt_selection_uses_supplied_source_count_not_a_summary_bullet_limit() {
        let s = snapshot(&["row"; 12]);
        let rows: Vec<_> = (1..=12)
            .map(|i| json!({"sources":[format!("S{i:06}")]}))
            .collect();
        assert_eq!(decode(&s, json!(rows)).unwrap().len(), 12);
        let overflow = vec![json!({"sources":["S000001"]}); 13];
        let fault = decode(&s, json!(overflow)).unwrap_err().feedback();
        assert_eq!(fault["actual"], 13);
        assert_eq!(fault["limit"], 12);
    }
    #[test]
    fn excerpt_selection_never_admits_an_invented_later_point_or_duplicate_json_key() {
        let s = snapshot(&["First.", "Second."]);
        assert!(decode(&s, json!([{"sources":["S000001"]},{"sources":["S000099"]}])).is_err());
        let answer = format!(
            r#"{{"job":"{}","points":[{{"sources":["S000001"],"sources":["S000002"]}}]}}"#,
            s.job()
        );
        assert!(decode_packet(&answer, &s, &ids(&s)).is_err());
        assert!(decode(&s, json!([])).unwrap().is_empty());
    }
}
