# Oshabe roadmap and council notes

Status as of v0.3.0. Rust compiles and 305 unit tests pass in the dev sandbox, but nothing was run on a real Windows or Mac machine by the assistant. Done since v0.1.2: setup wizard, model filters, meeting notes (capture, transcript tidy-up, summary, Word and PDF export, retention, hotkey, tray, call detection), one-click Ollama models. Still open: real-hardware testing of meeting capture, bundled local model, signing, updater keys.

## Known problems (honest list)

- **Windows: "Transcription failed" (open, user confirmed).** Tested with Nemotron Streaming. Likely path: the live stream returns empty text, then the batch fallback (`tm.transcribe`) fails on a streaming-only model. This is a hypothesis. Need the exact toast text or the last `Transcription failed:` line in `%APPDATA%\com.hikariwebworks.oshabe\logs`. Workaround: try Parakeet (non-streaming).
- **No bundled local cleanup model yet.** Ollama works. Bundling llama.cpp inside the app is unsafe (see Local LLM).
- **Updater key is still Handy's.** Auto-update will not verify until own keys exist.
- **No code signing.** SmartScreen and Gatekeeper warn.
- **Model files come from `blob.handy.computer`** (upstream bandwidth) and Hugging Face. Mirror them before a public launch.
- **Rust dependencies not audited** (`cargo audit` not run). API keys sit in the settings file, not the OS keychain.
- **UI never visually checked on a real Windows or Mac window** by the assistant. Needs a human pass.

## Where data lives

- Recordings, transcripts (history DB), settings, downloaded models, logs: local app data folder (`%APPDATA%\com.hikariwebworks.oshabe`, macOS `~/Library/Application Support/com.hikariwebworks.oshabe`, Linux `~/.local/share/com.hikariwebworks.oshabe`). Portable mode keeps them next to the exe.
- Speech to text runs fully on the device. No telemetry or analytics code was found.
- Network use: model downloads, update check (GitHub), and cleanup only if a cloud provider is chosen (the transcript text, not audio, goes to that provider).
- Fully offline setup: local speech model + Apple Intelligence or Ollama for cleanup.

## Council (code-read scores, not a running app)

| Member                | Score | Main point                                                 |
| --------------------- | ----- | ---------------------------------------------------------- |
| Creative Director     | 78    | Distinct brand. Retina DMG art is soft.                    |
| Performance Architect | 70    | One large JS chunk. OK for desktop.                        |
| Usability Judge       | 62    | Onboarding shows model names first. Cleanup needs setup.   |
| Growth Strategist     | 55    | No landing page, no demo, no meetings story.               |
| Lead Engineer         | 66    | Rust uncompiled locally, thin tests, one open Windows bug. |

Top blockers: Windows failure, cleanup not out-of-box, onboarding speaks in model names.

## Done: onboarding redesign (v0.2.0)

Goal: user never sees a model list unless they ask.

1. Welcome: "What will you use it for?" Dictation / Meetings / Both.
2. Permissions: mic (with live test, done), accessibility on macOS.
3. Quality preset instead of models: Fast, Balanced, Best, plus language. App picks and downloads the model, shows size and one progress bar.
4. Try it: a practice box, press the hotkey, see your words appear.
5. Cleanup: Off / Local (offline) / Apple Intelligence / Cloud key. Default to Local once it exists.
6. Hotkey: confirm or change.
7. Done. "Advanced: choose models yourself" stays in Settings.

## Done (needs real-machine testing): meeting note taker (v0.3.0)

Works with Zoom, Meet, Teams, Slack huddles, any browser call, because it records the computer's audio. No bot joins the call.

- **Capture:** mic plus system audio. Windows: WASAPI loopback. macOS: ScreenCaptureKit (13+), needs Screen Recording permission. Linux: PulseAudio or PipeWire monitor source.
- **Speakers:** mic is "You", system audio is "Others". Speaker separation for several remote people uses the diarization support in the speech engine.
- **Long recordings:** chunk by voice activity, transcribe as it goes, save to the local history DB. Handle sleep and device changes.
- **Output:** transcript + summary + decisions + action items (owner, due date). Summary uses the local model or a chosen provider, JSON schema `{summary, decisions[], actionItems[]}`.
- **Meetings tab:** list, search, export (Markdown, copy, share).
- **Prompt to record:** detect a meeting app in the foreground (app detection already exists) and ask.
- **Consent:** show a visible recording banner. Recording laws vary by country and some require telling participants. Add a reminder in the UI.
- **Order:** Windows first, then macOS, then Linux. Needs real-machine testing on each.

## Done, untested on real downloads: built-in local cleanup model (v0.4.0)

Do not link llama.cpp into the app. The speech engine already contains ggml 0.26.0 and llama.cpp brings 0.24.0, same symbol names, one process. Instead:

- Cleanup models appear as download cards (Qwen 1.5B, Qwen 3B), same look as speech models.
- One extra download: the official `llama-server`, started on demand on 127.0.0.1 and stopped on exit.
- Cleanup goes through the same OpenAI-style path as Ollama, with raw-text fallback.
- Downloads checked against published SHA-256 (Hugging Face ETag, GitHub asset digest). The sandbox could not reach GitHub or Hugging Face, so the real download and server start still need one test on Windows and one on Mac.
- Server is stopped after 10 idle minutes and on exit.

## Later

- Voice commands: "delete that", "new line", snippets.
- Auto-learn dictionary from corrections.
- Own updater keys, then code signing.
- Landing page with a short demo.
- Mobile apps (iOS keyboard, Android IME) as separate projects.
