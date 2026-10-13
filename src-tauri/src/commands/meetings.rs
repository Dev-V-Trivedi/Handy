//! Tauri commands for meeting notes.

use crate::meetings::store::{MeetingDetail, MeetingRow};
use crate::meetings::{ActiveMeeting, MeetingManager};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

fn manager(app: &AppHandle) -> Result<Arc<MeetingManager>, String> {
    app.try_state::<Arc<MeetingManager>>()
        .map(|s| Arc::clone(&s))
        .ok_or_else(|| "Meeting notes are not available".to_string())
}

#[tauri::command]
#[specta::specta]
pub fn meeting_start(app: AppHandle, title: Option<String>) -> Result<ActiveMeeting, String> {
    manager(&app)?.start(title)
}

#[tauri::command]
#[specta::specta]
pub async fn meeting_stop(app: AppHandle) -> Result<i64, String> {
    let mgr = manager(&app)?;
    let mgr_for_stop = Arc::clone(&mgr);
    let id = tauri::async_runtime::spawn_blocking(move || mgr_for_stop.stop())
        .await
        .map_err(|e| e.to_string())??;
    mgr.summarize_in_background(id);
    Ok(id)
}

#[tauri::command]
#[specta::specta]
pub fn meeting_active(app: AppHandle) -> Result<Option<ActiveMeeting>, String> {
    Ok(manager(&app)?.active())
}

#[tauri::command]
#[specta::specta]
pub fn meeting_list(app: AppHandle) -> Result<Vec<MeetingRow>, String> {
    manager(&app)?.store().list().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn meeting_get(app: AppHandle, id: i64) -> Result<Option<MeetingDetail>, String> {
    manager(&app)?.store().get(id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn meeting_rename(app: AppHandle, id: i64, title: String) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    manager(&app)?
        .store()
        .rename(id, title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn meeting_delete(app: AppHandle, id: i64) -> Result<(), String> {
    let mgr = manager(&app)?;
    if mgr.active().map(|a| a.id) == Some(id) {
        return Err("Stop the meeting before deleting it".to_string());
    }
    mgr.store().delete(id).map_err(|e| e.to_string())
}

/// Make (or remake) the summary for a finished meeting.
#[tauri::command]
#[specta::specta]
pub fn meeting_summarize(app: AppHandle, id: i64) -> Result<(), String> {
    manager(&app)?.summarize_in_background(id);
    Ok(())
}
