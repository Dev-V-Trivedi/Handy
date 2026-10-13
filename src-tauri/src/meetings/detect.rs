//! Notice when a call app comes to the front and ask "Take notes?".
//!
//! This only looks at the name and window title of the front app. It never
//! reads audio, screen content or call data. The check can be turned off in the
//! Meetings tab.

use crate::active_app::{foreground_app, ActiveApp};
use crate::settings::get_settings;
use log::debug;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const POLL: Duration = Duration::from_secs(4);
/// After asking about one app, stay quiet about it for this long.
const COOLDOWN: Duration = Duration::from_secs(20 * 60);

const BROWSERS: [&str; 9] = [
    "chrome", "msedge", "edge", "firefox", "brave", "arc", "safari", "opera", "vivaldi",
];

#[derive(Debug, Clone, Serialize)]
struct DetectedEvent {
    app: String,
}

/// Name of the call app if `app` looks like a call window, else `None`.
pub fn detect_meeting_app(app: &ActiveApp) -> Option<&'static str> {
    let name = app.name.to_lowercase();
    let title = app.title.to_lowercase();
    let in_browser = BROWSERS.iter().any(|b| name.contains(b));

    if name.contains("zoom") && title.contains("zoom meeting") {
        return Some("Zoom");
    }
    if in_browser
        && (title.contains("google meet")
            || title.starts_with("meet - ")
            || title.starts_with("meet – "))
    {
        return Some("Google Meet");
    }
    if (name.contains("teams") || in_browser)
        && title.contains("microsoft teams")
        && (title.contains("meeting") || title.contains("call"))
    {
        return Some("Microsoft Teams");
    }
    if name.contains("webex") && title.contains("meeting") {
        return Some("Webex");
    }
    if name.contains("slack") && title.contains("huddle") {
        return Some("Slack huddle");
    }
    None
}

/// Start the background check. Cheap: one front-window lookup every few seconds.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut asked: HashMap<&'static str, Instant> = HashMap::new();
        loop {
            std::thread::sleep(POLL);
            if !get_settings(&app).meeting_prompt_enabled {
                continue;
            }
            let Some(manager) = app
                .try_state::<Arc<super::MeetingManager>>()
                .map(|s| Arc::clone(&s))
            else {
                continue;
            };
            if manager.active().is_some() {
                continue;
            }
            let Some(front) = foreground_app() else {
                continue;
            };
            let Some(kind) = detect_meeting_app(&front) else {
                continue;
            };
            if asked
                .get(kind)
                .map(|t| t.elapsed() < COOLDOWN)
                .unwrap_or(false)
            {
                continue;
            }
            asked.insert(kind, Instant::now());
            debug!("Call app detected: {}", kind);
            let _ = app.emit(
                "meeting-detected",
                DetectedEvent {
                    app: kind.to_string(),
                },
            );
            crate::show_main_window(&app);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(name: &str, title: &str) -> ActiveApp {
        ActiveApp {
            name: name.to_string(),
            title: title.to_string(),
        }
    }

    #[test]
    fn finds_common_call_apps() {
        assert_eq!(
            detect_meeting_app(&app("zoom.us", "Zoom Meeting")),
            Some("Zoom")
        );
        assert_eq!(
            detect_meeting_app(&app("chrome.exe", "Meet - abc-defg-hij - Google Chrome")),
            Some("Google Meet")
        );
        assert_eq!(
            detect_meeting_app(&app(
                "ms-teams.exe",
                "Weekly sync | Meeting | Microsoft Teams"
            )),
            Some("Microsoft Teams")
        );
        assert_eq!(
            detect_meeting_app(&app("slack.exe", "Huddle with Sam")),
            Some("Slack huddle")
        );
    }

    #[test]
    fn ignores_ordinary_windows() {
        assert_eq!(detect_meeting_app(&app("zoom.us", "Zoom Workplace")), None);
        assert_eq!(
            detect_meeting_app(&app("chrome.exe", "Inbox - Gmail")),
            None
        );
        assert_eq!(
            detect_meeting_app(&app("ms-teams.exe", "Chat | Microsoft Teams")),
            None
        );
        assert_eq!(
            detect_meeting_app(&app("code.exe", "meeting-notes.md")),
            None
        );
    }
}
