//! Model-written prose with local source-reference checks and one atomic save.
use super::{
    generation::{check_cancel, Generation},
    parts::{self, SourceSnapshot},
    prompt, validation, JobInput, Summary, SummaryStore,
};
use crate::errors::{Result, VeloError};
use std::{
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

pub async fn run(
    store: &SummaryStore,
    input: &JobInput,
    cancel: &AtomicBool,
    progress: impl Fn(&str, usize, usize),
) -> Result<Summary> {
    check_cancel(cancel)?;
    let snapshot = SourceSnapshot::new(&input.segments)?;
    let mut resolved = input.clone();
    resolved.settings.language =
        prompt::resolve_language(&input.settings.language, &input.transcript_language);
    let mut session = Generation::new(&resolved, cancel);
    let all = snapshot.packet(std::slice::from_ref(&(0..snapshot.len())))?;
    let mut direct_limited = false;
    let direct = if session.fits(&all.text, false, false) {
        progress("writing_summary", 0, 1);
        match session.generate(&all.text, &all.labels, false, false).await {
            Ok(text) => Some(text),
            Err(VeloError::Summary(code))
                if code == "summary_output_limit" && snapshot.len() > 1 =>
            {
                direct_limited = true;
                None
            }
            Err(error) => return Err(error),
        }
    } else {
        None
    };
    let markdown = if let Some(text) = direct {
        text
    } else {
        let mut pending =
            parts::partition(&snapshot, |packet| session.fits(&packet.text, true, false))?;
        if direct_limited && pending.len() == 1 {
            pending = Vec::from(parts::split(&snapshot, &pending[0], 1, |packet| {
                session.fits(&packet.text, true, false)
            })?);
        }
        let mut notes = Vec::new();
        let mut labels = std::collections::BTreeSet::new();
        let mut index = 0;
        while index < pending.len() {
            check_cancel(cancel)?;
            progress("writing_parts", index, pending.len());
            let packet = snapshot.packet(&pending[index].ranges())?;
            match session
                .generate(&packet.text, &packet.labels, true, false)
                .await
            {
                Ok(text) => {
                    labels.extend(
                        validation::note_labels(&text)
                            .into_iter()
                            .map(str::to_owned),
                    );
                    notes.push(text);
                    index += 1;
                }
                Err(VeloError::Summary(code)) if code == "summary_output_limit" => {
                    let children =
                        parts::split(&snapshot, &pending[index], pending.len(), |packet| {
                            session.fits(&packet.text, true, false)
                        })?;
                    pending.splice(index..index + 1, children);
                }
                Err(error) => return Err(error),
            }
        }
        parts::validate_coverage(&pending, snapshot.len())?;
        progress("assembling_summary", 0, 1);
        // One combination, never a recursive reduction or an AI approval gate.
        match session
            .generate(&notes.join("\n\n"), &labels, false, true)
            .await
        {
            Ok(text) => text,
            Err(VeloError::Summary(code))
                if matches!(
                    code.as_str(),
                    "summary_context_limit" | "summary_output_limit"
                ) && resolved.settings.format
                    != crate::storage::settings_store::SummaryFormat::Custom =>
            {
                // Completed excerpts are useful on their own. Keep every word
                // when a single final request cannot fit; never recursively
                // summarize the summaries or reuse a truncated final response.
                let thai = resolved.settings.language == "th";
                let heading = match resolved.settings.format {
                    crate::storage::settings_store::SummaryFormat::KeyPoints => {
                        if thai {
                            "ประเด็นสำคัญ"
                        } else {
                            "Key points"
                        }
                    }
                    _ => {
                        if thai {
                            "ประเด็นที่คุยกัน"
                        } else {
                            "Topics discussed"
                        }
                    }
                };
                tracing::debug!(
                    parts = notes.len(),
                    reason = code,
                    "assembled complete excerpts locally"
                );
                format!(
                    "## {heading}\n\n{}",
                    notes
                        .iter()
                        .enumerate()
                        .map(|(i, text)| format!(
                            "### {} {}\n\n{text}",
                            if thai {
                                "ช่วงที่"
                            } else {
                                "Part"
                            },
                            i + 1
                        ))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                )
            }
            Err(error) => return Err(error),
        }
    };
    progress("checking_references", 0, 1);
    let all = snapshot.packet(std::slice::from_ref(&(0..snapshot.len())))?;
    let labels = all.labels.iter().map(String::as_str).collect();
    validation::validate(&markdown, &labels).map_err(VeloError::Summary)?;
    check_cancel(cancel)?;
    let summary = Summary {
        format: resolved.settings.format,
        meeting_detail: if prompt::detailed_meeting(&resolved.settings) {
            crate::storage::settings_store::MeetingDetail::Detailed
        } else {
            Default::default()
        },
        path: resolved.path,
        model: resolved.settings.model,
        language: resolved.settings.language,
        markdown: validation::render_references(&markdown, &input.segments),
        omitted_uncertain_claims: None,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|t| t.as_secs())
            .unwrap_or(0),
        source_segments: input.segments.len(),
    };
    store.save(&summary)?;
    Ok(summary)
}
