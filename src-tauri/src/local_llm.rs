//! Built-in local cleanup model, no Ollama needed.
//!
//! We never link llama.cpp into the app: the speech engine already ships a
//! different ggml and two copies in one process clash. Instead Oshabe downloads
//! the official `llama-server` program and a small GGUF model, starts the server
//! on 127.0.0.1 only when cleanup needs it, and stops it when idle or on exit.
//! Cleanup then goes through the same OpenAI-style path as every other provider.

use futures_util::StreamExt;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub const PROVIDER_ID: &str = "oshabe_local";
/// Fixed loopback port so the provider's base URL never changes.
pub const PORT: u16 = 38519;
const PROGRESS_EVENT: &str = "local-llm-progress";
const IDLE_STOP: Duration = Duration::from_secs(10 * 60);
const START_TIMEOUT: Duration = Duration::from_secs(120);
const RELEASE_API: &str = "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest";

pub struct CatalogModel {
    pub id: &'static str,
    pub name: &'static str,
    pub file: &'static str,
    pub url: &'static str,
    pub size_mb: u32,
}

pub const CATALOG: &[CatalogModel] = &[
    CatalogModel {
        id: "qwen2.5-1.5b",
        name: "Qwen 2.5 1.5B (fast)",
        file: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf",
        size_mb: 1100,
    },
    CatalogModel {
        id: "qwen2.5-3b",
        name: "Qwen 2.5 3B (better)",
        file: "qwen2.5-3b-instruct-q4_k_m.gguf",
        url: "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf",
        size_mb: 2100,
    },
];

fn catalog_model(id: &str) -> Option<&'static CatalogModel> {
    CATALOG.iter().find(|m| m.id == id)
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct LocalModelStatus {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct LocalLlmStatus {
    pub runner_installed: bool,
    pub models: Vec<LocalModelStatus>,
    /// Model currently loaded in the running server, if any.
    pub running: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct LocalLlmProgress {
    /// `runner` or a catalog model id.
    pub id: String,
    pub completed: u64,
    pub total: u64,
}

struct Running {
    child: Child,
    model: String,
    last_used: Instant,
}

fn state() -> &'static Mutex<Option<Running>> {
    static S: OnceLock<Mutex<Option<Running>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

fn start_guard() -> &'static tauri::async_runtime::Mutex<()> {
    static G: OnceLock<tauri::async_runtime::Mutex<()>> = OnceLock::new();
    G.get_or_init(|| tauri::async_runtime::Mutex::new(()))
}

fn base_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(crate::portable::app_data_dir(app)
        .map_err(|e| e.to_string())?
        .join("local-llm"))
}

fn runner_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(base_dir(app)?.join("runner"))
}

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(base_dir(app)?.join("models"))
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    }
}

/// Find `llama-server` anywhere under the runner folder (archives nest it).
fn find_runner(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_runner(&path) {
                return Some(found);
            }
        } else if path.file_name().map(|n| n == exe_name()).unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

pub fn status(app: &AppHandle) -> Result<LocalLlmStatus, String> {
    let runner_installed = find_runner(&runner_dir(app)?).is_some();
    let models_dir = models_dir(app)?;
    let models = CATALOG
        .iter()
        .map(|m| LocalModelStatus {
            id: m.id.to_string(),
            name: m.name.to_string(),
            size_mb: m.size_mb,
            installed: models_dir.join(m.file).is_file(),
        })
        .collect();
    let running = state()
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|r| r.model.clone()));
    Ok(LocalLlmStatus {
        runner_installed,
        models,
        running,
    })
}

/// Choose the right release archive for this computer, or `None`.
pub fn pick_asset<'a>(names: &'a [String], os: &str, arch: &str) -> Option<&'a String> {
    let arch_tokens: &[&str] = match arch {
        "x86_64" => &["x64"],
        "aarch64" => &["arm64"],
        _ => return None,
    };
    let os_token = match os {
        "windows" => "win",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => return None,
    };
    const BAD: [&str; 11] = [
        "cuda", "vulkan", "hip", "sycl", "opencl", "openvino", "kleidiai", "rocm", "cudart",
        "radeon", "musa",
    ];
    names.iter().find(|n| {
        let l = n.to_lowercase();
        (l.ends_with(".zip") || l.ends_with(".tar.gz"))
            && l.contains("-bin-")
            && l.contains(os_token)
            && arch_tokens.iter().any(|a| l.contains(a))
            && !BAD.iter().any(|b| l.contains(b))
            && (os != "windows" || l.contains("cpu"))
    })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

