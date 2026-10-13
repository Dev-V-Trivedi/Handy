//! Foreground application detection and per-app tone matching.
//!
//! Used by post-processing so the LLM cleanup can adapt tone to where the text
//! will land (chat app, email client, code editor). Detection is best effort:
//! on any failure (Wayland, missing tool, permissions) it returns `None` and
//! post-processing proceeds with the plain prompt.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Foreground application snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveApp {
    /// Process or application name, e.g. `slack.exe`, `Slack`, `code`.
    pub name: String,
    /// Window title, if available. Lets rules match web apps in a browser
    /// (e.g. "Gmail" in the tab title).
    pub title: String,
}

/// User-editable rule: when `pattern` appears in the app name or window title
/// (case-insensitive substring), append `tone` to the cleanup prompt.
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
pub struct AppToneRule {
    pub id: String,
    pub pattern: String,
    pub tone: String,
}

pub fn default_tone_rules() -> Vec<AppToneRule> {
    let rule = |id: &str, pattern: &str, tone: &str| AppToneRule {
        id: id.to_string(),
        pattern: pattern.to_string(),
        tone: tone.to_string(),
    };
    vec![
        rule(
            "chat_casual",
            "slack",
            "Casual chat message. Keep it short and conversational. Contractions are fine. No greeting or sign-off unless spoken.",
        ),
        rule(
            "chat_discord",
            "discord",
            "Casual chat message. Keep it short and conversational. Contractions are fine.",
        ),
        rule(
            "chat_whatsapp",
            "whatsapp",
            "Casual chat message. Keep it short and conversational. Contractions are fine.",
        ),
        rule(
            "chat_telegram",
            "telegram",
            "Casual chat message. Keep it short and conversational. Contractions are fine.",
        ),
        rule(
            "email_outlook",
            "outlook",
            "Professional email tone. Complete sentences, polite, no slang.",
        ),
        rule(
            "email_gmail",
            "gmail",
            "Professional email tone. Complete sentences, polite, no slang.",
        ),
        rule(
            "email_thunderbird",
            "thunderbird",
            "Professional email tone. Complete sentences, polite, no slang.",
        ),
        rule(
            "code_vscode",
            "code",
            "Technical context. Keep identifiers, file names, commands and code terms exactly as spoken, in their usual casing. Do not add prose punctuation to code.",
        ),
        rule(
            "code_cursor",
            "cursor",
            "Technical context. Keep identifiers, file names, commands and code terms exactly as spoken, in their usual casing.",
        ),
        rule(
            "code_terminal",
            "terminal",
            "Technical context. Keep commands, flags and paths exactly as spoken. No added punctuation.",
        ),
    ]
}

/// First rule whose pattern appears in the app name or window title.
/// Empty patterns never match. Rules are checked in order, so users can put
/// specific rules before generic ones.
pub fn match_tone<'a>(rules: &'a [AppToneRule], app: &ActiveApp) -> Option<&'a AppToneRule> {
    let name = app.name.to_lowercase();
    let title = app.title.to_lowercase();
    rules.iter().find(|rule| {
        let pattern = rule.pattern.trim().to_lowercase();
        !pattern.is_empty() && (name.contains(&pattern) || title.contains(&pattern))
    })
}

/// Text appended to the cleanup prompt for a matched rule.
pub fn tone_instruction(rule: &AppToneRule) -> String {
    format!(
        "\n\nContext: the text will be pasted into an app where this style applies: {}\nApply this style without changing the meaning or adding content.",
        rule.tone.trim()
    )
}

/// Best-effort foreground app lookup. Blocking, call from a blocking context
/// or accept the few-ms cost on Windows and tens of ms on macOS/Linux.
pub fn foreground_app() -> Option<ActiveApp> {
    imp::foreground_app()
}

