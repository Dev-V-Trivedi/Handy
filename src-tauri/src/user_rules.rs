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
}
