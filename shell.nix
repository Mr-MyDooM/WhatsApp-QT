{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    qt6.qtbase
    qt6.qtdeclarative
    qt6.qtwebengine
    qt6.qttools
    # Build tools needed for C++ compilation (Qt resources)
    cmake
    pkg-config
    gcc
  ];

  shellHook = ''
    # Add Qt tools to PATH - this is the KEY fix for rcc!
    export PATH="${pkgs.qt6.qtbase}/libexec:$PATH"
    export PATH="${pkgs.qt6.qttools}/bin:$PATH"

    # Set Qt environment variables for better integration
    export QT_PLUGIN_PATH="${pkgs.qt6.qtbase}/lib/qt-6/plugins:$QT_PLUGIN_PATH"
    export QML2_IMPORT_PATH="${pkgs.qt6.qtdeclarative}/lib/qt-6/qml:$QML2_IMPORT_PATH"

    # Verify system Rust toolchain
    if command -v cargo >/dev/null 2>&1; then
      echo "✅ System Cargo is available: $(cargo --version)"
    else
      echo "❌ System Cargo not found. Please install with: sudo zypper install rust cargo"
    fi

    if command -v rustc >/dev/null 2>&1; then
      echo "✅ System Rustc is available: $(rustc --version)"
    else
      echo "❌ System Rustc not found. Please install with: sudo zypper install rust cargo"
    fi

    # Verify Qt Resource Compiler is available
    echo "🔧 Checking Qt tools availability..."
    if command -v rcc >/dev/null 2>&1; then
      echo "✅ Qt Resource Compiler (rcc) is available: $(which rcc)"
      echo "📦 rcc version: $(rcc --version | head -n 1)"
    else
      echo "❌ rcc still not found in PATH"
      echo "🔍 Searching for rcc in nix store..."
      find /nix/store -name "rcc" -type f 2>/dev/null | head -5
    fi

    # Verify other important tools
    if command -v moc >/dev/null 2>&1; then
      echo "✅ Meta-Object Compiler (moc) is available"
    fi

  '';
}
