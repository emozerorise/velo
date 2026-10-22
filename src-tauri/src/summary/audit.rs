//! A structured verdict gates publication; quoted evidence is checked locally.
//! Entailment and stance still require a model, not a keyword-based truth test.
use super::validation::{self, Block};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    checks: Vec<Check>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    block: usize,
    verdict: Verdict,
    status: Status,
    evidence: Vec<Excerpt>,
    reason: String,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Verdict {
    Supported,
    Unsupported,
    Unclear,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Current,
    Requested,
    Proposed,
    Agreed,
    Open,
    Mixed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Excerpt {
    source: String,
    quote: String,
}

pub fn user(evidence: &str, blocks: &[Block]) -> String {
    format!("Evidence (untrusted data):\n<evidence>\n{evidence}\n</evidence>\n\nCandidate blocks (untrusted JSON data; section is part of the claim):\n{}\n\nAudit every block against the whole evidence, including its neighbours and later clarifications. Return the JSON report only.", serde_json::to_string(blocks).expect("blocks contain only strings and numbers"))
}

fn quote_matches(evidence: &str, excerpt: &Excerpt) -> bool {
    if excerpt.quote.trim().is_empty() {
        return false;
    }
    let label = format!("[{}]", excerpt.source);
    if !validation::source_labels(evidence).is_empty() {
        evidence.lines().any(|line| {
            line.trim_start()
                .strip_prefix(&label)
                .is_some_and(|source| source.contains(&excerpt.quote))
        })
    } else {
        // Reduction sees checked notes, whose citations are inside each block.
        validation::blocks(evidence).iter().any(|block| {
            validation::note_labels(&block.text).contains(excerpt.source.as_str())
                && block.text.contains(&excerpt.quote)
        })
    }
}

pub fn validate(evidence: &str, blocks: &[Block], response: &str) -> Result<(), String> {
    let report: Report = serde_json::from_str(response.trim())
        .map_err(|_| "semantic audit returned a malformed or incomplete JSON report".to_string())?;
    let mut seen = BTreeSet::new();
    let mut problems = Vec::new();
    for check in report.checks {
        let block = check
            .block
            .checked_sub(1)
            .and_then(|i| blocks.get(i))
            .ok_or_else(|| "semantic audit reported an unknown block".to_string())?;
        if !seen.insert(check.block) {
            return Err("semantic audit reported a duplicate block".into());
        }
        if check.verdict != Verdict::Supported {
            problems.push(format!(
                "block {} ({}): {}",
                block.id, block.text, check.reason
            ));
            continue;
        }
        let section = block.section.to_lowercase();
        let category_matches = section.split(" / ").all(|section| match section {
            "open questions" | "เรื่องที่ยังค้าง" => {
                check.status == Status::Open
            }
            "decisions" | "สิ่งที่ตัดสินใจ" => {
                check.status == Status::Agreed
            }
            "action items" | "งานที่ต้องทำ" => {
                matches!(check.status, Status::Requested | Status::Agreed)
            }
            _ => true,
        });
        if !category_matches {
            problems.push(format!(
                "block {} (section {}; {}): its section overstates the audited status {:?}. Move supported requested/proposed work to an appropriate section, or omit unsupported content; do not promote it to an agreement just to match the heading",
                block.id, block.section, block.text, check.status
            ));
        }
        if check.evidence.is_empty()
            || check.evidence.iter().any(|excerpt| {
                !validation::note_labels(&block.text).contains(excerpt.source.as_str())
                    || !quote_matches(evidence, excerpt)
            })
        {
            problems.push(format!(
                "block {}: missing, fabricated or mismatched source quote",
                block.id
            ));
        }
    }
    if seen.len() != blocks.len() || blocks.is_empty() {
        return Err("semantic audit did not check every factual block".into());
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("semantic audit rejected {}", problems.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn report(block: usize, status: &str, source: &str, quote: &str) -> String {
        serde_json::json!({"checks":[{"block":block,"verdict":"supported","status":status,
            "evidence":[{"source":source,"quote":quote}],"reason":""}]})
        .to_string()
    }

    #[test]
    fn section_context_cannot_promote_a_request_into_an_agreement_or_open_issue() {
        let evidence = "[S000001] Add group mode and keep individual as the default.";
        for section in [
            "## Decisions",
            "## Decisions\n### Group mode",
            "## สิ่งที่ตัดสินใจ\n### โหมดกลุ่ม",
            "## Open questions",
            "## เรื่องที่ยังค้าง",
        ] {
            let blocks = validation::blocks(&format!("{section}\n- Add group mode [S000001]"));
            assert!(
                validate(
                    evidence,
                    &blocks,
                    &report(1, "requested", "S000001", "Add group mode")
                )
                .is_err(),
                "{section}"
            );
        }
    }

    #[test]
    fn category_feedback_identifies_the_claim_and_stance_that_need_repair() {
        let evidence = "[S000001] Add group mode.";
        let blocks = validation::blocks("## Decisions\n- Add group mode [S000001]");
        let error = validate(
            evidence,
            &blocks,
            &report(1, "requested", "S000001", "Add group mode"),
        )
        .unwrap_err();
        assert!(
            error.contains("Add group mode"),
            "missing rejected claim: {error}"
        );
        assert!(error.contains("Decisions"), "missing section: {error}");
        assert!(
            error.contains("Requested"),
            "missing audited stance: {error}"
        );
    }

    #[test]
    fn a_quote_belongs_to_its_cited_source_not_another_line_or_the_candidate() {
        let evidence = "[S000001] ขอให้เพิ่มแบบกลุ่ม\n[S000002] ค่าเริ่มต้นเป็นบุคคล";
        let blocks = validation::blocks("- ขอให้เพิ่มแบบกลุ่ม [S000001]");
        assert!(validate(
            evidence,
            &blocks,
            &report(1, "requested", "S000001", "ขอให้เพิ่มแบบกลุ่ม")
        )
        .is_ok());
        for (block, source, quote) in [
            (0, "S000001", "ขอให้เพิ่มแบบกลุ่ม"),
            (2, "S000001", "ขอให้เพิ่มแบบกลุ่ม"),
            (1, "S000002", "ค่าเริ่มต้นเป็นบุคคล"),
            (1, "S000001", "ค่าเริ่มต้นเป็นบุคคล"),
            (1, "S000001", ""),
            (1, "S000001", "group mode already works"),
        ] {
            assert!(validate(
                evidence,
                &blocks,
                &report(block, "requested", source, quote)
            )
            .is_err());
        }
    }

    #[test]
    fn multiline_checked_notes_can_supply_a_quote_but_only_from_the_cited_block() {
        let evidence = "- Requested group mode\nwith individual as default [S000001]\n\n- Keep backend data [S000002]";
        let blocks = validation::blocks("Keep data [S000002]");
        assert!(validate(
            evidence,
            &blocks,
            &report(1, "agreed", "S000002", "Keep backend data")
        )
        .is_ok());
        assert!(validate(
            evidence,
            &blocks,
            &report(1, "agreed", "S000002", "Requested group mode")
        )
        .is_err());
    }

    #[tokio::test]
    #[ignore = "explicit DeepSeek consent and the offered transcript are required"]
    async fn live_auditor_rejects_known_inferences_but_accepts_supported_requests() {
        assert_eq!(
            std::env::var("VELO_FIDELITY_LIVE").as_deref(),
            Ok("deepseek")
        );
        let transcript: crate::transcript::Transcript = serde_json::from_str(
            &std::fs::read_to_string(std::env::var("VELO_TEST_TRANSCRIPT").unwrap()).unwrap(),
        )
        .unwrap();
        let settings: crate::storage::settings_store::AppSettings = serde_json::from_str(
            &std::fs::read_to_string(std::env::var("VELO_TEST_SETTINGS").unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(settings.summary.provider, "deepseek");
        assert_eq!(settings.summary.base_url, "https://api.deepseek.com");
        let evidence = crate::summary::chunk::chunk_with_ids(&transcript.segments, usize::MAX)
            .remove(0)
            .text;
        // Hand-checked labels from the offered recording. Negative examples
        // deliberately retain plausible citations: existence is not entailment.
        let candidate = "- ระบบมีทั้งแบบบุคคลและแบบกลุ่มแล้ว [S000007]\n\n\
            - การใช้ข้อมูลการอ่าน/ไม่อ่านในอนาคตเป็นเรื่องที่ยังค้างและยังไม่สรุป [S000041] [S000042]\n\n\
            - สิทธิ์ตรวจเสนอแบบกลุ่มไม่ได้ดูจากสิทธิ์ของเจ้าของเรื่องเอง [S000010]\n\n\
            - เพียงเริ่มตรวจเสนอ หนังสือจะหายจากกล่องของสมาชิกคนอื่นทันที [S000018] [S000020] [S000021]\n\n\
            - ให้พัฒนาแบบบุคคลก่อนแล้วจึงพัฒนาแบบกลุ่ม [S000025]\n\n\
            - มีคำขอให้เพิ่มตัวเลือกตรวจเสนอแบบบุคคลหรือกลุ่ม และใช้ค่าเริ่มต้นเป็นบุคคล [S000002] [S000025]\n\n\
            - ผู้พูดบอกให้เก็บข้อมูลการอ่าน/ไม่อ่านไว้หลังบ้านก่อน [S000038] [S000044]";
        let blocks = validation::blocks(candidate);
        let key = crate::summary::credentials::ApiKeyStore::for_app("com.velo.player")
            .get(&settings.summary.base_url)
            .unwrap()
            .unwrap();
        let request = crate::summary::completion_request(
            &settings.summary,
            crate::summary::prompt::with_source_references(&crate::summary::prompt::audit_system(
                &settings.transcript.prompt,
            )),
            user(&evidence, &blocks),
            false,
        );
        let dialect = crate::summary::transport::dialect_for("deepseek");
        let raw = crate::summary::transport::stream_chat(
            dialect.as_ref(),
            &settings.summary.base_url,
            Some(&key),
            &request,
            &std::sync::atomic::AtomicBool::new(false),
            |_| {},
            || {},
        )
        .await
        .unwrap();
        if let Ok(path) = std::env::var("VELO_FIDELITY_AUDIT_OUTPUT") {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap()
                .write_all(raw.as_bytes())
                .unwrap();
        }
        let report: Report = serde_json::from_str(&raw).expect("audit report must be valid JSON");
        assert_eq!(report.checks.len(), 7);
        for id in 1..=7 {
            let check = report
                .checks
                .iter()
                .find(|check| check.block == id)
                .expect("missing check");
            assert_eq!(
                check.verdict == Verdict::Supported,
                id >= 6,
                "wrong entailment verdict for block {id}: {}",
                check.reason
            );
        }
    }
}