async fn download(
    app: &AppHandle,
    id: &str,
    url: &str,
    dest: &Path,
    expected_sha: Option<String>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Oshabe")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!(
            "Download failed: server returned {}",
            resp.status()
        ));
    }
    // Hugging Face exposes the file's SHA-256 as the linked ETag.
    let header_sha = resp
        .headers()
        .get("x-linked-etag")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_matches('"').to_lowercase())
        .filter(|v| v.len() == 64 && v.chars().all(|c| c.is_ascii_hexdigit()));
    let expected = expected_sha.or(header_sha);
    let total = resp.content_length().unwrap_or(0);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let part = dest.with_extension("part");
    let result = async {
        let mut file = create_file(&part)?;
        let mut stream = resp.bytes_stream();
        let mut done = 0u64;
        let mut last_emit = Instant::now();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download interrupted: {e}"))?;
            std::io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
            done += chunk.len() as u64;
            if last_emit.elapsed() > Duration::from_millis(250) {
                last_emit = Instant::now();
                let _ = app.emit(
                    PROGRESS_EVENT,
                    LocalLlmProgress {
                        id: id.to_string(),
                        completed: done,
                        total,
                    },
                );
            }
        }
        drop(file);
        if let Some(expected) = expected {
            let part_for_hash = part.clone();
            let actual = tauri::async_runtime::spawn_blocking(move || sha256_file(&part_for_hash))
                .await
                .map_err(|e| e.to_string())??;
            if actual != expected.to_lowercase() {
                return Err("The download is corrupted. Please try again.".to_string());
            }
        } else {
            warn!("No published checksum for {}, skipping verification", id);
        }
        std::fs::rename(&part, dest).map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    }
    .await;
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

fn create_file(path: &Path) -> Result<std::fs::File, String> {
    std::fs::File::create(path).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    digest: Option<String>,
}

pub async fn install_runner(app: &AppHandle) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Oshabe")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let release: Release = client
        .get(RELEASE_API)
        .send()
        .await
        .map_err(|e| format!("Could not reach GitHub: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Could not read the llama.cpp release list: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let names: Vec<String> = release.assets.iter().map(|a| a.name.clone()).collect();
    let name = pick_asset(&names, std::env::consts::OS, std::env::consts::ARCH)
        .ok_or_else(|| {
            "No built-in model runner is available for this computer. You can use Ollama instead."
                .to_string()
        })?
        .clone();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or("asset vanished")?;
    let expected = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(|d| d.to_lowercase());

    let dir = runner_dir(app)?;
    let archive = base_dir(app)?.join(&name);
    download(
        app,
        "runner",
        &asset.browser_download_url,
        &archive,
        expected,
    )
    .await?;

    let extract_to = dir.clone();
    let archive_for_extract = archive.clone();
    let extracted = tauri::async_runtime::spawn_blocking(move || {
        let _ = std::fs::remove_dir_all(&extract_to);
        std::fs::create_dir_all(&extract_to).map_err(|e| e.to_string())?;
        extract(&archive_for_extract, &extract_to)
    })
    .await
    .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&archive);
    extracted?;

    let runner = find_runner(&dir).ok_or("The download did not contain llama-server")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&runner, std::fs::Permissions::from_mode(0o755));
    }
    info!("Installed llama-server at {}", runner.display());
    Ok(())
}

fn extract(archive: &Path, to: &Path) -> Result<(), String> {
    let name = archive.to_string_lossy().to_lowercase();
    if name.ends_with(".zip") {
        let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            // Skip anything that would escape the target folder.
            let Some(rel) = entry.enclosed_name() else {
                continue;
            };
            let out = to.join(rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    } else {
        let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let gz = flate2::read::GzDecoder::new(file);
        tar::Archive::new(gz).unpack(to).map_err(|e| e.to_string())
    }
}

pub async fn download_model(app: &AppHandle, id: &str) -> Result<(), String> {
    let model = catalog_model(id).ok_or("Unknown model")?;
    let dest = models_dir(app)?.join(model.file);
    download(app, model.id, model.url, &dest, None).await
}

pub fn delete_model(app: &AppHandle, id: &str) -> Result<(), String> {
    let model = catalog_model(id).ok_or("Unknown model")?;
    if state()
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|r| r.model == id))
        .unwrap_or(false)
    {
        stop();
    }
    let path = models_dir(app)?.join(model.file);
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Stop the server (also used on app exit).
pub fn stop() {
    if let Ok(mut guard) = state().lock() {
        if let Some(mut running) = guard.take() {
            let _ = running.child.kill();
            let _ = running.child.wait();
            debug!("llama-server stopped");
        }
    }
}

fn spawn_idle_watcher() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }
    std::thread::spawn(|| loop {
        std::thread::sleep(Duration::from_secs(60));
        let idle = state()
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|r| r.last_used.elapsed() > IDLE_STOP))
            .unwrap_or(false);
        if idle {
            info!("Stopping idle local model");
            stop();
        }
    });
}

fn alive_with(model: &str) -> bool {
    let Ok(mut guard) = state().lock() else {
        return false;
    };
    let Some(r) = guard.as_mut() else {
        return false;
    };
    if r.model == model && matches!(r.child.try_wait(), Ok(None)) {
        r.last_used = Instant::now();
        true
    } else {
        false
    }
}

