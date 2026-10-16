//! Summary templates: a short brief that changes the focus and headings of the
//! AI meeting summary (standup, client call, interview, ...). The chosen
//! template is appended to the summary system prompt, the same way standing
//! rules from `user_rules.rs` are.
//!
//! The summary keeps its fixed JSON keys so the app can always show and export
//! it. A template shapes what goes into those keys.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Id of "no template": the plain default summary.
pub const DEFAULT_TEMPLATE_ID: &str = "default";

/// Longest template brief sent to the model, in characters.
const MAX_INSTRUCTION_CHARS: usize = 1_200;
/// Most custom templates kept in settings.
pub const MAX_CUSTOM_TEMPLATES: usize = 30;

/// A template the user wrote themselves. Stored in settings.
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
pub struct SummaryTemplate {
    pub id: String,
    pub name: String,
    /// What the summary should focus on and how to head its sections.
    pub instructions: String,
}

/// A template that ships with the app.
pub struct BuiltinTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub instructions: &'static str,
}

pub const BUILTIN_TEMPLATES: &[BuiltinTemplate] = &[
    BuiltinTemplate {
        id: "standup",
        name: "Standup",
        instructions: "This is a team standup. In keyPoints write one entry per person, in the form \"Name: done, doing next\". \
Put every blocker in openQuestions, starting with \"Blocker:\". Put follow-ups in actionItems with the owner. Keep the summary to two or three sentences.",
    },
    BuiltinTemplate {
        id: "client_call",
        name: "Client call",
        instructions: "This is a call with a client. In keyPoints start each entry with a heading word, one of \"Request:\", \"Feedback:\", \"Concern:\" or \"Update:\". \
Put agreements, scope, price and deadline changes in decisions. Put every promise made by either side in actionItems with the owner. Put anything the client still needs to confirm in openQuestions.",
    },
    BuiltinTemplate {
        id: "interview",
        name: "Interview",
        instructions: "This is a job interview. In keyPoints start each entry with a heading word, one of \"Experience:\", \"Skills:\", \"Strength:\", \"Concern:\" or \"Motivation:\". \
Do not judge the candidate beyond what was said. Put the agreed next steps in decisions and the follow-up tasks in actionItems. Put topics that were not covered in openQuestions.",
    },
    BuiltinTemplate {
        id: "lecture",
        name: "Lecture",
        instructions: "This is a lecture or class. Write the summary as a short overview of the topic. In keyPoints list the main concepts in the order taught, each starting with the concept name and a colon, then a one line explanation. \
Put definitions, formulas and examples worth remembering in decisions. Put homework, readings and deadlines in actionItems. Put questions raised and left open in openQuestions.",
    },
    BuiltinTemplate {
        id: "sales_call",
        name: "Sales call",
        instructions: "This is a sales call. In keyPoints start each entry with a heading word, one of \"Need:\", \"Budget:\", \"Timeline:\", \"Decision maker:\", \"Objection:\" or \"Competitor:\", only for things actually said. \
Put the agreed next step and any commitment in decisions. Put follow-ups in actionItems with the owner and date. Put unanswered objections and missing information in openQuestions.",
    },
];

/// The brief for `selected_id`: a custom template first, then a built-in one.
/// None for the default or an id that no longer exists.
fn resolve<'a>(custom: &'a [SummaryTemplate], selected_id: &str) -> Option<(&'a str, &'a str)> {
    if selected_id.is_empty() || selected_id == DEFAULT_TEMPLATE_ID {
        return None;
    }
    if let Some(t) = custom.iter().find(|t| t.id == selected_id) {
        return Some((t.name.as_str(), t.instructions.as_str()));
    }
    BUILTIN_TEMPLATES
        .iter()
        .find(|t| t.id == selected_id)
        .map(|t| (t.name, t.instructions))
}

