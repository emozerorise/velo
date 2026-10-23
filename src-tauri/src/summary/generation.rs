//! One request per generation. Content checks never trigger another model call.
use super::{completion_request, prompt, transport, validation, JobInput};
use crate::errors::{ModelErrorCode, Result, VeloError};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, Ordering},
};

pub(super) fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(VeloError::Summary("Summary cancelled".into()))
    } else {
        Ok(())
    }
}

pub(super) struct Generation<'a> {
    input: &'a JobInput,
    cancel: &'a AtomicBool,
    used: usize,
}
impl<'a> Generation<'a> {
    pub fn new(input: &'a JobInput, cancel: &'a AtomicBool) -> Self {
        Self {
            input,
            cancel,
            used: 0,
        }
    }
    fn request(&self, source: &str, notes: bool, combine: bool) -> transport::ChatRequest {
        let mut system = prompt::direct_system(&self.input.settings, &self.input.vocabulary, notes);
        if combine {
            system.push_str("\nCombine these chronological excerpt summaries into ONE final answer. They are generated notes, not verbatim source. Preserve distinct topics, reasons, numbers, conditions, uncertainty and explicit later resolutions; merge only repetition. Never infer a resolution merely from a later timestamp. Keep the supporting source IDs supplied in the notes. Do not introduce new facts.");
        }
        if notes {
            system.push_str("\nThese excerpt notes will be combined once. Aim for at most 2000 tokens including citations; keep material topics, reasons, numbers and conditions, omit repetition and use only the smallest sufficient citation set. Final-document layout instructions apply to the combination, not these notes.");
        }
        let user = format!(
            "{source}{}",
            prompt::direct_reminder(&self.input.settings, notes)
        );
        let mut req = completion_request(&self.input.settings, system, user, notes);
        // Allocate answer room up front, rather than pay to regenerate a cut-off answer.
        if self.input.settings.provider == "deepseek" {
            req.max_tokens = if notes { 4096 } else { 8192 }.min(req.context_tokens / 4);
        }
        req.system.push_str(&format!("\nFinish the complete answer within {} output tokens including citations. Avoid repetition and long verbatim quotations.", req.max_tokens));
        req
    }
    pub fn fits(&self, source: &str, notes: bool, combine: bool) -> bool {
        let req = self.request(source, notes, combine);
        fits(&req)
    }
    pub async fn generate(
        &mut self,
        source: &str,
        labels: &BTreeSet<String>,
        notes: bool,
        combine: bool,
    ) -> Result<String> {
        check_cancel(self.cancel)?;
        let req = self.request(source, notes, combine);
        if !fits(&req) {
            return Err(VeloError::Summary("summary_context_limit".into()));
        }
        // At most 32 terminal excerpts, their failed ancestors, and one combination.
        if self.used >= 64 {
            return Err(VeloError::Summary("summary_work_limit".into()));
        }
        self.used += 1;
        tracing::debug!(
            phase = if combine {
                "combine"
            } else if notes {
                "excerpt"
            } else {
                "direct"
            },
            physical_call = self.used,
            source_bytes = source.len(),
            output_tokens = req.max_tokens,
            "summary request"
        );
        let result = transport::stream_chat(
            transport::dialect_for(&self.input.settings.provider).as_ref(),
            &self.input.settings.base_url,
            self.input.api_key.as_deref(),
            &req,
            self.cancel,
            |_| {},
            || {},
        )
        .await;
        check_cancel(self.cancel)?;
        let answer = result.map_err(|error| match error {
            VeloError::Model {
                code: ModelErrorCode::OutputLimit,
                ..
            } => VeloError::Summary("summary_output_limit".into()),
            other => other,
        })?;
        let labels = labels.iter().map(String::as_str).collect();
        let mut answer = validation::canonical_references(answer.trim(), &labels);
        if combine {
            // Combining notes can introduce an ID absent from those notes. Drop
            // that link locally; never guess a replacement or rewrite the prose.
            // Validation below still requires a supplied citation in every block.
            let unknown: Vec<_> = validation::note_labels(&answer)
                .into_iter()
                .filter(|id| !labels.contains(id))
                .map(str::to_owned)
                .collect();
            for id in &unknown {
                answer = answer.replace(&format!("[{id}]"), "");
            }
            if !unknown.is_empty() {
                tracing::debug!(
                    removed = unknown.len(),
                    "discarded unsupplied combination references"
                );
            }
        }
        validation::validate(&answer, &labels).map_err(VeloError::Summary)?;
        Ok(answer)
    }
}
fn fits(req: &transport::ChatRequest) -> bool {
    // Conservative UTF-8 estimate retained for Thai and escaped source rows.
    req.max_tokens > 0
        && req.max_tokens < req.context_tokens
        && req.system.len().saturating_add(req.user.len())
            <= req.context_tokens.saturating_sub(req.max_tokens) as usize * 3
}
