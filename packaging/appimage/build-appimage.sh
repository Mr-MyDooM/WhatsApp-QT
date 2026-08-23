#!/usr/bin/env bash
# Build a WhatsApp-QT AppImage.
# Requires: nix-shell (shell.nix in repo root), curl
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUILD_DIR="$REPO_ROOT/target/appimage"
APPDIR="$BUILD_DIR/AppDir"
TOOLS_DIR="$BUILD_DIR/tools"

# ── 1. Build release binary ──────────────────────────────────────────────────
echo "==> Building release binary..."
cd "$REPO_ROOT"
nix-shell --run "cargo build --release"

# ── 2. Stage AppDir ──────────────────────────────────────────────────────────
echo "==> Staging AppDir..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" \
         "$APPDIR/usr/share/whatsapp-qt" \
         "$APPDIR/usr/share/icons/hicolor/512x512/apps" \
         "$APPDIR/usr/share/applications"

cp "$REPO_ROOT/target/release/WhatsApp-QT" "$APPDIR/usr/bin/"
cp -r "$REPO_ROOT/qml"       "$APPDIR/usr/share/whatsapp-qt/qml"
cp -r "$REPO_ROOT/resources" "$APPDIR/usr/share/whatsapp-qt/resources"
cp "$REPO_ROOT/resources/icons/tray.png" \
   "$APPDIR/usr/share/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png"
cp "$REPO_ROOT/packaging/WhatsApp-QT.desktop" \
   "$APPDIR/usr/share/applications/io.github.mrdoom.WhatsAppQT.desktop"
cp "$REPO_ROOT/packaging/appimage/AppRun" "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"
ln -sf "usr/share/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png" \
       "$APPDIR/io.github.mrdoom.WhatsAppQT.png"

# ── 3. Download tools ────────────────────────────────────────────────────────
mkdir -p "$TOOLS_DIR"

fetch_tool() {
    local name="$1" url="$2"
    local dest="$TOOLS_DIR/$name"
    if [ ! -f "$dest" ]; then
        echo "==> Downloading $name..."
        curl -Lo "$dest" "$url"
        chmod +x "$dest"
    fi
}

fetch_tool linuxdeploy \
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
fetch_tool linuxdeploy-plugin-qt \
    "https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage"

# ── 4. Extract AppImages (bypasses AppImageLauncher binfmt_misc interception) ─
# nix's unsquashfs supports zstd; system squashfuse may not.
extract_tool() {
    local name="$1"
    local appimage="$TOOLS_DIR/$name"
    local dest="$TOOLS_DIR/${name}.appdir"
    if [ ! -d "$dest" ]; then
        echo "==> Extracting $name..."
        nix-shell "$REPO_ROOT/shell.nix" --run \
            "unsquashfs -d '$dest' '$appimage'"
    fi
}

extract_tool linuxdeploy
extract_tool linuxdeploy-plugin-qt

# ── 5. Bundle Qt libraries and produce AppImage ──────────────────────────────
echo "==> Bundling Qt libraries..."
QMAKE="$(nix-shell "$REPO_ROOT/shell.nix" --run 'echo $QMAKE')"
export OUTPUT="$REPO_ROOT/WhatsApp-QT-x86_64.AppImage"

# Symlink the Qt plugin where linuxdeploy expects it
PLUGIN_DIR="$TOOLS_DIR/linuxdeploy.appdir/usr/bin/plugins"
mkdir -p "$PLUGIN_DIR"
ln -sf "$TOOLS_DIR/linuxdeploy-plugin-qt.appdir/usr/bin/linuxdeploy-plugin-qt" \
       "$PLUGIN_DIR/linuxdeploy-plugin-qt"

nix-shell "$REPO_ROOT/shell.nix" --run "
    export QMAKE='$QMAKE'
    export OUTPUT='$OUTPUT'
    export LD_LIBRARY_PATH=\"\$QT_PLUGIN_PATH/../lib:\${LD_LIBRARY_PATH:-}\"
    '$TOOLS_DIR/linuxdeploy.appdir/AppRun' \
        --appdir '$APPDIR' \
        --plugin qt \
        --output appimage \
        --desktop-file '$APPDIR/usr/share/applications/io.github.mrdoom.WhatsAppQT.desktop' \
        --icon-file '$APPDIR/usr/share/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png'
"

echo ""
echo "AppImage: $REPO_ROOT/WhatsApp-QT-x86_64.AppImage"