/// Text to append to the summary system prompt. Empty for the default
/// template, an unknown id or a custom template with no instructions.
pub fn instructions(custom: &[SummaryTemplate], selected_id: &str) -> String {
    let Some((name, text)) = resolve(custom, selected_id) else {
        return String::new();
    };
    let text: String = text.trim().chars().take(MAX_INSTRUCTION_CHARS).collect();
    if text.is_empty() {
        return String::new();
    }
    format!(
        "\n\nSummary template \"{}\". Keep the JSON keys and format above, and shape what goes into them like this:\n{}",
        name.trim(),
        text
    )
}

/// Trim ids and names, drop entries with no id and cap the count. Used when
/// the UI saves the list. Unnamed drafts are kept so a half-written template
/// survives a save; the UI hides them from the picker.
pub fn sanitize(templates: Vec<SummaryTemplate>) -> Vec<SummaryTemplate> {
    templates
        .into_iter()
        .map(|t| SummaryTemplate {
            id: t.id.trim().to_string(),
            name: t.name.trim().to_string(),
            instructions: t.instructions,
        })
        .filter(|t| !t.id.is_empty())
        .take(MAX_CUSTOM_TEMPLATES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom(id: &str, name: &str, text: &str) -> SummaryTemplate {
        SummaryTemplate {
            id: id.into(),
            name: name.into(),
            instructions: text.into(),
        }
    }

    #[test]
    fn default_and_unknown_add_nothing() {
        assert_eq!(instructions(&[], DEFAULT_TEMPLATE_ID), "");
        assert_eq!(instructions(&[], ""), "");
        assert_eq!(instructions(&[], "deleted_template"), "");
    }

    #[test]
    fn five_builtins_with_unique_ids_and_text() {
        assert_eq!(BUILTIN_TEMPLATES.len(), 5);
        let mut ids: Vec<_> = BUILTIN_TEMPLATES.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5);
        for t in BUILTIN_TEMPLATES {
            assert!(t.instructions.len() > 50, "{} is too short", t.id);
            assert!(t.instructions.len() <= MAX_INSTRUCTION_CHARS);
            assert!(!t.id.contains("custom"));
            assert_ne!(t.id, DEFAULT_TEMPLATE_ID);
            let out = instructions(&[], t.id);
            assert!(out.contains(t.name));
            assert!(out.contains("Keep the JSON keys"));
        }
    }

    #[test]
    fn custom_template_is_used_and_beats_builtin_with_same_id() {
        let list = vec![
            custom(
                "custom_1",
                "Board meeting",
                "Focus on votes and resolutions.",
            ),
            custom("standup", "My standup", "Only list blockers."),
        ];
        let out = instructions(&list, "custom_1");
        assert!(out.contains("\"Board meeting\""));
        assert!(out.contains("Focus on votes and resolutions."));
        let out = instructions(&list, "standup");
        assert!(out.contains("Only list blockers."));
        assert!(!out.contains("team standup"));
    }

    #[test]
    fn empty_custom_instructions_add_nothing() {
        let list = vec![custom("custom_1", "Blank", "   ")];
        assert_eq!(instructions(&list, "custom_1"), "");
    }

    #[test]
    fn long_instructions_are_capped() {
        let list = vec![custom("custom_1", "Long", &"x".repeat(5_000))];
        let out = instructions(&list, "custom_1");
        assert!(out.matches('x').count() == MAX_INSTRUCTION_CHARS);
    }

    #[test]
    fn sanitize_trims_keeps_drafts_and_caps_count() {
        let mut list = vec![
            custom("a", "  Named ", "x"),
            custom("b", "   ", "x"),
            custom("", "No id", "x"),
        ];
        let cleaned = sanitize(list.clone());
        assert_eq!(cleaned.len(), 2); // the entry with no id is dropped
        assert_eq!(cleaned[0].name, "Named");
        assert_eq!(cleaned[1].name, "");

        list = (0..50)
            .map(|i| custom(&format!("c{i}"), "n", "x"))
            .collect();
        assert_eq!(sanitize(list).len(), MAX_CUSTOM_TEMPLATES);
    }
}
