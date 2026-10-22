use crate::storage::settings_store::{SummaryFormat, SummarySettings};

pub fn words(text: &str) -> usize {
    icu_segmenter::WordSegmenter::new_dictionary(Default::default())
        .segment_str(text)
        .iter_with_word_type()
        .filter(|(_, kind)| kind.is_word_like())
        .count()
}

// Recognize explicit numeric word ceilings, never infer one from a duration,
// transcript, bare number or a request for "short" text. Other prose stays focus.
pub fn ceiling(settings: &SummarySettings) -> Option<usize> {
    if settings.format != SummaryFormat::Custom {
        return None;
    }
    let focus = settings.instructions.to_ascii_lowercase();
    [
        ("ไม่เกิน", "คำ"),
        ("at most", "word"),
        ("no more than", "word"),
        ("up to", "word"),
    ]
    .into_iter()
    .flat_map(|(prefix, unit)| {
        focus.split(prefix).skip(1).filter_map(move |tail| {
            let tail = tail.trim_start();
            let end = tail
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(tail.len());
            let n = tail[..end].parse::<usize>().ok()?;
            (n > 0 && tail[end..].trim_start().starts_with(unit)).then_some(n)
        })
    })
    .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_thai_words_without_spaces_and_not_punctuation() {
        assert_eq!(words("ทุกสองสัปดาห์"), 3);
        assert_eq!(words("“Keep records.”"), 2);
        assert_eq!(words("... “ ”"), 0);
    }
    #[test]
    fn only_explicit_custom_word_ceilings_constrain_selection() {
        let mut settings = SummarySettings {
            format: SummaryFormat::Custom,
            ..Default::default()
        };
        for (focus, expected) in [
            ("ย่อหน้าสั้น ไม่เกิน 150 คำ ไม่ต้องมีหัวข้อ", Some(150)),
            ("at most 20 words, no more than 10 words", Some(10)),
            ("up to 25 words", Some(25)),
            ("ประชุมไม่เกิน 150 นาที", None),
            ("สรุปสั้น ๆ", None),
            ("no more than 0 words", None),
        ] {
            settings.instructions = focus.into();
            assert_eq!(ceiling(&settings), expected);
        }
        settings.format = SummaryFormat::Meeting;
        settings.instructions = "ไม่เกิน 150 คำ".into();
        assert_eq!(ceiling(&settings), None);
    }
}
