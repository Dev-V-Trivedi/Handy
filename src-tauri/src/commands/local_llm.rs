//! Commands for the built-in local cleanup model.

use crate::local_llm::{self, LocalLlmStatus};
use crate::settings::{get_settings, write_settings};
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
pub fn local_llm_status(app: AppHandle) -> Result<LocalLlmStatus, String> {
    local_llm::status(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn local_llm_install_runner(app: AppHandle) -> Result<(), String> {
    local_llm::install_runner(&app).await
}

#[tauri::command]
#[specta::specta]
pub async fn local_llm_download_model(app: AppHandle, id: String) -> Result<(), String> {
    local_llm::download_model(&app, &id).await
}

#[tauri::command]
#[specta::specta]
pub fn local_llm_delete_model(app: AppHandle, id: String) -> Result<(), String> {
    local_llm::delete_model(&app, &id)
}

/// Make the built-in model the active cleanup model.
#[tauri::command]
#[specta::specta]
pub fn local_llm_use_model(app: AppHandle, id: String) -> Result<(), String> {
    if !local_llm::CATALOG.iter().any(|m| m.id == id) {
        return Err("Unknown model".to_string());
    }
    let mut settings = get_settings(&app);
    settings.post_process_provider_id = local_llm::PROVIDER_ID.to_string();
    settings
        .post_process_models
        .insert(local_llm::PROVIDER_ID.to_string(), id);
    write_settings(&app, settings);
    Ok(())
}
