//! One-click local cleanup models through Ollama.
//!
//! The settings screen shows a short list of recommended models. This module
//! talks to the user's local Ollama server (never anything remote unless the
//! user pointed the Ollama provider somewhere else) so non-technical users can
//! install a model with a button instead of a terminal.

use futures_util::StreamExt;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::settings::get_settings;

/// Event emitted while a model downloads.
const PROGRESS_EVENT: &str = "ollama-pull-progress";

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OllamaStatus {
    /// True when the Ollama server answered.
    pub running: bool,
    /// Names of models already installed (for example `qwen2.5:3b`).
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct OllamaPullProgress {
    pub model: String,
    pub status: String,
    pub completed: u64,
    pub total: u64,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Deserialize)]
struct PullLine {
    #[serde(default)]
    status: String,
    #[serde(default)]
    total: u64,
    #[serde(default)]
    completed: u64,
    #[serde(default)]
    error: Option<String>,
}

fn cancel_flags() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static FLAGS: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    FLAGS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Server root for the Ollama provider, without the OpenAI `/v1` suffix.
fn ollama_root(app: &AppHandle) -> String {
    let settings = get_settings(app);
    let base = settings
        .post_process_provider("ollama")
        .map(|p| p.base_url.clone())
        .unwrap_or_else(|| "http://localhost:11434/v1".to_string());
    let trimmed = base.trim_end_matches('/');
    trimmed
        .strip_suffix("/v1")
        .unwrap_or(trimmed)
        .trim_end_matches('/')
        .to_string()
}

/// Model names are passed to the server as data, but keep them to the
/// characters Ollama itself allows so nothing odd reaches the log or the UI.
fn valid_model_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '-'))
}

fn client(timeout: Option<Duration>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder();
    if let Some(t) = timeout {
        builder = builder.timeout(t);
    }
    builder.build().map_err(|e| e.to_string())
}

/// Is Ollama reachable, and which models does it already have?
#[tauri::command]
#[specta::specta]
pub async fn ollama_status(app: AppHandle) -> Result<OllamaStatus, String> {
    let url = format!("{}/api/tags", ollama_root(&app));
    let http = client(Some(Duration::from_secs(3)))?;
    match http.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let tags: TagsResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(OllamaStatus {
                running: true,
                models: tags.models.into_iter().map(|m| m.name).collect(),
            })
        }
        Ok(resp) => {
            debug!("Ollama status returned {}", resp.status());
            Ok(OllamaStatus {
                running: false,
                models: Vec::new(),
            })
        }
        Err(e) => {
            debug!("Ollama not reachable: {}", e);
            Ok(OllamaStatus {
                running: false,
                models: Vec::new(),
            })
        }
    }
}

/// Download a model, emitting `ollama-pull-progress` events as it goes.
#[tauri::command]
#[specta::specta]
pub async fn ollama_pull_model(app: AppHandle, model: String) -> Result<(), String> {
    if !valid_model_name(&model) {
        return Err("Invalid model name".to_string());
    }

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut flags = cancel_flags().lock().map_err(|_| "lock poisoned")?;
        if flags.contains_key(&model) {
            return Err("This model is already downloading".to_string());
        }
        flags.insert(model.clone(), Arc::clone(&cancel));
    }

    let result = pull_inner(&app, &model, &cancel).await;

    if let Ok(mut flags) = cancel_flags().lock() {
        flags.remove(&model);
    }
    result
}

async fn pull_inner(app: &AppHandle, model: &str, cancel: &AtomicBool) -> Result<(), String> {
    let url = format!("{}/api/pull", ollama_root(app));
    // No overall timeout: model files are large. The stream itself reports
    // progress, and the user can cancel.
    let http = client(None)?;
    let resp = http
        .post(&url)
        .json(&serde_json::json!({ "name": model, "stream": true }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Ollama: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned {}", resp.status()));
    }

    let mut stream = resp.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Err("Download cancelled".to_string());
        }
        let chunk = chunk.map_err(|e| format!("Download interrupted: {e}"))?;
        buffer.extend_from_slice(&chunk);

        // The response is newline-delimited JSON.
        while let Some(pos) = buffer.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&line);
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            match serde_json::from_str::<PullLine>(text) {
                Ok(parsed) => {
                    if let Some(err) = parsed.error {
                        return Err(err);
                    }
                    let _ = app.emit(
                        PROGRESS_EVENT,
                        OllamaPullProgress {
                            model: model.to_string(),
                            status: parsed.status,
                            completed: parsed.completed,
                            total: parsed.total,
                        },
                    );
                }
                Err(e) => warn!("Unparseable Ollama pull line: {}", e),
            }
        }
    }

    if cancel.load(Ordering::Relaxed) {
        return Err("Download cancelled".to_string());
    }
    Ok(())
}

/// Stop a running download. Ollama keeps partial data and resumes next time.
#[tauri::command]
#[specta::specta]
pub fn ollama_cancel_pull(model: String) -> Result<(), String> {
    let flags = cancel_flags().lock().map_err(|_| "lock poisoned")?;
    if let Some(flag) = flags.get(&model) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// Remove an installed model.
#[tauri::command]
#[specta::specta]
pub async fn ollama_delete_model(app: AppHandle, model: String) -> Result<(), String> {
    if !valid_model_name(&model) {
        return Err("Invalid model name".to_string());
    }
    let url = format!("{}/api/delete", ollama_root(&app));
    let http = client(Some(Duration::from_secs(30)))?;
    let resp = http
        .delete(&url)
        .json(&serde_json::json!({ "name": model }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Ollama: {e}"))?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Ollama returned {}", resp.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_names_are_restricted() {
        assert!(valid_model_name("qwen2.5:3b"));
        assert!(valid_model_name("library/llama3.2:3b"));
        assert!(!valid_model_name(""));
        assert!(!valid_model_name("a b"));
        assert!(!valid_model_name("x;rm -rf"));
        assert!(!valid_model_name(&"a".repeat(200)));
    }

    #[test]
    fn pull_lines_parse() {
        let ok: PullLine =
            serde_json::from_str(r#"{"status":"pulling abc","total":100,"completed":40}"#).unwrap();
        assert_eq!(ok.total, 100);
        assert_eq!(ok.completed, 40);
        let err: PullLine =
            serde_json::from_str(r#"{"error":"pull model manifest: not found"}"#).unwrap();
        assert!(err.error.is_some());
    }
}
