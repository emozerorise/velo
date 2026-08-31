//! The instructions handed to the model.
//!
//! Output is Markdown rather than JSON on purpose: an 8B model writing long
//! Thai JSON fails often, and a parse failure would present as an empty
//! panel. Markdown degrades to readable text instead, and the `[mm:ss]`
//! citations are picked out by pattern at render time.

use crate::storage::settings_store::SummarySettings;

/// Section headings, per output language. The model is far more likely to
/// keep a structure it has been shown verbatim.
fn headings(language: &str) -> [&'static str; 5] {
    match language {
        "th" => [
            "ภาพรวม",
            "ประเด็นที่คุยกัน",
            "สิ่งที่ตัดสินใจ",
            "งานที่ต้องทำ",
            "เรื่องที่ยังค้าง",
        ],
        _ => [
            "Overview",
            "Topics discussed",
            "Decisions",
            "Action items",
            "Open questions",
        ],
    }
}

/// What "auto" actually means: the language whisper reported for this
/// recording. Without this the prompt contradicts itself -- it asks for the
/// meeting's language while showing English headings as the template, and
/// the model follows the headings.
pub fn resolve_language(configured: &str, detected: &str) -> String {
    if configured != "auto" {
        return configured.to_string();
    }

    match detected.trim() {
        "" | "auto" => "auto".to_string(),
        language => language.to_string(),
    }
}

fn language_rule(language: &str) -> String {
    match language {
        "th" => "Write the summary in Thai.".into(),
        "en" => "Write the summary in English.".into(),
        // Mixed Thai/English meetings are the norm here, so "the language of
        // the meeting" is a clearer instruction than naming one.
        _ => "Write the summary in the language the meeting was held in.".into(),
    }
}

fn extras(settings: &SummarySettings, vocabulary: &str) -> String {
    let mut out = String::new();

    // The same domain terms that fix transcription stop the summariser from
    // mangling product and team names.
    if !vocabulary.trim().is_empty() {
        out.push_str(&format!(
            "\n\nTerms used by this team, spelled correctly:\n{}",
            vocabulary.trim()
        ));
    }

    if !settings.instructions.trim().is_empty() {
        out.push_str(&format!(
            "\n\nAdditional instructions from the user:\n{}",
            settings.instructions.trim()
        ));
    }

    out
}

/// The map stage: notes from one excerpt, not a finished summary.
pub fn map_system(settings: &SummarySettings, vocabulary: &str) -> String {
    format!(
        "You are taking notes on one excerpt of a meeting transcript.\n\
         Each line of the excerpt begins with its timestamp, as [mm:ss].\n\n\
         List only what this excerpt actually contains: topics raised, \
         decisions made, tasks assigned (with who and by when, if said), and \
         questions left unanswered.\n\n\
         Rules:\n\
         - Every point ends with the timestamp it came from, copied exactly, \
         e.g. [12:34].\n\
         - Use short bullet points. No preamble, no closing remark, no headings.\n\
         - Never invent a decision, a name, or a date that is not in the text.\n\
         - {}{}",
        language_rule(&settings.language),
        extras(settings, vocabulary)
    )
}

/// The reduce stage: one summary out of the notes, in the fixed shape.
pub fn reduce_system(settings: &SummarySettings, vocabulary: &str) -> String {
    let [overview, topics, decisions, actions, open] = headings(&settings.language);

    format!(
        "You are writing the final summary of a meeting from notes taken \
         across it, in order. Timestamps in the notes, written as [mm:ss], \
         are how a reader jumps back to the recording.\n\n\
         Write Markdown with exactly these five headings, in this order:\n\
         ## {}\n## {}\n## {}\n## {}\n## {}\n\n\
         Rules:\n\
         - Two or three sentences under the first heading; bullet points under \
         the rest.\n\
         - Every bullet keeps the timestamp it came from, copied exactly.\n\
         - Merge points that repeat across notes; keep the earliest timestamp.\n\
         - Under the action items heading, name the owner and the deadline \
         when the notes give them.\n\
         - Write \"-\" under a heading that has nothing.\n\
         - Never invent anything that is not in the notes.\n\
         - {}{}",
        overview,
        topics,
        decisions,
        actions,
        open,
        language_rule(&settings.language),
        extras(settings, vocabulary)
    )
}

