#!/usr/bin/env bash
# Build a WhatsApp-QT.app bundle (and optionally a .dmg) for macOS.
# Requires: Qt6 with QtWebEngine (nix-shell, or `brew install qt`), Rust toolchain.
# Homebrew's `qt` formula is Qt 6; `qt@5` is the old one and will not work since
# the migration to cxx-qt.
#
# NOTE: this produces an UNSIGNED bundle. Gatekeeper will refuse to open it on
# first launch ("damaged / unidentified developer"); right-click > Open, or
# `xattr -dr com.apple.quarantine WhatsApp-QT.app`. Code signing and
# notarization are deliberately out of scope (no Apple Developer cert).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | cut -d'"' -f2)"
BUILD_DIR="$REPO_ROOT/target/macos"
APP="$BUILD_DIR/WhatsApp-QT.app"
CONTENTS="$APP/Contents"

# ── 1. Build release binary ──────────────────────────────────────────────────
echo "==> Building release binary (v$VERSION)..."
cd "$REPO_ROOT"
if command -v nix-shell >/dev/null 2>&1; then
    nix-shell --run "cargo build --release"
else
    # Homebrew Qt is keg-only, so qmake is not on PATH by default.
    if [ -z "${QMAKE:-}" ] && [ -x "$(brew --prefix qt 2>/dev/null)/bin/qmake" ]; then
        export QMAKE="$(brew --prefix qt)/bin/qmake"
    fi
    cargo build --release
fi

# ── 2. Stage the bundle ──────────────────────────────────────────────────────
# find_data_dir() in src/main.rs resolves data as <bin dir>/../share/whatsapp-qt,
# which from Contents/MacOS lands in Contents/share/whatsapp-qt.
echo "==> Staging app bundle..."
rm -rf "$APP"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources" "$CONTENTS/share/whatsapp-qt"

cp "$REPO_ROOT/target/release/WhatsApp-QT" "$CONTENTS/MacOS/"
cp -r "$REPO_ROOT/qml"       "$CONTENTS/share/whatsapp-qt/qml"
cp -r "$REPO_ROOT/resources" "$CONTENTS/share/whatsapp-qt/resources"

sed "s/@VERSION@/$VERSION/g" "$REPO_ROOT/packaging/macos/Info.plist" \
    > "$CONTENTS/Info.plist"

# ── 3. Icon ──────────────────────────────────────────────────────────────────
# Same source icon as the Linux packages (resources/icons/tray.png).
echo "==> Building .icns..."
ICONSET="$BUILD_DIR/WhatsApp-QT.iconset"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z $size $size "$REPO_ROOT/resources/icons/tray.png" \
        --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    sips -z $((size * 2)) $((size * 2)) "$REPO_ROOT/resources/icons/tray.png" \
        --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$CONTENTS/Resources/WhatsApp-QT.icns"
rm -rf "$ICONSET"

# ── 4. Optional .dmg ─────────────────────────────────────────────────────────
if command -v hdiutil >/dev/null 2>&1; then
    echo "==> Building .dmg..."
    DMG="$BUILD_DIR/WhatsApp-QT-${VERSION}.dmg"
    rm -f "$DMG"
    hdiutil create -quiet -volname "WhatsApp QT" -srcfolder "$APP" \
        -ov -format UDZO "$DMG"
    echo ""
    echo ".dmg: $DMG"
else
    echo "==> hdiutil not found, skipping .dmg"
fi

echo ""
echo ".app: $APP"
echo "Run: open '$APP'   (first launch: right-click > Open, unsigned build)"
