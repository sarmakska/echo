/// Split text into sentence-ish chunks for sentence-by-sentence TTS streaming
/// (PLAN.md §4.4). Splits after '.', '!', '?' followed by whitespace/end,
/// preserving the terminator. Trailing text without a terminator is its own chunk.
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        cur.push(c);
        let is_terminator = matches!(c, '.' | '!' | '?');
        let next_is_boundary = chars.get(i + 1).map_or(true, |n| n.is_whitespace());
        if is_terminator && next_is_boundary {
            let trimmed = cur.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
            cur.clear();
        }
    }
    let tail = cur.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_multiple_sentences() {
        let s = split_sentences("Hello there. How are you? I am well!");
        assert_eq!(s, vec!["Hello there.", "How are you?", "I am well!"]);
    }

    #[test]
    fn keeps_unterminated_tail() {
        assert_eq!(split_sentences("no terminator here"), vec!["no terminator here"]);
    }

    #[test]
    fn empty_input_yields_empty() {
        assert!(split_sentences("   ").is_empty());
    }

    #[test]
    fn does_not_split_decimal_without_following_space() {
        // "3.5" has no whitespace after the dot, so it stays in one chunk.
        assert_eq!(split_sentences("It is 3.5 today"), vec!["It is 3.5 today"]);
    }
}