#[cfg(windows)]
mod imp {
    use super::ActiveApp;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HWND};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    pub fn foreground_app() -> Option<ActiveApp> {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut title_buf = [0u16; 512];
            let title_len = GetWindowTextW(hwnd, &mut title_buf).max(0) as usize;
            let title = String::from_utf16_lossy(&title_buf[..title_len.min(title_buf.len())]);

            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut path_buf = [0u16; 1024];
            let mut size = path_buf.len() as u32;
            let queried = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(path_buf.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(handle);
            queried.ok()?;

            let path = String::from_utf16_lossy(&path_buf[..size as usize]);
            let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
            Some(ActiveApp { name, title })
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::ActiveApp;
    use std::process::Command;

    pub fn foreground_app() -> Option<ActiveApp> {
        // lsappinfo ships with macOS and needs no Accessibility permission.
        let front = Command::new("lsappinfo").arg("front").output().ok()?;
        let asn = String::from_utf8_lossy(&front.stdout).trim().to_string();
        if asn.is_empty() {
            return None;
        }
        let info = Command::new("lsappinfo")
            .args(["info", "-only", "name", &asn])
            .output()
            .ok()?;
        // Output looks like: "LSDisplayName"="Slack"
        let text = String::from_utf8_lossy(&info.stdout);
        let name = text.split('=').nth(1)?.trim().trim_matches('"').to_string();
        if name.is_empty() {
            return None;
        }
        Some(ActiveApp {
            name,
            title: String::new(),
        })
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use super::ActiveApp;
    use std::process::Command;

    fn run(cmd: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(cmd).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    }

    pub fn foreground_app() -> Option<ActiveApp> {
        // X11 only. On Wayland there is no portable API, so this returns None.
        let id = run("xdotool", &["getactivewindow"])?;
        let title = run("xdotool", &["getwindowname", &id]).unwrap_or_default();
        let class = run("xprop", &["-id", &id, "WM_CLASS"])
            .and_then(|s| {
                // WM_CLASS(STRING) = "code", "Code"
                s.split('=')
                    .nth(1)
                    .map(|v| v.trim().trim_matches('"').replace("\", \"", " "))
            })
            .unwrap_or_default();
        let name = if class.is_empty() {
            title.clone()
        } else {
            class
        };
        Some(ActiveApp { name, title })
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    use super::ActiveApp;
    pub fn foreground_app() -> Option<ActiveApp> {
        None
    }
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
    fn matches_on_process_name_case_insensitive() {
        let rules = default_tone_rules();
        let hit = match_tone(&rules, &app("Slack.exe", "general | Acme")).unwrap();
        assert_eq!(hit.id, "chat_casual");
    }

    #[test]
    fn matches_web_app_via_window_title() {
        let rules = default_tone_rules();
        let hit = match_tone(&rules, &app("chrome.exe", "Inbox (3) - Gmail")).unwrap();
        assert_eq!(hit.id, "email_gmail");
    }

    #[test]
    fn first_rule_wins_so_specific_rules_can_go_first() {
        let rules = vec![
            AppToneRule {
                id: "a".into(),
                pattern: "code review".into(),
                tone: "formal".into(),
            },
            AppToneRule {
                id: "b".into(),
                pattern: "code".into(),
                tone: "technical".into(),
            },
        ];
        let hit = match_tone(&rules, &app("chrome", "Code review - GitHub")).unwrap();
        assert_eq!(hit.id, "a");
    }

    #[test]
    fn empty_pattern_never_matches() {
        let rules = vec![AppToneRule {
            id: "x".into(),
            pattern: "  ".into(),
            tone: "anything".into(),
        }];
        assert!(match_tone(&rules, &app("notepad.exe", "Untitled")).is_none());
    }

    #[test]
    fn no_match_returns_none() {
        let rules = default_tone_rules();
        assert!(match_tone(&rules, &app("notepad.exe", "Untitled")).is_none());
    }

    #[test]
    fn instruction_embeds_tone() {
        let rules = default_tone_rules();
        let text = tone_instruction(&rules[0]);
        assert!(text.contains("Casual chat"));
        assert!(text.starts_with("\n\nContext:"));
    }
}