/// Appended after the excerpt, not before it. A transcript runs to tens of
/// thousands of tokens, which leaves the system prompt a long way behind by
/// the time the model starts writing -- and the rule it drops first is the
/// one that makes the summary navigable. Repeat it where it is about to be
/// used.
pub fn transcript_reminder() -> &'static str {
    "\n\n---\nThat is the end of the excerpt. Write the summary now, under the \
     headings you were given, and end every single point with the timestamp it \
     came from -- copied exactly from the lines above, in square brackets, like \
     [12:34]. A point without its timestamp is not usable."
}

/// The same rule for the reduce stage, where the input is notes rather than
/// transcript lines.
pub fn notes_reminder() -> &'static str {
    "\n\n---\nThat is the end of the notes. Write the final summary now, and \
     carry each point's timestamp across unchanged, in square brackets, like \
     [12:34]. A point without its timestamp is not usable."
}

/// A short transcript skips the map stage: it is already the notes.
pub fn single_pass_system(settings: &SummarySettings, vocabulary: &str) -> String {
    let [overview, topics, decisions, actions, open] = headings(&settings.language);

    format!(
        "You are summarising a meeting transcript. Each line begins with its \
         timestamp, as [mm:ss], which is how a reader jumps back to the \
         recording.\n\n\
         Write Markdown with exactly these five headings, in this order:\n\
         ## {}\n## {}\n## {}\n## {}\n## {}\n\n\
         Rules:\n\
         - Two or three sentences under the first heading; bullet points under \
         the rest.\n\
         - Every bullet ends with the timestamp it came from, copied exactly, \
         e.g. [12:34].\n\
         - Under the action items heading, name the owner and the deadline \
         when they are said.\n\
         - Write \"-\" under a heading that has nothing.\n\
         - Never invent a decision, a name, or a date that is not in the \
         transcript.\n\
         - {}{}",
        overview,
        topics,
        decisions,
        actions,
        open,
        language_rule(&settings.language),
        extras(settings, vocabulary)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(language: &str) -> SummarySettings {
        SummarySettings {
            language: language.into(),
            ..SummarySettings::default()
        }
    }

    #[test]
    fn thai_prompts_carry_thai_headings() {
        let prompt = single_pass_system(&settings("th"), "");
        assert!(prompt.contains("## ภาพรวม"));
        assert!(prompt.contains("## งานที่ต้องทำ"));
        assert!(prompt.contains("Write the summary in Thai"));
    }

    #[test]
    fn auto_resolves_to_what_whisper_heard() {
        assert_eq!(resolve_language("auto", "th"), "th");
        assert_eq!(resolve_language("auto", "en"), "en");
        // An explicit choice always wins over the recording.
        assert_eq!(resolve_language("en", "th"), "en");
        // Nothing detected: fall back to the language-neutral instruction.
        assert_eq!(resolve_language("auto", ""), "auto");
    }

    #[test]
    fn a_thai_recording_on_auto_gets_thai_headings() {
        let mut s = settings("auto");
        s.language = resolve_language(&s.language, "th");
        let prompt = single_pass_system(&s, "");

        assert!(prompt.contains("## ภาพรวม"));
        assert!(prompt.contains("Write the summary in Thai"));
    }

    #[test]
    fn auto_asks_for_the_meetings_own_language() {
        let prompt = reduce_system(&settings("auto"), "");
        assert!(prompt.contains("the language the meeting was held in"));
    }

    #[test]
    fn vocabulary_and_instructions_are_appended() {
        let mut s = settings("th");
        s.instructions = "เน้นงานที่ต้องทำ".into();
        let prompt = map_system(&s, "Velo, libmpv");

        assert!(prompt.contains("Velo, libmpv"));
        assert!(prompt.contains("เน้นงานที่ต้องทำ"));
    }

    #[test]
    fn the_reminders_show_the_format_they_ask_for() {
        assert!(transcript_reminder().contains("[12:34]"));
        assert!(notes_reminder().contains("[12:34]"));
    }

    #[test]
    fn empty_extras_add_nothing() {
        let prompt = map_system(&settings("en"), "   ");
        assert!(!prompt.contains("Terms used by this team"));
        assert!(!prompt.contains("Additional instructions"));
    }
}
