//! Meeting summary: key points, decisions and action items.
//!
//! Uses the same provider, model and API key the user already set up for
//! dictation cleanup (a local Ollama model, a cloud key, and so on). If nothing
//! is set up, the transcript is still saved and the UI explains how to enable
//! summaries.

use super::store::{MeetingDetail, MeetingStore};
use crate::settings::{get_settings, APPLE_INTELLIGENCE_PROVIDER_ID};
use log::{debug, warn};
use serde_json::{json, Value};
use tauri::AppHandle;

/// Above this many characters the transcript is summarised in parts first.
const SINGLE_PASS_LIMIT: usize = 12_000;
const PART_SIZE: usize = 9_000;

const SYSTEM_PROMPT: &str = "You write meeting notes from a transcript. \
Lines starting with \"You\" are the person who recorded the meeting. Lines starting with \"Others\" are everyone else, whose names are unknown unless said aloud. \
Use only what was said. Never invent names, dates, numbers or tasks. Write in the language the meeting used. \
Return ONLY a JSON object with these keys: \
\"title\" (short, under 8 words), \
\"summary\" (3 to 6 sentences), \
\"keyPoints\" (array of short strings), \
\"decisions\" (array of short strings), \
\"actionItems\" (array of objects with \"task\", \"owner\" (string or null) and \"due\" (string or null)), \
\"openQuestions\" (array of short strings). \
Use empty arrays when there is nothing for a key.";

const PART_PROMPT: &str = "You are given one part of a longer meeting transcript. \
Write concise bullet notes: key points, decisions, and action items with owner and due date when stated. \
Use only what was said. Write in the language the meeting used. Plain text bullets only.";

