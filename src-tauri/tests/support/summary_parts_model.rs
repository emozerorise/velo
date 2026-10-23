//! Deterministic protocol replies, not a semantic model or accuracy oracle.
use super::wire::Reply;
use serde_json::Value;
use velo_lib::{
    storage::settings_store::{SummaryFormat, SummarySettings},
    summary::JobInput,
    transcript::TranscriptSegment,
};

pub fn phase(request: &Value) -> &'static str {
    let system = request["messages"][0]["content"].as_str().unwrap();
    if system.contains("Routing protocol:") {
        "consistency"
    } else if system.contains("Interpret layout instructions only") {
        "layout"
    } else if system.contains("Compose one short global overview") {
        "overview"
    } else if system.contains("Write ONE finished source-owned part") {
        "part"
    } else {
        "legacy"
    }
}

pub fn reply(request: &Value) -> Reply {
    let user = request["messages"][1]["content"].as_str().unwrap();
    let system = request["messages"][0]["content"].as_str().unwrap();
    match phase(request) {
        "layout" => Reply::complete(if user.contains("Selected format: Custom") {
            r#"{"kind":"single_paragraph"}"#
        } else {
            r#"{"kind":"builtin"}"#
        }),
        "consistency" => Reply::complete(r#"{"parts":[]}"#),
        "part" => {
            let owned = user
                .split("Owned source interval: ")
                .nth(1)
                .unwrap()
                .lines()
                .next()
                .unwrap();
            let (first, last) = owned.split_once("..").unwrap();
            let body = format!("Supported topic [{first}] [{last}].");
            if system.contains("Use ONLY these top-level headings") {
                if system.contains("## ภาพรวม")
                    || system.contains("headings, with Markdown ##, when supported: ภาพรวม")
                {
                    Reply::complete(format!("## ภาพรวม\n{body}\n\n## ประเด็นที่คุยกัน\n- {body}"))
                } else {
                    Reply::complete(format!(
                        "## Overview\n{body}\n\n## Topics discussed\n- {body}"
                    ))
                }
            } else if system.contains("Use ONLY ##") {
                if system.contains("Use ONLY ## ภาพรวม") {
                    Reply::complete(format!("## ภาพรวม\n{body}\n\n## ประเด็นสำคัญ\n- {body}"))
                } else {
                    Reply::complete(format!("## Overview\n{body}\n\n## Key points\n- {body}"))
                }
            } else {
                Reply::complete(body)
            }
        }
        "overview" => {
            let ids: Vec<_> = user
                .split("[S")
                .skip(1)
                .filter_map(|s| {
                    let (id, _) = s.split_once(']')?;
                    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
                })
                .collect();
            Reply::complete(format!(
                "Global overview [S{}] [S{}].",
                ids.first().unwrap(),
                ids.last().unwrap()
            ))
        }
        _ => Reply::complete("Legacy only [S000001]."),
    }
}

pub fn job(base: String) -> JobInput {
    JobInput {
        path: "/synthetic-long.wav".into(),
        settings: SummarySettings {
            provider: "deepseek".into(),
            base_url: base,
            format: SummaryFormat::Custom,
            instructions: "Write one paragraph without headings or lists.".into(),
            language: "en".into(),
            ..Default::default()
        },
        segments: (0..32)
            .map(|i| TranscriptSegment {
                start: (i * 10) as f64,
                end: ((i + 1) * 10) as f64,
                text: format!(
                    "Synthetic topic {i}; effect only after completion. {}",
                    "x".repeat(1000)
                ),
            })
            .collect(),
        vocabulary: String::new(),
        transcript_language: "en".into(),
        api_key: None,
    }
}
