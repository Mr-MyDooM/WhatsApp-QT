{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Qt6, required by cxx-qt. The frontend uses Qt6-only QML APIs (`import QtCore`,
    # versionless `import QtQuick.Dialogs`), which is why this is not Qt5.
    qt6.qtbase
    qt6.qtdeclarative
    qt6.qttools
    # Now a BUILD-time dependency, not just runtime: build.rs links the
    # WebEngineQuick module for QtWebEngineQuick::initialize().
    # nixpkgs sometimes marks qtwebengine insecure; if evaluation fails, run with
    #   NIXPKGS_ALLOW_INSECURE=1 nix-shell --impure
    # or add it to permittedInsecurePackages in your nixpkgs config.
    qt6.qtwebengine

    # C++ build tools. No explicit compiler: mkShell's stdenv already provides
    # cc/c++ (gcc on Linux, clang on Darwin). Naming `gcc` here broke Darwin.
    cmake
    pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    # AppImage creation
    squashfsTools  # unsquashfs with zstd support, for extracting linuxdeploy

    # Qt's GTK platform theme integration. libappindicator is gone along with the
    # tray-item crate: the tray is now Qt.labs.platform.SystemTrayIcon in QML.
    gtk3
    glib
  ];

  shellHook = ''
    # qt-build-utils (via cxx-qt-build) locates Qt through $QMAKE.
    export QMAKE="${pkgs.qt6.qtbase.dev}/bin/qmake"
    export PATH="${pkgs.qt6.qtbase.dev}/bin:${pkgs.qt6.qtbase.dev}/libexec:${pkgs.qt6.qttools.dev}/bin:$PATH"
    export QT_PLUGIN_PATH="${pkgs.qt6.qtbase}/lib/qt-6/plugins:$QT_PLUGIN_PATH"
    # Qt6 reads QML_IMPORT_PATH; QML2_IMPORT_PATH is still honoured for compatibility.
    export QML_IMPORT_PATH="${pkgs.qt6.qtdeclarative}/lib/qt-6/qml:${pkgs.qt6.qtwebengine}/lib/qt-6/qml:$QML_IMPORT_PATH"
    export QML2_IMPORT_PATH="$QML_IMPORT_PATH"
    ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    export PKG_CONFIG_PATH="${pkgs.gtk3.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
    ''}
    echo "Qt: $(${pkgs.qt6.qtbase.dev}/bin/qmake --version | tail -1)"
    echo "Cargo: $(cargo --version 2>/dev/null || echo 'not found')"
  '';
}
