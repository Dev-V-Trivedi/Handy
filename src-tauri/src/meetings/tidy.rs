//! Make a raw meeting transcript read like a conversation.
//!
//! Capture works in chunks of 10 to 28 seconds, so one sentence is often cut
//! in two and the two speakers alternate in small pieces. For reading, export
//! and summaries we join consecutive chunks from the same speaker into turns,
//! fix spacing, capitalise sentence starts and close the turn with a full stop.
//! The raw chunks stay untouched in the database.

use super::store::MeetingSegment;

/// A pause longer than this (milliseconds) between chunks from the same
/// speaker starts a new turn.
const TURN_GAP_MS: i64 = 45_000;

fn is_terminal(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…' | '。' | '！' | '？' | '।' | '؟')
}

/// Collapse whitespace, remove spaces before punctuation, capitalise the
/// start of each sentence.
pub fn tidy_text(input: &str) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(collapsed.len());
    let mut start_of_sentence = true;
    for c in collapsed.chars() {
        if c == ' ' && out.ends_with(' ') {
            continue;
        }
        if matches!(c, ',' | '.' | ';' | ':' | '!' | '?') && out.ends_with(' ') {
            out.pop();
        }
        if start_of_sentence && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            start_of_sentence = false;
        } else {
            out.push(c);
            if c.is_alphanumeric() {
                start_of_sentence = false;
            }
        }
        if is_terminal(c) {
            start_of_sentence = true;
        }
    }
    out.trim().to_string()
}

fn close_turn(text: &mut String) {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return;
    }
    let last = trimmed.chars().last().unwrap_or(' ');
    if !is_terminal(last) && !matches!(last, '"' | ')' | '\'') {
        *text = format!("{trimmed}.");
    } else {
        *text = trimmed.to_string();
    }
}

/// Join consecutive chunks from the same speaker into tidy turns.
pub fn merge_turns(segments: Vec<MeetingSegment>) -> Vec<MeetingSegment> {
    let mut out: Vec<MeetingSegment> = Vec::new();
    let mut last_start = 0i64;
    for seg in segments {
        let text = seg.text.trim();
        if text.is_empty() {
            continue;
        }
        match out.last_mut() {
            Some(prev) if prev.track == seg.track && seg.start_ms - last_start <= TURN_GAP_MS => {
                prev.text.push(' ');
                prev.text.push_str(text);
            }
            _ => out.push(MeetingSegment {
                track: seg.track.clone(),
                start_ms: seg.start_ms,
                text: text.to_string(),
            }),
        }
        last_start = seg.start_ms;
    }
    for seg in &mut out {
        seg.text = tidy_text(&seg.text);
        close_turn(&mut seg.text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(track: &str, start_ms: i64, text: &str) -> MeetingSegment {
        MeetingSegment {
            track: track.to_string(),
            start_ms,
            text: text.to_string(),
        }
    }

    #[test]
    fn spacing_and_capitals_are_fixed() {
        assert_eq!(
            tidy_text("  hello  there ,  how are you ? i am fine. great"),
            "Hello there, how are you? I am fine. Great"
        );
    }

    #[test]
    fn same_speaker_chunks_join_into_one_turn() {
        let turns = merge_turns(vec![
            seg("you", 0, "so the plan for next quarter is"),
            seg("you", 12_000, "to ship the new onboarding"),
            seg("others", 25_000, "sounds good"),
            seg("you", 30_000, "great. let's do it."),
        ]);
        assert_eq!(turns.len(), 3);
        assert_eq!(
            turns[0].text,
            "So the plan for next quarter is to ship the new onboarding."
        );
        assert_eq!(turns[1].text, "Sounds good.");
        assert_eq!(turns[2].text, "Great. Let's do it.");
    }

    #[test]
    fn long_silence_starts_a_new_turn() {
        let turns = merge_turns(vec![seg("you", 0, "first"), seg("you", 120_000, "second")]);
        assert_eq!(turns.len(), 2);
    }

    #[test]
    fn empty_chunks_are_dropped() {
        assert!(merge_turns(vec![seg("you", 0, "   ")]).is_empty());
    }

    #[test]
    fn non_latin_text_is_untouched_apart_from_spacing() {
        let turns = merge_turns(vec![seg("others", 0, "नमस्ते  दोस्तों")]);
        assert_eq!(turns[0].text, "नमस्ते दोस्तों.");
    }
}
