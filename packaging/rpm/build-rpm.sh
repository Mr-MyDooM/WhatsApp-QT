#!/usr/bin/env bash
# Build a .rpm package for WhatsApp-QT.
# Requires: rpm-build (sudo apt install rpm OR sudo zypper install rpm-build)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SPEC="$REPO_ROOT/packaging/rpm/whatsapp-qt.spec"
RPMBUILD_DIR="$REPO_ROOT/target/rpmbuild"

if ! command -v rpmbuild >/dev/null 2>&1; then
    echo "rpmbuild not found."
    echo "  Debian/Ubuntu: sudo apt install rpm"
    echo "  openSUSE:      sudo zypper install rpm-build"
    exit 1
fi

# ── 1. Build ─────────────────────────────────────────────────────────────────
echo "==> Building release binary..."
cd "$REPO_ROOT"
cargo build --release

# ── 2. Stage sources for rpmbuild ────────────────────────────────────────────
echo "==> Staging sources..."
mkdir -p "$RPMBUILD_DIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

cp "$REPO_ROOT/target/release/WhatsApp-QT"    "$RPMBUILD_DIR/SOURCES/WhatsApp-QT"
cp -r "$REPO_ROOT/qml"                         "$RPMBUILD_DIR/SOURCES/qml"
cp -r "$REPO_ROOT/resources"                   "$RPMBUILD_DIR/SOURCES/resources"
cp "$REPO_ROOT/resources/icons/tray.png"       "$RPMBUILD_DIR/SOURCES/tray.png"
cp "$REPO_ROOT/packaging/WhatsApp-QT.desktop"  "$RPMBUILD_DIR/SOURCES/WhatsApp-QT.desktop"
cp "$SPEC"                                      "$RPMBUILD_DIR/SPECS/whatsapp-qt.spec"

# ── 3. Build RPM ─────────────────────────────────────────────────────────────
echo "==> Building .rpm..."
rpmbuild -bb \
    --define "_topdir $RPMBUILD_DIR" \
    "$RPMBUILD_DIR/SPECS/whatsapp-qt.spec"

RPM_FILE=$(find "$RPMBUILD_DIR/RPMS" -name "*.rpm" | head -1)
cp "$RPM_FILE" "$REPO_ROOT/target/"

echo ""
echo ".rpm: $REPO_ROOT/target/$(basename "$RPM_FILE")"
echo "Install: sudo rpm -i target/$(basename "$RPM_FILE")"
echo "  or:    sudo zypper install target/$(basename "$RPM_FILE")"
