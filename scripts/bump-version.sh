#!/usr/bin/env bash
# Usage: scripts/bump-version.sh 0.1.2
# Updates every place the app version lives. Run on every update you push.
set -euo pipefail
v="${1:?usage: bump-version.sh X.Y.Z}"
[[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version: $v" >&2; exit 1; }
cd "$(dirname "$0")/.."
sed -i -E '0,/"version": "[^"]*"/s//"version": "'"$v"'"/' package.json src-tauri/tauri.conf.json
sed -i -E '0,/^version = "[^"]*"/s//version = "'"$v"'"/' src-tauri/Cargo.toml
# Cargo.lock: only the app crate ("handy")
sed -i -E '/^name = "handy"$/{n;s/^version = "[^"]*"/version = "'"$v"'"/}' src-tauri/Cargo.lock
grep -m1 '"version"' package.json src-tauri/tauri.conf.json
grep -m1 '^version' src-tauri/Cargo.toml
grep -A1 '^name = "handy"$' src-tauri/Cargo.lock
