//! Formatting applied to dictated text: the always-on sentence finish (a
//! capital first letter and closing punctuation), and the check that a local
//! LLM's "format only" reply kept the dictated words.

/// Scripts whose sentences end with no mark at all.
fn ends_without_mark(c: char) -> bool {
    matches!(c, '\u{0E00}'..='\u{0E7F}' | '\u{0E80}'..='\u{0EFF}') // Thai, Lao
}

/// Han, kana (and their fullwidth forms) close with the ideographic full stop.
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{30FF}'   // Hiragana, Katakana
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A
        | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{FF66}'..='\u{FF9F}') // Halfwidth Katakana
}

fn is_devanagari(c: char) -> bool {
    matches!(c, '\u{0900}'..='\u{097F}')
}

/// The text as one finished sentence: its first letter capitalised (in
/// scripts that have capitals) and a closing mark added when it ends on a
/// letter or digit. Text already ending in any punctuation, a quote or a
/// bracket is left as it ends.
pub fn finish_sentence(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(trimmed.len() + 3);
    // Capitalise the first letter, past any opening quotes or brackets, but
    // only when it starts a lowercase word (leaves "iPhone" alone).
    let mut capitalised = false;
    let chars: Vec<char> = trimmed.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if !capitalised && c.is_alphanumeric() {
            capitalised = true;
            let next_is_upper = chars.get(i + 1).is_some_and(|n| n.is_uppercase());
            if c.is_lowercase() && !next_is_upper {
                out.extend(c.to_uppercase());
                continue;
            }
        }
        out.push(c);
    }

    let last = chars[chars.len() - 1];
    if last.is_alphanumeric() && !ends_without_mark(last) {
        out.push(if is_cjk(last) {
            '。'
        } else if is_devanagari(last) {
            '।'
        } else {
            '.'
        });
    }
    out
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Whether `output` only reformats `input`: (almost) every word it contains
/// was dictated, so it neither translated, answered nor added anything, and
/// it kept most of the dictated words (list formatting may drop the spoken
/// "first / second / third").
pub fn is_faithful_reformat(input: &str, output: &str) -> bool {
    let dictated = words(input);
    let written = words(output);
    if dictated.is_empty() || written.is_empty() {
        return false;
    }
    let dictated_set: std::collections::HashSet<&str> =
        dictated.iter().map(String::as_str).collect();
    let written_set: std::collections::HashSet<&str> = written.iter().map(String::as_str).collect();

    let added = written
        .iter()
        .filter(|w| !dictated_set.contains(w.as_str()))
        .count();
    let kept = dictated
        .iter()
        .filter(|w| written_set.contains(w.as_str()))
        .count();

    // Allow one stray word (a fixed spelling) per ten written.
    let added_ok = added * 10 <= written.len();
    let kept_ok = kept * 10 >= dictated.len() * 6;
    added_ok && kept_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_a_period_and_a_capital_to_a_bare_sentence() {
        assert_eq!(
            finish_sentence("so we ship it on thursday"),
            "So we ship it on thursday."
        );
    }

    #[test]
    fn keeps_existing_closing_punctuation() {
        for text in [
            "Done.",
            "Really?",
            "Wow!",
            "Wait…",
            "He said \"go.\"",
            "(See below)",
            "Items:",
        ] {
            assert_eq!(finish_sentence(text), text, "{text}");
        }
    }

    #[test]
    fn handles_cyrillic_hebrew_and_digits() {
        assert_eq!(finish_sentence("отправь отчёт"), "Отправь отчёт.");
        // Hebrew has no capitals; it still gets its period.
        assert_eq!(finish_sentence("שלח את הדוח"), "שלח את הדוח.");
        assert_eq!(finish_sentence("we need 25"), "We need 25.");
    }

    #[test]
    fn uses_each_scripts_own_full_stop() {
        assert_eq!(finish_sentence("我们明天发布"), "我们明天发布。");
        assert_eq!(
            finish_sentence("明日リリースします"),
            "明日リリースします。"
        );
        assert_eq!(finish_sentence("मैं कल आऊंगा"), "मैं कल आऊंगा।");
        assert_eq!(finish_sentence("ไปตลาด"), "ไปตลาด");
    }

    #[test]
    fn leaves_mixed_case_words_and_blank_text_alone() {
        assert_eq!(finish_sentence("iPhone is charging"), "iPhone is charging.");
        assert_eq!(finish_sentence("\"quoted start"), "\"Quoted start.");
        assert_eq!(finish_sentence("   "), "   ");
        assert_eq!(finish_sentence(""), "");
    }

    #[test]
    fn a_list_reformat_is_faithful() {
        let input = "I need three things first update the README second rerun the tests third send the link to Raphael.";
        let output =
            "I need three things:\n- update the README\n- rerun the tests\n- send the link to Raphael";
        assert!(is_faithful_reformat(input, output));

        let ru = "Мне нужно три вещи. Первый – обновить документацию. Второй – перезапустить тесты. Третий – отправить ссылку Рафаэлю.";
        let ru_out = "Мне нужно три вещи:\n- обновить документацию\n- перезапустить тесты\n- отправить ссылку Рафаэлю";
        assert!(is_faithful_reformat(ru, ru_out));
    }

    /// The local 1.5B model once answered a Russian list in English.
    #[test]
    fn a_translation_is_not_faithful() {
        let ru = "Мне нужно три вещи. Первый – обновить документацию. Второй – перезапустить тесты. Третий – отправить ссылку Рафаэлю.";
        let en =
            "First - update documentation. Second - restart tests. Third - send link to Rafael.";
        assert!(!is_faithful_reformat(ru, en));
    }

    #[test]
    fn an_answer_or_a_rewrite_is_not_faithful() {
        let question = "what time is the meeting tomorrow";
        assert!(is_faithful_reformat(
            question,
            "What time is the meeting tomorrow?"
        ));
        assert!(!is_faithful_reformat(
            question,
            "The meeting tomorrow is at 10 AM in the main conference room."
        ));
        assert!(!is_faithful_reformat(
            "ship it",
            "Sure! Here is the text: Ship it."
        ));
        assert!(!is_faithful_reformat("ship it", ""));
    }

    #[test]
    fn dropping_most_words_is_not_faithful() {
        let input = "so I talked to Danny about the release and we agreed to ship it on Thursday";
        assert!(!is_faithful_reformat(input, "Ship on Thursday."));
    }
}
