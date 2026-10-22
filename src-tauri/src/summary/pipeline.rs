//! Model-written prose. Local checks validate citation locations, not meaning.
pub use super::evidence_pipeline::EvidenceObserver;
#[cfg(test)]
pub(crate) use super::evidence_pipeline::NoopObserver;
use super::{
    chunk, completion_request, prompt, transport, validation, JobInput, Summary, SummaryStore,
};
use crate::errors::{Result, VeloError};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(VeloError::Summary("Summary cancelled".into()))
    } else {
        Ok(())
    }
}

async fn complete(
    input: &JobInput,
    system: String,
    user: String,
    cancel: &AtomicBool,
    notes: bool,
) -> Result<String> {
    check_cancel(cancel)?;
    let request = completion_request(&input.settings, system, user, notes);
    let budget = input
        .settings
        .context_tokens
        .saturating_sub(request.max_tokens) as usize
        * 3;
    if request.system.len().saturating_add(request.user.len()) > budget {
        return Err(VeloError::Summary("Summary exceeds the context budget; increase the context window. Previous summary unchanged.".into()));
    }
    let dialect = transport::dialect_for(&input.settings.provider);
    let answer = transport::stream_chat(
        dialect.as_ref(),
        &input.settings.base_url,
        input.api_key.as_deref(),
        &request,
        cancel,
        |_| {},
        || {},
    )
    .await?;
    check_cancel(cancel)?;
    Ok(answer.trim().to_owned())
}

