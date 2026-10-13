# Oshabe fork notes

Oshabe is a fork of Handy (MIT). This file tracks what was changed for the rebrand and what is still open.

## Done

- Product name, window title, tray, app identifier (`com.hikariwebworks.oshabe`), UI strings in all locales
- LICENSE keeps CJ Pais copyright, adds Dev Trivedi
- Upstream sponsor assets, donation link and `FUNDING.yml` removed
- Upstream code-signing command removed from `tauri.conf.json`
- Updater endpoint points to `Dev-V-Trivedi/oshabe` releases; `createUpdaterArtifacts` is `false`
- Interim text wordmark replaces the Handy logo and hand icon in the sidebar and onboarding

## Open (do before a public release)

1. **Name clearance.** "Oshabe" was only checked with web searches, which are noisy. Check domains, GitHub org and repo, App Store and Play Store, and trademark databases (India, WIPO, USPTO, software classes 9 and 42).
2. **Icons and logo.** `src-tauri/icons/`, tray icons, `public/` and installer images (`src-tauri/nsis`) are still Handy's. Replace them.
3. **Updater keys.** Run `tauri signer generate`, replace `plugins.updater.pubkey` in `tauri.conf.json` (it is still Handy's public key), store the private key as a CI secret, then set `createUpdaterArtifacts` back to `true`.
4. **Code signing.** Add your own Windows and macOS signing setup. `.github/workflows` still reference upstream repos, secrets and runners.
5. **Repository.** The new repo URL `Dev-V-Trivedi/oshabe` is used in the About page, the updater endpoint and issue templates. Create it or change the references.
6. **Model downloads.** The model catalog and manager still point at upstream hosted mirrors. They work, but you rely on someone else's bandwidth.
7. **Version number.** Still `0.9.8`, so the "What's new" dialog can show upstream release notes. Reset the version and clear `src/content/release-notes`.
8. **Docs.** `CONTRIBUTING*.md` and `.github/PULL_REQUEST_TEMPLATE.md` still describe Handy's contribution process.

## Intentionally not renamed

Renaming these would break builds or packaging for no user benefit: Cargo package `handy`, the `handy_keys` shortcut backend, `HANDY_*` environment variables, `/usr/lib/Handy` Linux package path, the portable mode marker text. 9. **Docs links.** The macOS secure input warning links to Handy's troubleshooting page (`handy.computer`). The advice still applies, but replace it with your own docs when they exist.
