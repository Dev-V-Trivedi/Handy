# Oshabe

Private, local-first voice dictation for your desktop. Hold a key, speak, and clean text appears in whatever app you are using.

Oshabe is an independent fork of [Handy](https://github.com/cjpais/Handy) by CJ Pais, used under the MIT license. It keeps Handy's local speech engines and adds smarter cleanup on top.

> Status: early. Built and tested mainly on desktop. See `docs/REBRAND-NOTES.md` for what is still being finished.

## What it does

- **Local speech to text.** Whisper and Parakeet style models run on your machine. Audio does not leave your computer unless you choose a cloud model.
- **Push to talk or toggle.** Global shortcut, works in any text field.
- **AI cleanup (optional).** Send the transcript to an LLM to remove filler words, fix punctuation and handle self-corrections. Use a local model through Ollama, or a cloud provider with your own key.
- **Flow Cleanup prompt.** A default prompt for dictation: strips fillers, keeps only the corrected version when you correct yourself, and leaves mixed Hindi, Telugu and English wording as spoken.
- **Context tone.** Detects the app you are dictating into (Slack, Outlook, VS Code and more) and adapts the cleanup style. Rules are editable in Settings, Post Process. Works on Windows, macOS and Linux X11.
- **Custom words.** Fuzzy correction for names and terms you use often.

## Roadmap

- Snippets: say a trigger phrase, paste saved text
- Voice edit commands on selected text ("make it shorter")
- Dictionary that learns from your corrections
- Bundled small local cleanup model, no Ollama needed
- Android and iOS keyboard apps (separate project)

## Install and build

Installers are built by GitHub Actions. Download from the Releases page once a release is published:

| OS      | File                          | Notes                                                             |
| ------- | ----------------------------- | ----------------------------------------------------------------- |
| Windows | `.exe` (NSIS) or `.msi`       | Unsigned: SmartScreen shows "More info", then "Run anyway"        |
| macOS   | `.dmg` (Apple Silicon, Intel) | Unsigned: drag to Applications, then right-click the app and Open |
| Linux   | `.AppImage`, `.deb`, `.rpm`   | AppImage: `chmod +x` and run                                      |

**Which file?** Test builds from the Actions tab are named by system:

- `oshabe-windows-x64`: normal Windows PCs (Intel or AMD). Unzip, open the `nsis` folder, run the setup `.exe`.
- `oshabe-windows-arm64`: only Windows on ARM (Snapdragon). On a normal PC it shows "This app can't run on your PC".
- `oshabe-macos-apple-silicon` (M1 and newer) and `oshabe-macos-intel`.
- `oshabe-linux-x64-deb`, `oshabe-linux-x64-appimage-rpm`, `oshabe-linux-arm64`.

Check your Windows CPU in Settings > System > About > System type.

To cut a release: run `bun run bump X.Y.Z`, push, then run the Release workflow (Actions tab). It creates a draft release with all installers attached. Publish the draft when ready. Signing turns on when the repo variable `SIGN_BINARIES` is `true` and the signing secrets exist.

Or build from source:

```bash
bun install
bun run tauri dev          # development
bun run tauri build        # production build
```

Prerequisites and per-platform setup are in [BUILD.md](BUILD.md). Required model setup is in [AGENTS.md](AGENTS.md).

Using AI cleanup with Ollama:

1. Install [Ollama](https://ollama.com) and pull a small model, for example `ollama pull qwen2.5:3b`.
2. In Oshabe, open Post Process, choose the Custom provider (default `http://localhost:11434/v1`), set the model name.
3. Pick the Flow Cleanup prompt and use the Post Process shortcut.

## Documentation

- [docs/UPSTREAM-README.md](docs/UPSTREAM-README.md): Handy's original README, still useful for Linux notes, troubleshooting and manual model installation
- [docs/REBRAND-NOTES.md](docs/REBRAND-NOTES.md): fork status and remaining to-dos

## License and credits

MIT. See [LICENSE](LICENSE). Copyright (c) 2025 CJ Pais, copyright (c) 2026 Dev Trivedi.

Oshabe exists because of [Handy](https://github.com/cjpais/Handy) and its contributors. Local transcription is built on transcribe.cpp and ggml; thanks to Georgi Gerganov and all contributors.

Built by [Hikari Webworks](https://hikariwebworks.studio).
