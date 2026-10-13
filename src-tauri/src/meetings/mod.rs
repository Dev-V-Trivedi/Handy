//! Meeting notes: record a call (microphone plus computer audio), transcribe
//! it as it happens, and summarise it afterwards.
//!
//! Everything runs on the device. Only text is saved, never audio.

pub mod capture;
pub mod store;
pub mod summary;

use crate::managers::transcription::TranscriptionManager;
use crate::settings::get_settings;
use capture::{CaptureHandle, Track, TARGET_RATE};
use log::{debug, error, info, warn};
use serde::Serialize;
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use store::MeetingStore;
use tauri::{AppHandle, Emitter, Manager};

/// Shortest and longest piece of audio sent to the speech model at once.
const MIN_FLUSH_SECS: f32 = 10.0;
const MAX_FLUSH_SECS: f32 = 28.0;
/// Pieces shorter than this are not worth transcribing.
const MIN_SPEECH_SECS: f32 = 0.4;
/// A pause this long ends a piece early (computer audio is silent between talk).
const GAP_FLUSH: Duration = Duration::from_millis(1_500);
/// Length of trailing quiet that marks a natural break.
const QUIET_TAIL_SECS: f32 = 0.6;
/// Below this RMS level a piece is treated as silence.
const SILENCE_RMS: f32 = 0.004;
/// How far back to look for the other side's words when removing echo.
const ECHO_WINDOW_MS: i64 = 40_000;