fn format_clock(ms: i64) -> String {
    let secs = (ms / 1000).max(0);
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

/// "[mm:ss] You: text" lines, in time order.
pub fn transcript_text(detail: &MeetingDetail) -> String {
    detail
        .segments
        .iter()
        .map(|s| {
            let who = if s.track == "you" { "You" } else { "Others" };
            format!("[{}] {}: {}", format_clock(s.start_ms), who, s.text.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_parts(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if !current.is_empty() && current.len() + line.len() + 1 > PART_SIZE {
            parts.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

fn strip_think(s: &str) -> &str {
    if let Some(rest) = s.trim_start().strip_prefix("<think>") {
        if let Some(end) = rest.find("</think>") {
            return rest[end + "</think>".len()..].trim_start();
        }
    }
    s
}

/// Pull a JSON object out of a model reply, tolerating code fences and chatter.
pub fn parse_summary(reply: &str) -> Value {
    let text = strip_think(reply).trim();
    if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) {
        if start < end {
            if let Ok(Value::Object(mut obj)) = serde_json::from_str::<Value>(&text[start..=end]) {
                for key in ["keyPoints", "decisions", "actionItems", "openQuestions"] {
                    obj.entry(key).or_insert_with(|| json!([]));
                }
                obj.entry("summary").or_insert_with(|| json!(""));
                return Value::Object(obj);
            }
        }
    }
    // The model ignored the format. Keep what it said as the summary.
    json!({
        "summary": text,
        "keyPoints": [],
        "decisions": [],
        "actionItems": [],
        "openQuestions": []
    })
}

async fn ask(
    app: &AppHandle,
    system: &str,
    user: String,
    want_json: bool,
) -> Result<String, String> {
    let settings = get_settings(app);
    let system = format!(
        "{system}{}",
        crate::user_rules::instructions(&settings.user_rules, crate::user_rules::SCOPE_MEETINGS)
    );
    let system = system.as_str();
    let provider = settings
        .active_post_process_provider()
        .cloned()
        .ok_or_else(|| "no_provider".to_string())?;
    if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
        return Err("apple_unsupported".to_string());
    }
    let model = settings
        .post_process_models
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();
    if model.trim().is_empty() {
        return Err("no_model".to_string());
    }
    let api_key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();
    crate::local_llm::prepare(app, &settings).await?;
    let disable_reasoning = matches!(
        provider.id.as_str(),
        "custom" | "ollama" | "openrouter" | "oshabe_local"
    );

    let schema = if want_json && provider.supports_structured_output {
        Some(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "summary": {"type": "string"},
                "keyPoints": {"type": "array", "items": {"type": "string"}},
                "decisions": {"type": "array", "items": {"type": "string"}},
                "actionItems": {"type": "array", "items": {
                    "type": "object",
                    "properties": {
                        "task": {"type": "string"},
                        "owner": {"type": ["string", "null"]},
                        "due": {"type": ["string", "null"]}
                    },
                    "required": ["task"]
                }},
                "openQuestions": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["title", "summary", "keyPoints", "decisions", "actionItems", "openQuestions"],
            "additionalProperties": false
        }))
    } else {
        None
    };

    match crate::llm_client::send_chat_completion_with_schema(
        &provider,
        api_key,
        &model,
        user,
        Some(system.to_string()),
        schema,
        disable_reasoning,
    )
    .await
    {
        Ok(Some(content)) => Ok(content),
        Ok(None) => Err("The model returned nothing".to_string()),
        Err(e) => Err(e),
    }
}

/// Build and store the summary for a finished meeting.
pub async fn summarize(app: &AppHandle, store: &MeetingStore, id: i64) -> Result<(), String> {
    let detail = store
        .get(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Meeting not found".to_string())?;
    if detail.segments.is_empty() {
        return Err("no_transcript".to_string());
    }

    let transcript = transcript_text(&detail);
    debug!("Summarising meeting {} ({} chars)", id, transcript.len());

    let input = if transcript.len() <= SINGLE_PASS_LIMIT {
        transcript
    } else {
        let mut notes = Vec::new();
        for (i, part) in split_parts(&transcript).into_iter().enumerate() {
            match ask(app, PART_PROMPT, part, false).await {
                Ok(text) => notes.push(format!("Part {}:\n{}", i + 1, strip_think(&text).trim())),
                Err(e) => {
                    warn!("Summary part {} failed: {}", i + 1, e);
                    return Err(e);
                }
            }
        }
        format!(
            "These are notes from consecutive parts of one meeting:\n\n{}",
            notes.join("\n\n")
        )
    };

    let reply = ask(app, SYSTEM_PROMPT, input, true).await?;
    let summary = parse_summary(&reply);
    let title = summary
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_string);
    store
        .set_summary(id, &summary.to_string(), title.as_deref())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meetings::store::MeetingSegment;

    #[test]
    fn parses_clean_json() {
        let v = parse_summary(
            r#"{"title":"Sync","summary":"We met.","keyPoints":["a"],"decisions":[],"actionItems":[{"task":"Send deck","owner":"Dev","due":null}],"openQuestions":[]}"#,
        );
        assert_eq!(v["title"], "Sync");
        assert_eq!(v["actionItems"][0]["owner"], "Dev");
    }

    #[test]
    fn parses_fenced_json_with_chatter() {
        let v = parse_summary(
            "Sure! Here you go:\n```json\n{\"summary\":\"ok\"}\n```\nHope that helps",
        );
        assert_eq!(v["summary"], "ok");
        assert!(v["actionItems"].is_array());
    }

    #[test]
    fn strips_think_blocks() {
        let v = parse_summary("<think>hmm</think>{\"summary\":\"done\"}");
        assert_eq!(v["summary"], "done");
    }

    #[test]
    fn falls_back_to_plain_text() {
        let v = parse_summary("They agreed to ship on Friday.");
        assert_eq!(v["summary"], "They agreed to ship on Friday.");
        assert!(v["decisions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn transcript_lines_are_labelled() {
        let detail = MeetingDetail {
            id: 1,
            title: "t".into(),
            started_at: 0,
            ended_at: None,
            summary: None,
            segments: vec![
                MeetingSegment {
                    track: "you".into(),
                    start_ms: 5_000,
                    text: "Hello".into(),
                },
                MeetingSegment {
                    track: "others".into(),
                    start_ms: 65_000,
                    text: "Hi".into(),
                },
            ],
        };
        assert_eq!(
            transcript_text(&detail),
            "[00:05] You: Hello\n[01:05] Others: Hi"
        );
    }

    #[test]
    fn long_transcripts_split_on_line_boundaries() {
        let line = "x".repeat(1_000);
        let text = vec![line; 25].join("\n");
        let parts = split_parts(&text);
        assert!(parts.len() > 1);
        assert!(parts.iter().all(|p| p.len() <= PART_SIZE));
        assert_eq!(parts.join("\n").len(), text.len());
    }
}
