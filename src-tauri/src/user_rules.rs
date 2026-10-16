//! Standing instructions the user wants the AI to always follow ("use bullet
//! points", "keep summaries short"). They are appended to the system prompt of
//! every AI step that matches the rule's scope.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Where a rule applies.
pub const SCOPE_ALL: &str = "all";
pub const SCOPE_MEETINGS: &str = "meetings";
pub const SCOPE_DICTATION: &str = "dictation";

/// Most rules sent to the model. More only confuses small local models.
const MAX_RULES: usize = 20;
/// Longest single rule, in characters.
const MAX_RULE_CHARS: usize = 300;

#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
pub struct UserRule {
    pub id: String,
    pub text: String,
    /// `all`, `meetings` or `dictation`.
    pub scope: String,
    pub enabled: bool,
}

/// Text to append to a system prompt for `scope`. Empty when no rule applies.
pub fn instructions(rules: &[UserRule], scope: &str) -> String {
    let lines: Vec<String> = rules
        .iter()
        .filter(|r| r.enabled && (r.scope == SCOPE_ALL || r.scope == scope))
        .map(|r| {
            r.text
                .trim()
                .chars()
                .take(MAX_RULE_CHARS)
                .collect::<String>()
        })
        .filter(|t| !t.is_empty())
        .take(MAX_RULES)
        .map(|t| format!("- {t}"))
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    format!(
        "\n\nThe user has standing preferences. Follow them in your output, unless they conflict with the task above:\n{}",
        lines.join("\n")
    )
}

/// Most rules kept in settings. Only the first `MAX_RULES` enabled ones are
/// sent to the model, but a hard stop keeps voice commands from piling up.
const MAX_STORED_RULES: usize = 50;
/// Shortest spoken rule worth saving, in non-space characters.
const MIN_REMEMBER_CHARS: usize = 3;

/// If `s` starts with `word` (ASCII, any case) as a whole word, the text after it.
fn strip_word_ci<'a>(s: &'a str, word: &str) -> Option<&'a str> {
    let (sb, wb) = (s.as_bytes(), word.as_bytes());
    if sb.len() < wb.len() || !sb[..wb.len()].eq_ignore_ascii_case(wb) {
        return None;
    }
    // The matched prefix is ASCII, so this is a char boundary.
    let rest = &s[wb.len()..];
    if rest.chars().next().is_some_and(char::is_alphanumeric) {
        return None;
    }
    Some(rest)
}

fn is_separator(c: char) -> bool {
    matches!(c, ':' | ',' | '-' | ';' | '\u{ff1a}' | '\u{2013}')
}

/// Detect a spoken "remember" command and return the rule text.
///
/// Accepted forms (any capitalisation):
/// - `Oshabe remember always use bullet points` (also `Oshabe, remember: ...`
///   and `Oshabe remember that ...`)
/// - `Remember: always use bullet points` (also `Remember that: ...`)
///
/// Without the `Oshabe` wake word a colon is required, so ordinary dictation
/// such as "Remember to call Sam" is never swallowed.
pub fn extract_remember(text: &str) -> Option<String> {
    let t = text.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '"' | '\''));
    let (t, woke) = match strip_word_ci(t, "oshabe") {
        Some(rest) => (
            rest.trim_start_matches(|c: char| c.is_whitespace() || is_separator(c)),
            true,
        ),
        None => (t, false),
    };
    let after = strip_word_ci(t, "remember")?.trim_start();
    let after = strip_word_ci(after, "that")
        .or_else(|| strip_word_ci(after, "this"))
        .map(str::trim_start)
        .unwrap_or(after);
    let has_separator = after.chars().next().is_some_and(is_separator);
    if !woke && !has_separator {
        return None;
    }
    let body = after
        .trim_start_matches(|c: char| c.is_whitespace() || is_separator(c))
        .trim();
    if body.chars().filter(|c| !c.is_whitespace()).count() < MIN_REMEMBER_CHARS {
        return None;
    }
    let mut chars = body.chars();
    let first = chars.next()?;
    let capped: String = first
        .to_uppercase()
        .chain(chars)
        .take(MAX_RULE_CHARS)
        .collect();
    Some(capped)
}

/// What happened when a spoken rule was saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RememberOutcome {
    Added,
    /// The same rule is already saved.
    Duplicate,
    /// Too many rules saved already.
    Full,
}

