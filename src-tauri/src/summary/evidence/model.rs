use super::super::{completion_request, transport, JobInput};
use crate::errors::{Result, VeloError};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct EvidenceModel<'a> {
    pub input: &'a JobInput,
    pub cancel: &'a AtomicBool,
    pub observer: &'a dyn super::super::evidence_pipeline::EvidenceObserver,
}
impl EvidenceModel<'_> {
    pub fn check_cancel(&self) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(VeloError::Summary("Summary cancelled".into()))
        } else {
            Ok(())
        }
    }
    pub fn fits(&self, system: &str, user: &str) -> bool {
        let req = completion_request(&self.input.settings, String::new(), String::new(), false);
        system.len().saturating_add(user.len())
            <= self
                .input
                .settings
                .context_tokens
                .saturating_sub(req.max_tokens) as usize
                * 3
    }
    pub async fn complete(&self, stage: &str, system: String, user: String) -> Result<String> {
        self.check_cancel()?;
        if !self.fits(&system, &user) {
            return Err(VeloError::Summary("Evidence exceeds the context budget; increase the context window. Previous summary unchanged.".into()));
        }
        let req = completion_request(&self.input.settings, system, user, false);
        // Artifact observation is injected only by the opt-in acceptance harness.
        // This allowlist intentionally excludes JobInput, keys, headers and URLs.
        self.observer.record("request",&serde_json::json!({"stage":stage,"system":req.system,"user":req.user,"max_tokens":req.max_tokens,"context_tokens":req.context_tokens}))?;
        let dialect = transport::dialect_for(&self.input.settings.provider);
        let answer = transport::stream_chat(
            dialect.as_ref(),
            &self.input.settings.base_url,
            self.input.api_key.as_deref(),
            &req,
            self.cancel,
            |_| {},
            || {},
        )
        .await?;
        self.check_cancel()?;
        self.observer.record(
            "response",
            &serde_json::json!({"stage":stage,"text":answer}),
        )?;
        Ok(answer)
    }
}
