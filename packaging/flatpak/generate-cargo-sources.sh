#!/usr/bin/env bash
# Generate cargo-sources.json for offline Flatpak builds.
# Run this whenever Cargo.lock changes, then commit the updated cargo-sources.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

GENERATOR_URL="https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py"
GENERATOR="$SCRIPT_DIR/flatpak-cargo-generator.py"

if [ ! -f "$GENERATOR" ]; then
    echo "Downloading flatpak-cargo-generator.py..."
    curl -Lo "$GENERATOR" "$GENERATOR_URL"
    chmod +x "$GENERATOR"
fi

python3 "$GENERATOR" "$REPO_ROOT/Cargo.lock" -o "$SCRIPT_DIR/cargo-sources.json"
echo "cargo-sources.json written to $SCRIPT_DIR/cargo-sources.json"
echo "Commit this file alongside any Cargo.lock changes."