#[derive(Debug, Clone, Serialize, Type)]
pub struct ActiveMeeting {
    pub id: i64,
    pub started_at: i64,
    /// False when only the microphone is being recorded.
    pub system_audio: bool,
    /// Machine-readable note for the UI (`system_audio_unavailable: ...`,
    /// `microphone_unavailable: ...`), if something is degraded.
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SegmentEvent {
    meeting_id: i64,
    track: String,
    start_ms: i64,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorEvent {
    meeting_id: i64,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct SummaryEvent {
    meeting_id: i64,
    ok: bool,
    message: String,
}

struct Running {
    info: ActiveMeeting,
    stop: Arc<AtomicBool>,
    captures: Vec<CaptureHandle>,
    worker: JoinHandle<()>,
}

pub struct MeetingManager {
    app: AppHandle,
    store: Arc<MeetingStore>,
    active: Mutex<Option<Running>>,
}

impl MeetingManager {
    pub fn new(app: &AppHandle) -> anyhow::Result<Self> {
        let dir = crate::portable::app_data_dir(app)?;
        std::fs::create_dir_all(&dir)?;
        let store = Arc::new(MeetingStore::open(&dir.join("meetings.db"))?);
        Ok(Self {
            app: app.clone(),
            store,
            active: Mutex::new(None),
        })
    }

    pub fn store(&self) -> Arc<MeetingStore> {
        Arc::clone(&self.store)
    }

    pub fn active(&self) -> Option<ActiveMeeting> {
        self.active
            .lock()
            .ok()
            .and_then(|a| a.as_ref().map(|r| r.info.clone()))
    }

    pub fn start(&self, title: Option<String>) -> Result<ActiveMeeting, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        if active.is_some() {
            return Err("A meeting is already being recorded".to_string());
        }

        let settings = get_settings(&self.app);
        if settings.selected_model.trim().is_empty() {
            return Err("no_model".to_string());
        }
        let tm = self.app.state::<Arc<TranscriptionManager>>();
        tm.initiate_model_load();

        let started_at = chrono::Utc::now().timestamp();
        let (title, is_default) = match title
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
        {
            Some(t) => (t, false),
            None => (
                format!("Meeting {}", chrono::Local::now().format("%Y-%m-%d %H:%M")),
                true,
            ),
        };
        let id = self
            .store
            .create(&title, is_default, started_at)
            .map_err(|e| e.to_string())?;

        let (tx, rx) = mpsc::channel();
        let mic = capture::start(Track::You, settings.selected_microphone.clone(), tx.clone());
        let system = capture::start(Track::Others, None, tx.clone());
        drop(tx);

        let mut captures = Vec::new();
        let mut warning = None;
        let mut system_audio = false;
        match mic {
            Ok(h) => captures.push(h),
            Err(e) => warning = Some(format!("microphone_unavailable: {e}")),
        }
        match system {
            Ok(h) => {
                captures.push(h);
                system_audio = true;
            }
            Err(e) => {
                warning.get_or_insert(format!("system_audio_unavailable: {e}"));
            }
        }
        if captures.is_empty() {
            let _ = self.store.delete(id);
            return Err(warning.unwrap_or_else(|| "Could not start recording".to_string()));
        }

        let stop = Arc::new(AtomicBool::new(false));
        let worker = {
            let app = self.app.clone();
            let store = Arc::clone(&self.store);
            let stop = Arc::clone(&stop);
            std::thread::Builder::new()
                .name("meeting-worker".to_string())
                .spawn(move || run_worker(app, store, id, rx, stop, Instant::now()))
                .map_err(|e| e.to_string())?
        };

        let info = ActiveMeeting {
            id,
            started_at,
            system_audio,
            warning,
        };
        info!(
            "Meeting {} started (system audio: {}, warning: {:?})",
            id, info.system_audio, info.warning
        );
        *active = Some(Running {
            info: info.clone(),
            stop,
            captures,
            worker,
        });
        let _ = self.app.emit("meeting-status", true);
        Ok(info)
    }

    /// Stop recording, wait for the last piece to be transcribed and return the
    /// meeting id. Blocking: call from a blocking thread.
    pub fn stop(&self) -> Result<i64, String> {
        let running = self
            .active
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .take()
            .ok_or_else(|| "No meeting is being recorded".to_string())?;
        let id = running.info.id;

        running.stop.store(true, Ordering::SeqCst);
        for capture in running.captures {
            capture.stop();
        }
        if running.worker.join().is_err() {
            error!("Meeting worker panicked");
        }
        if let Err(e) = self.store.finish(id, chrono::Utc::now().timestamp()) {
            error!("Failed to close meeting {}: {}", id, e);
        }
        let _ = self.app.emit("meeting-status", false);
        info!("Meeting {} stopped", id);
        Ok(id)
    }

    /// Make the summary in the background and tell the UI when it is done.
    pub fn summarize_in_background(&self, id: i64) {
        let app = self.app.clone();
        let store = Arc::clone(&self.store);
        tauri::async_runtime::spawn(async move {
            let event = match summary::summarize(&app, &store, id).await {
                Ok(()) => SummaryEvent {
                    meeting_id: id,
                    ok: true,
                    message: String::new(),
                },
                Err(message) => {
                    debug!("Meeting {} summary not made: {}", id, message);
                    SummaryEvent {
                        meeting_id: id,
                        ok: false,
                        message,
                    }
                }
            };
            let _ = app.emit("meeting-summary", event);
        });
    }
}

/// Audio waiting to be transcribed for one track.
struct TrackBuf {
    track: Track,
    samples: Vec<f32>,
    /// Meeting time of the first sample, in milliseconds.
    start_ms: i64,
    last_arrival: Instant,
}

impl TrackBuf {
    fn new(track: Track) -> Self {
        Self {
            track,
            samples: Vec::new(),
            start_ms: 0,
            last_arrival: Instant::now(),
        }
    }

    fn push(&mut self, chunk: Vec<f32>, elapsed_ms: i64) {
        if self.samples.is_empty() {
            let chunk_ms = (chunk.len() as f64 * 1000.0 / TARGET_RATE as f64) as i64;
            self.start_ms = (elapsed_ms - chunk_ms).max(0);
        }
        self.samples.extend(chunk);
        self.last_arrival = Instant::now();
    }

    fn secs(&self) -> f32 {
        self.samples.len() as f32 / TARGET_RATE as f32
    }

    fn should_flush(&self, now: Instant) -> bool {
        if self.samples.is_empty() {
            return false;
        }
        let secs = self.secs();
        secs >= MAX_FLUSH_SECS
            || now.duration_since(self.last_arrival) >= GAP_FLUSH
            || (secs >= MIN_FLUSH_SECS && tail_is_quiet(&self.samples))
    }
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

fn tail_is_quiet(samples: &[f32]) -> bool {
    let n = (QUIET_TAIL_SECS * TARGET_RATE as f32) as usize;
    if samples.len() < n {
        return false;
    }
    rms(&samples[samples.len() - n..]) < SILENCE_RMS
}

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Without headphones the microphone hears the other side through the
/// speakers. Drop a "You" line whose words are almost all in what "Others"
/// just said.
fn is_echo(you_text: &str, recent_others: &[(i64, String)], start_ms: i64) -> bool {
    let you = words(you_text);
    if you.len() < 3 {
        return false;
    }
    let mut pool: Vec<String> = Vec::new();
    for (ms, text) in recent_others {
        if (start_ms - ms).abs() <= ECHO_WINDOW_MS {
            pool.extend(words(text));
        }
    }
    if pool.is_empty() {
        return false;
    }
    let hits = you.iter().filter(|w| pool.contains(w)).count();
    hits as f32 / you.len() as f32 >= 0.75
}

struct Worker {
    app: AppHandle,
    store: Arc<MeetingStore>,
    meeting_id: i64,
    recent_others: Vec<(i64, String)>,
    error_reported: bool,
}

impl Worker {
    fn wait_for_model(&self, tm: &TranscriptionManager) -> bool {
        if tm.is_model_loaded() {
            return true;
        }
        tm.initiate_model_load();
        for _ in 0..240 {
            if tm.is_model_loaded() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        false
    }

    fn flush(&mut self, buf: &mut TrackBuf) {
        let samples = std::mem::take(&mut buf.samples);
        if (samples.len() as f32 / TARGET_RATE as f32) < MIN_SPEECH_SECS
            || rms(&samples) < SILENCE_RMS
        {
            return;
        }

        let tm = self.app.state::<Arc<TranscriptionManager>>();
        if !self.wait_for_model(&tm) {
            self.report("The speech model is not loaded".to_string());
            return;
        }

        let text = match tm.transcribe(samples) {
            Ok(t) => t.trim().to_string(),
            Err(e) => {
                warn!("Meeting chunk failed: {}", e);
                self.report(e.to_string());
                return;
            }
        };
        if text.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
            return;
        }

        match buf.track {
            Track::Others => {
                self.recent_others.push((buf.start_ms, text.clone()));
                let cutoff = buf.start_ms - ECHO_WINDOW_MS * 2;
                self.recent_others.retain(|(ms, _)| *ms >= cutoff);
            }
            Track::You => {
                if is_echo(&text, &self.recent_others, buf.start_ms) {
                    debug!("Dropped echo from microphone: {}", text);
                    return;
                }
            }
        }

        if let Err(e) =
            self.store
                .add_segment(self.meeting_id, buf.track.as_str(), buf.start_ms, &text)
        {
            error!("Failed to save meeting segment: {}", e);
        }
        let _ = self.app.emit(
            "meeting-segment",
            SegmentEvent {
                meeting_id: self.meeting_id,
                track: buf.track.as_str().to_string(),
                start_ms: buf.start_ms,
                text,
            },
        );
    }

    fn report(&mut self, message: String) {
        if self.error_reported {
            return;
        }
        self.error_reported = true;
        let _ = self.app.emit(
            "meeting-error",
            ErrorEvent {
                meeting_id: self.meeting_id,
                message,
            },
        );
    }
}

fn run_worker(
    app: AppHandle,
    store: Arc<MeetingStore>,
    meeting_id: i64,
    rx: Receiver<(Track, Vec<f32>)>,
    _stop: Arc<AtomicBool>,
    started: Instant,
) {
    let mut worker = Worker {
        app,
        store,
        meeting_id,
        recent_others: Vec::new(),
        error_reported: false,
    };
    let mut bufs = [TrackBuf::new(Track::You), TrackBuf::new(Track::Others)];

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok((track, chunk)) => {
                let idx = if track == Track::You { 0 } else { 1 };
                bufs[idx].push(chunk, started.elapsed().as_millis() as i64);
            }
            Err(RecvTimeoutError::Timeout) => {}
            // Every capture has stopped and the queue is drained.
            Err(RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        for buf in bufs.iter_mut() {
            if buf.should_flush(now) {
                worker.flush(buf);
            }
        }
    }

    for buf in bufs.iter_mut() {
        worker.flush(buf);
    }
    debug!("Meeting {} worker finished", meeting_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_is_detected_when_words_match() {
        let others = vec![(
            10_000,
            "we should ship the update on friday morning".to_string(),
        )];
        assert!(is_echo("ship the update on friday", &others, 12_000));
    }

    #[test]
    fn real_replies_are_not_echo() {
        let others = vec![(
            10_000,
            "we should ship the update on friday morning".to_string(),
        )];
        assert!(!is_echo(
            "sounds good i will prepare the release notes",
            &others,
            12_000
        ));
    }

    #[test]
    fn short_replies_and_old_context_are_kept() {
        let others = vec![(10_000, "okay thanks".to_string())];
        assert!(!is_echo("okay thanks", &others, 12_000)); // too short to judge
        let old = vec![(0, "we should ship the update on friday".to_string())];
        assert!(!is_echo("ship the update on friday", &old, 200_000));
    }

    #[test]
    fn silence_detection() {
        let mut s = vec![0.2f32; 16_000 * 11];
        s.extend(vec![0.0f32; 16_000]);
        assert!(tail_is_quiet(&s));
        assert!(!tail_is_quiet(&vec![0.2f32; 16_000]));
        assert!(rms(&[0.0; 100]) < SILENCE_RMS);
    }

    #[test]
    fn buffers_flush_on_length_and_gap() {
        let mut b = TrackBuf::new(Track::Others);
        assert!(!b.should_flush(Instant::now()));
        b.push(vec![0.3; 16_000], 5_000);
        assert_eq!(b.start_ms, 4_000);
        assert!(!b.should_flush(Instant::now()));
        assert!(b.should_flush(Instant::now() + Duration::from_secs(2)));
        let mut long = TrackBuf::new(Track::You);
        long.push(vec![0.3; (16_000.0 * MAX_FLUSH_SECS) as usize + 10], 40_000);
        assert!(long.should_flush(Instant::now()));
    }
}
