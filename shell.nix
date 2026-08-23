{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Qt5 required by qmetaobject crate (compile-time only)
    qt5.qtbase
    qt5.qtdeclarative
    qt5.qttools

    # C++ build tools. No explicit compiler: mkShell's stdenv already provides
    # cc/c++ (gcc on Linux, clang on Darwin). Naming `gcc` here broke Darwin.
    cmake
    pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    # AppImage creation
    squashfsTools  # unsquashfs with zstd support, for extracting linuxdeploy

    # GTK and tray support (Linux-only; tray-item uses libappindicator here,
    # and Cocoa on macOS which needs no extra nixpkgs inputs)
    gtk3
    glib
    libappindicator-gtk3
  ];

  shellHook = ''
    # Point qmetaobject's build script to the nix Qt5 qmake
    export QMAKE="${pkgs.qt5.qtbase.dev}/bin/qmake"
    export PATH="${pkgs.qt5.qtbase.dev}/bin:${pkgs.qt5.qttools.dev}/bin:$PATH"
    export QT_PLUGIN_PATH="${pkgs.qt5.qtbase}/lib/qt-5/plugins:$QT_PLUGIN_PATH"
    export QML2_IMPORT_PATH="${pkgs.qt5.qtdeclarative}/lib/qt-5/qml:$QML2_IMPORT_PATH"
    ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    export PKG_CONFIG_PATH="${pkgs.gtk3.dev}/lib/pkgconfig:${pkgs.libappindicator-gtk3}/lib/pkgconfig:$PKG_CONFIG_PATH"
    ''}
    echo "Qt: $(${pkgs.qt5.qtbase.dev}/bin/qmake --version | tail -1)"
    echo "Cargo: $(cargo --version 2>/dev/null || echo 'not found')"

    echo ""
    echo "Note: qtwebengine is NOT in this shell (marked insecure in nixpkgs)."
    echo "WebEngineView requires QtWebEngine installed on the host system at runtime."
    echo "On Debian/Ubuntu: sudo apt install qml-module-qtwebengine"
    echo "On openSUSE:      sudo zypper install libqt5-qtwebengine"
    echo "On macOS:         brew install qt@5  (includes QtWebEngine)"
  '';
}