/// Add `text` as an enabled rule with scope `all`, unless it is already there.
pub fn add_rule(rules: &mut Vec<UserRule>, text: &str, id: String) -> RememberOutcome {
    let wanted = text.trim().to_lowercase();
    if rules.iter().any(|r| r.text.trim().to_lowercase() == wanted) {
        return RememberOutcome::Duplicate;
    }
    if rules.len() >= MAX_STORED_RULES {
        return RememberOutcome::Full;
    }
    rules.push(UserRule {
        id,
        text: text.trim().to_string(),
        scope: SCOPE_ALL.to_string(),
        enabled: true,
    });
    RememberOutcome::Added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(text: &str, scope: &str, enabled: bool) -> UserRule {
        UserRule {
            id: text.to_string(),
            text: text.to_string(),
            scope: scope.to_string(),
            enabled,
        }
    }

    #[test]
    fn empty_when_nothing_applies() {
        assert_eq!(instructions(&[], SCOPE_MEETINGS), "");
        let rules = vec![rule("Use bullet points", SCOPE_DICTATION, true)];
        assert_eq!(instructions(&rules, SCOPE_MEETINGS), "");
    }

    #[test]
    fn scope_and_enabled_are_respected() {
        let rules = vec![
            rule("Use bullet points", SCOPE_MEETINGS, true),
            rule("Be formal", SCOPE_ALL, true),
            rule("Skip names", SCOPE_MEETINGS, false),
            rule("   ", SCOPE_ALL, true),
        ];
        let out = instructions(&rules, SCOPE_MEETINGS);
        assert!(out.contains("- Use bullet points"));
        assert!(out.contains("- Be formal"));
        assert!(!out.contains("Skip names"));
        assert_eq!(out.matches("\n- ").count(), 2);
    }

    #[test]
    fn rule_count_is_capped() {
        let rules: Vec<UserRule> = (0..40)
            .map(|i| rule(&format!("rule {i}"), SCOPE_ALL, true))
            .collect();
        assert_eq!(
            instructions(&rules, SCOPE_DICTATION)
                .matches("\n- ")
                .count(),
            20
        );
    }

    #[test]
    fn remember_with_colon_is_detected() {
        assert_eq!(
            extract_remember("remember: always use bullet points").as_deref(),
            Some("Always use bullet points")
        );
        assert_eq!(
            extract_remember("  Remember: keep it short.").as_deref(),
            Some("Keep it short.")
        );
        assert_eq!(
            extract_remember("REMEMBER THAT: no emojis").as_deref(),
            Some("No emojis")
        );
    }

    #[test]
    fn wake_word_needs_no_colon() {
        assert_eq!(
            extract_remember("Oshabe remember always write in British English").as_deref(),
            Some("Always write in British English")
        );
        assert_eq!(
            extract_remember("Oshabe, remember that I prefer short emails").as_deref(),
            Some("I prefer short emails")
        );
        assert_eq!(
            extract_remember("oshabe remember: use bullet points").as_deref(),
            Some("Use bullet points")
        );
    }

    #[test]
    fn ordinary_dictation_is_not_a_command() {
        // No colon and no wake word.
        assert_eq!(extract_remember("Remember to call Sam tomorrow"), None);
        assert_eq!(extract_remember("remember that the meeting is at 5"), None);
        // "remember" must be a whole word at the start.
        assert_eq!(extract_remember("Remembering: the old days"), None);
        assert_eq!(extract_remember("I must remember: milk"), None);
        // Wake word alone, or nothing to remember.
        assert_eq!(extract_remember("Oshabe remember"), None);
        assert_eq!(extract_remember("remember:"), None);
        assert_eq!(extract_remember("remember: ok"), None);
        assert_eq!(extract_remember("Oshabes remember this: hi there"), None);
        assert_eq!(extract_remember(""), None);
    }

    #[test]
    fn spoken_rule_is_capped_and_handles_unicode() {
        let long = format!("remember: {}", "a".repeat(1_000));
        assert_eq!(extract_remember(&long).unwrap().chars().count(), 300);
        assert_eq!(
            extract_remember("remember: \u{e9}crire en fran\u{e7}ais").as_deref(),
            Some("\u{c9}crire en fran\u{e7}ais")
        );
    }

    #[test]
    fn add_rule_saves_with_scope_all_and_skips_duplicates() {
        let mut rules = vec![rule("Be formal", SCOPE_MEETINGS, true)];
        assert_eq!(
            add_rule(&mut rules, "Use bullet points", "urule_1".into()),
            RememberOutcome::Added
        );
        let added = rules.last().unwrap();
        assert_eq!(added.scope, SCOPE_ALL);
        assert!(added.enabled);
        assert_eq!(added.id, "urule_1");
        assert_eq!(
            add_rule(&mut rules, "  use BULLET points ", "urule_2".into()),
            RememberOutcome::Duplicate
        );
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn add_rule_stops_at_the_limit() {
        let mut rules: Vec<UserRule> = (0..MAX_STORED_RULES)
            .map(|i| rule(&format!("rule {i}"), SCOPE_ALL, true))
            .collect();
        assert_eq!(
            add_rule(&mut rules, "one more", "x".into()),
            RememberOutcome::Full
        );
        assert_eq!(rules.len(), MAX_STORED_RULES);
    }
}
