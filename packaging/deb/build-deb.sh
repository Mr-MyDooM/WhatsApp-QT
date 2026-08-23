#!/usr/bin/env bash
# Build a .deb package for WhatsApp-QT.
# Requires: dpkg-deb (comes with dpkg, present on all Debian/Ubuntu systems)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="0.1.0"
ARCH="amd64"
PKG_NAME="whatsapp-qt_${VERSION}_${ARCH}"
STAGING="$REPO_ROOT/target/deb/$PKG_NAME"

# ── 1. Build ─────────────────────────────────────────────────────────────────
echo "==> Building release binary..."
cd "$REPO_ROOT"
nix-shell --run "cargo build --release"

# ── 2. Stage package tree ─────────────────────────────────────────────────────
echo "==> Staging package..."
rm -rf "$STAGING"
mkdir -p "$STAGING/DEBIAN"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/share/whatsapp-qt"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/usr/share/icons/hicolor/512x512/apps"

cp "$REPO_ROOT/target/release/WhatsApp-QT"   "$STAGING/usr/bin/"
cp -r "$REPO_ROOT/qml"                        "$STAGING/usr/share/whatsapp-qt/qml"
cp -r "$REPO_ROOT/resources"                  "$STAGING/usr/share/whatsapp-qt/resources"
cp "$REPO_ROOT/packaging/WhatsApp-QT.desktop" \
   "$STAGING/usr/share/applications/io.github.mrdoom.WhatsAppQT.desktop"
cp "$REPO_ROOT/resources/icons/tray.png" \
   "$STAGING/usr/share/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png"

# ── 3. Control file ───────────────────────────────────────────────────────────
INSTALLED_SIZE=$(du -sk "$STAGING/usr" | cut -f1)

cat > "$STAGING/DEBIAN/control" <<EOF
Package: whatsapp-qt
Version: $VERSION
Section: net
Priority: optional
Architecture: $ARCH
Installed-Size: $INSTALLED_SIZE
Depends: libqt6core6t64 | libqt6core6, libqt6gui6, libqt6qml6, libqt6quick6, libqt6webenginequick6, qml6-module-qtquick-controls, qml6-module-qtquick-dialogs, qml6-module-qtwebengine, qml6-module-qt-labs-platform
Maintainer: MJ <mritunjay.rajput@armakuni.com>
Homepage: https://github.com/mrdoom/whatsapp-qt
Description: WhatsApp desktop client for Linux
 A Linux-first, privacy-friendly WhatsApp desktop client built with
 Rust and Qt. Supports multiple accounts via browser-like tabs,
 light/dark/system themes, and system tray integration.
EOF

cat > "$STAGING/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
chmod 755 "$STAGING/DEBIAN/postinst"

# ── 4. Build .deb ─────────────────────────────────────────────────────────────
echo "==> Building .deb..."
dpkg-deb --build --root-owner-group "$STAGING" "$REPO_ROOT/target/deb/"

echo ""
echo ".deb: $REPO_ROOT/target/deb/${PKG_NAME}.deb"
echo "Install: sudo dpkg -i target/deb/${PKG_NAME}.deb"