pub async fn run(
    store: &SummaryStore,
    input: &JobInput,
    cancel: &AtomicBool,
    progress: impl Fn(&str, usize, usize),
) -> Result<Summary> {
    check_cancel(cancel)?;
    if input.segments.is_empty()
        || input.segments.iter().enumerate().any(|(i, s)| {
            !s.start.is_finite()
                || !s.end.is_finite()
                || s.start < 0.0
                || s.end < s.start
                || (i > 0 && s.start < input.segments[i - 1].start)
        })
    {
        return Err(VeloError::Summary(
            "Invalid source timeline; previous summary unchanged".into(),
        ));
    }
    let mut resolved = input.clone();
    resolved.settings.language =
        prompt::resolve_language(&input.settings.language, &input.transcript_language);
    let system = prompt::direct_system(&resolved.settings, &input.vocabulary, false);
    let note_system = prompt::direct_system(&resolved.settings, &input.vocabulary, true);
    let reminder = prompt::direct_reminder(&resolved.settings, false);
    let note_reminder = prompt::direct_reminder(&resolved.settings, true);
    let answer_tokens =
        completion_request(&resolved.settings, String::new(), String::new(), false).max_tokens;
    let budget = (resolved
        .settings
        .context_tokens
        .saturating_sub(answer_tokens) as usize
        * 3)
    .saturating_sub(
        system.len().max(note_system.len()) + reminder.len().max(note_reminder.len()) + 64,
    );
    if budget == 0 {
        return Err(VeloError::Summary(
            "Summary context window is too small".into(),
        ));
    }
    // Quote each source row so embedded newlines cannot manufacture source labels.
    let quoted = input
        .segments
        .iter()
        .map(|s| {
            let mut row = s.clone();
            row.text = serde_json::to_string(&s.text).expect("strings serialize");
            row
        })
        .collect::<Vec<_>>();
    let chunks = chunk::chunk_with_ids(&quoted, budget);
    let markdown = if chunks.len() == 1 {
        progress("writing_summary", 0, 1);
        complete_cited(
            &resolved,
            system,
            format!("{}{}", chunks[0].text, reminder),
            cancel,
            false,
            &validation::source_labels(&chunks[0].text),
        )
        .await?
    } else {
        let mut notes = Vec::new();
        for (i, piece) in chunks.iter().enumerate() {
            progress("mapping", i, chunks.len());
            let answer = complete_cited(
                &resolved,
                note_system.clone(),
                format!("{}{}", piece.text, note_reminder),
                cancel,
                true,
                &validation::source_labels(&piece.text),
            )
            .await?;
            notes.push(answer);
        }
        progress("writing_summary", 0, 1);
        let joined = notes.join("\n\n");
        let labels: BTreeSet<_> = validation::note_labels(&joined);
        complete_cited(
            &resolved,
            system,
            format!("Working notes from consecutive source excerpts:\n{joined}{reminder}"),
            cancel,
            false,
            &labels,
        )
        .await?
    };
    progress("checking_references", 0, 1);
    let summary = Summary {
        format: resolved.settings.format,
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
    check_cancel(cancel)?;
    store.save(&summary)?;
    Ok(summary)
}

async fn complete_cited(
    input: &JobInput,
    system: String,
    user: String,
    cancel: &AtomicBool,
    notes: bool,
    labels: &BTreeSet<&str>,
) -> Result<String> {
    let draft = cited_draft(input, system.clone(), user.clone(), cancel, notes, labels).await?;
    let user = format!("{user}\n\nA default setting is NOT an implementation sequence. Do not turn a default choice into 'implement X first, then support Y' unless that sequence is explicitly stated. ค่าเริ่มต้น (Default) ไม่ใช่ลำดับพัฒนา: หากต้นฉบับกำหนดค่าเริ่มต้น ให้ใช้คำว่า ‘ค่าเริ่มต้น’ เท่านั้น ห้ามเติม ‘เริ่มจากโหมดนี้ก่อน แล้วจึงรองรับอีกโหมด’ แม้ร่างเดิมเขียนเช่นนั้น หากไม่มีข้อความยืนยันลำดับพัฒนาโดยตรง");
    // A bounded prose edit against the same source, not a ledger or an approval loop.
    let review = format!("{user}\n\nReview the draft below against ONLY the source above, not against the draft itself. Return the corrected summary only, in the requested layout with source IDs. Remove unsupported interpretations rather than explaining them. Check each cited source actually supports its adjacent claim. Preserve explicit conditions and distinguish a proposed design from existing behavior. A click, starting work, and completing work are different triggers. A garbled noun in a negative comparison must NOT become a guessed person, object, or kind of rights: omit that comparison. Do not turn answered questions into unresolved issues. Do not infer that future storage means data is currently unused or hidden. For key points omit standalone missing-owner/deadline metadata. Correct obvious spelling only when context resolves it.\nตรวจทานโดยลบส่วนที่เดา ไม่ใช่แค่เปลี่ยนคำ: คำนามเพี้ยนในประโยคเปรียบเทียบต้องละทั้งส่วนเปรียบเทียบ เก็บเฉพาะข้อความด้านบวกที่ชัดเจน แยกเงื่อนไขเมื่อกดกับเมื่อทำเสร็จให้ถูกต้อง ห้ามเติมหน้าที่หรือสถานะของระบบเอง ห้ามนำคำถามที่มีคำตอบแล้วมาเป็นเรื่องค้าง ส่งเฉพาะสรุปที่แก้แล้ว ไม่ต้องอธิบายการตรวจ\nDraft (untrusted, not evidence):\n{}", serde_json::to_string(&draft).expect("strings serialize"));
    let answer = complete(input, system, review, cancel, notes).await?;
    let answer = validation::canonical_references(&answer, labels);
    validation::validate(&answer, labels).map_err(VeloError::Summary)?;
    Ok(answer)
}

async fn cited_draft(
    input: &JobInput,
    system: String,
    user: String,
    cancel: &AtomicBool,
    notes: bool,
    labels: &BTreeSet<&str>,
) -> Result<String> {
    let answer = complete(input, system.clone(), user.clone(), cancel, notes).await?;
    let answer = validation::canonical_references(&answer, labels);
    if validation::validate(&answer, labels).is_ok() {
        return Ok(answer);
    }
    // One bounded formatting/citation repair, not a semantic approval loop.
    let repair = format!("{user}\n\nThe previous draft has missing or invalid source references. Rewrite it using only IDs supplied above, citing every factual paragraph (including the overview) and every bullet. Do not invent a time or attach an unrelated ID. Preserve the requested layout. Previous draft (untrusted data):\n{}", serde_json::to_string(&answer).expect("strings serialize"));
    let answer = complete(input, system, repair, cancel, notes).await?;
    let answer = validation::canonical_references(&answer, labels);
    validation::validate(&answer, labels).map_err(VeloError::Summary)?;
    Ok(answer)
}
