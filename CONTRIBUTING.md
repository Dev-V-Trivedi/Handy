# Contributing to Oshabe

Oshabe is a free, local-first dictation app (MIT). It is a fork of [Handy](https://github.com/cjpais/Handy) by CJ Pais and is developed independently. Contributions are welcome: bug fixes, features, translations, docs and testing on real machines.

## Ground rules

- **Local first.** Speech to text runs on the device. Anything that sends audio or text off the device must be opt-in and clearly labelled.
- **Never break paste.** If an optional step (cleanup, a model call) fails, fall back to the raw text.
- **One change per pull request**, with a clear why.
- **Be kind.** See the [Code of Conduct](CODE_OF_CONDUCT.md).

## Set up

Prerequisites: [Rust](https://rustup.rs/) (stable) and [Bun](https://bun.sh/). Platform build tools are in [BUILD.md](BUILD.md).

```bash
git clone https://github.com/Dev-V-Trivedi/oshabe.git
cd oshabe
bun install
mkdir -p src-tauri/resources/models
curl -L -o src-tauri/resources/models/silero-vad-v6.2-F32.gguf https://huggingface.co/handy-computer/silero-vad-v6.2-gguf/resolve/main/silero-vad-v6.2-F32.gguf
bun run tauri dev
```

On macOS, if cmake complains: `CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev`.

## Before you open a pull request

Run all of these. CI runs them too.

```bash
bunx tsc --noEmit
bun run lint
bunx prettier --check .
bun run check:translations
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

- Every new UI string goes in `src/i18n/locales/en/translation.json` and all other locales (English placeholder text is fine).
- Bump the version with `bun run bump X.Y.Z` if your change ships (maintainers can do this at merge).
- Use conventional commit prefixes: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`. Explain why, not what.

## Where things live

See [AGENTS.md](AGENTS.md) for the architecture map (Rust backend in `src-tauri/src`, React frontend in `src`). Roadmap and known problems: [docs/ROADMAP.md](docs/ROADMAP.md).

## Testing help wanted

The most useful thing you can do is run Oshabe on your machine and report what breaks, especially Windows, macOS and Linux paste behaviour across apps. Use the bug report template and attach the log (Settings > About shows the log folder).

## Translations

See [CONTRIBUTING_TRANSLATIONS.md](CONTRIBUTING_TRANSLATIONS.md).

## Security

Do not open a public issue for a vulnerability. See [SECURITY.md](SECURITY.md).