/// Make sure the server is running with `model_id` loaded. Fast when it already is.
pub async fn ensure_running(app: &AppHandle, model_id: &str) -> Result<(), String> {
    if alive_with(model_id) {
        return Ok(());
    }
    let _guard = start_guard().lock().await;
    if alive_with(model_id) {
        return Ok(());
    }
    let model = catalog_model(model_id).ok_or("Unknown local model")?;
    let model_path = models_dir(app)?.join(model.file);
    if !model_path.is_file() {
        return Err("The local model is not downloaded yet".to_string());
    }
    let runner = find_runner(&runner_dir(app)?)
        .ok_or_else(|| "The local model runner is not installed yet".to_string())?;
    stop();

    let mut command = Command::new(&runner);
    command
        .args(["-m"])
        .arg(&model_path)
        .args(["--host", "127.0.0.1", "--port"])
        .arg(PORT.to_string())
        .args(["-c", "4096"])
        .current_dir(runner.parent().unwrap_or(Path::new(".")))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(target_os = "linux")]
    if let Some(dir) = runner.parent() {
        command.env("LD_LIBRARY_PATH", dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let child = command
        .spawn()
        .map_err(|e| format!("Could not start the local model: {e}"))?;
    if let Ok(mut guard) = state().lock() {
        *guard = Some(Running {
            child,
            model: model_id.to_string(),
            last_used: Instant::now(),
        });
    }
    spawn_idle_watcher();

    let http = reqwest::Client::new();
    let deadline = Instant::now() + START_TIMEOUT;
    let health = format!("http://127.0.0.1:{PORT}/health");
    while Instant::now() < deadline {
        if !alive_with(model_id) {
            stop();
            return Err("The local model stopped while starting".to_string());
        }
        if let Ok(resp) = http
            .get(&health)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            if resp.status().is_success() {
                info!("Local model {} ready", model_id);
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    stop();
    error!("Local model did not become ready in time");
    Err("The local model took too long to start".to_string())
}

/// Call before any cleanup request: starts the local server when the active
/// provider is the built-in one. Does nothing for every other provider.
pub async fn prepare(
    app: &AppHandle,
    settings: &crate::settings::AppSettings,
) -> Result<(), String> {
    let Some(provider) = settings.active_post_process_provider() else {
        return Ok(());
    };
    if provider.id != PROVIDER_ID {
        return Ok(());
    }
    let model = settings
        .post_process_models
        .get(PROVIDER_ID)
        .cloned()
        .unwrap_or_default();
    ensure_running(app, &model).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn picks_plain_cpu_builds() {
        let all = names(&[
            "llama-b1-bin-win-cuda-12.4-x64.zip",
            "llama-b1-bin-win-vulkan-x64.zip",
            "llama-b1-bin-win-cpu-x64.zip",
            "llama-b1-bin-win-cpu-arm64.zip",
            "llama-b1-bin-macos-arm64.tar.gz",
            "llama-b1-bin-macos-arm64-kleidiai.tar.gz",
            "llama-b1-bin-macos-x64.tar.gz",
            "llama-b1-bin-ubuntu-x64.tar.gz",
            "llama-b1-bin-ubuntu-vulkan-x64.tar.gz",
            "cudart-llama-bin-win-cuda-12.4-x64.zip",
        ]);
        assert_eq!(
            pick_asset(&all, "windows", "x86_64").map(String::as_str),
            Some("llama-b1-bin-win-cpu-x64.zip")
        );
        assert_eq!(
            pick_asset(&all, "windows", "aarch64").map(String::as_str),
            Some("llama-b1-bin-win-cpu-arm64.zip")
        );
        assert_eq!(
            pick_asset(&all, "macos", "aarch64").map(String::as_str),
            Some("llama-b1-bin-macos-arm64.tar.gz")
        );
        assert_eq!(
            pick_asset(&all, "macos", "x86_64").map(String::as_str),
            Some("llama-b1-bin-macos-x64.tar.gz")
        );
        assert_eq!(
            pick_asset(&all, "linux", "x86_64").map(String::as_str),
            Some("llama-b1-bin-ubuntu-x64.tar.gz")
        );
        assert_eq!(pick_asset(&all, "linux", "aarch64"), None);
        assert_eq!(pick_asset(&all, "freebsd", "x86_64"), None);
    }

    #[test]
    fn catalog_ids_are_unique_and_files_are_gguf() {
        let mut ids: Vec<_> = CATALOG.iter().map(|m| m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len());
        assert!(CATALOG.iter().all(|m| m.file.ends_with(".gguf")));
        assert!(CATALOG
            .iter()
            .all(|m| m.url.starts_with("https://huggingface.co/")));
    }

    #[test]
    fn zip_slip_entries_are_not_extracted() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("oshabe-zip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("t.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("../evil.txt", opts).unwrap();
            w.write_all(b"x").unwrap();
            w.start_file("ok/llama-server", opts).unwrap();
            w.write_all(b"y").unwrap();
            w.finish().unwrap();
        }
        let out = dir.join("out");
        std::fs::create_dir_all(&out).unwrap();
        extract(&zip_path, &out).unwrap();
        assert!(out.join("ok/llama-server").is_file());
        assert!(!dir.join("evil.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
