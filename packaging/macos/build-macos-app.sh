#!/usr/bin/env bash
# Build the macOS .app and .dmg with the Tauri bundler.
# Requires: Rust toolchain and the Tauri CLI
#   cargo install tauri-cli --version "^2" --locked
#
# The bundle is ad-hoc signed (bundle.macOS.signingIdentity "-" in
# tauri.conf.json), which gives it a stable bundle ID for notifications.
# It is not notarized: on first launch macOS refuses it. Open it once, then
# System Settings > Privacy & Security > Open Anyway.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

cargo tauri build

BUNDLE_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}/release/bundle"
echo ""
echo ".app: $BUNDLE_DIR/macos"
echo ".dmg: $BUNDLE_DIR/dmg"
