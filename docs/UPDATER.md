# Auto-update keys

Oshabe checks `https://github.com/Dev-V-Trivedi/oshabe/releases/latest/download/latest.json` for updates. An update is only installed if its signature matches the public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).

That key is still Handy's, so updates from Oshabe cannot verify yet. This is safe (nothing installs) but auto-update does nothing until you do the one-time setup below. Do it on your own computer so the private key never leaves it.

## One-time setup

1. Generate a key pair (pick a password and remember it):

   ```bash
   bunx tauri signer generate -w ~/.tauri/oshabe.key
   ```

   This writes `oshabe.key` (private, keep secret, back it up) and `oshabe.key.pub` (public).

2. In GitHub, repo `Dev-V-Trivedi/oshabe`, Settings, Secrets and variables, Actions:
   - Secret `TAURI_SIGNING_PRIVATE_KEY`: the full contents of `oshabe.key`
   - Secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password
   - Variable `UPDATER_SIGNING`: `true` (do not set `SIGN_BINARIES`; that one also turns on Apple and Windows certificate steps, which fail without certificates)

3. Put the contents of `oshabe.key.pub` in `src-tauri/tauri.conf.json` as `plugins.updater.pubkey` (or send it to the assistant to do it), bump the version (`bun run bump X.Y.Z`), push.

4. Publish a release with the release workflow. It uploads `latest.json` and the signed installers. Install that version once by hand; every later version updates itself.

## Rules

- Never commit the private key or paste it into chat or issues.
- If it leaks or is lost, generate a new pair and ship one manual release; users must reinstall once.
