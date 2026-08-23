# AGENTS.md

## Build Commands

```bash
cargo run                # Development
cargo build --release   # Optimized release (LTO, size-optimized)
RUST_LOG=debug cargo run # Verbose backend logging
```

Needs Qt6 (qtbase, qtdeclarative, **qtwebengine**, qttools). `cxx-qt-build` finds Qt via
`$QMAKE`; `shell.nix` sets it.

## Key Facts

- **QML IS compiled in**: every `.qml` file is embedded in the binary through the Qt resource
  system by `cxx-qt-build`. A new `.qml` file must be added to `qml_files` in `build.rs` or it
  does not exist at runtime. Entry point is `qrc:/qt/qml/AppController/qml/main.qml`.
- **Entry point**: `src/main.rs` is just `main()` + `find_data_dir()`. The `AppController` QObject
  lives in `src/cxxqt_object.rs` as a `#[cxx_qt::bridge]`, registered as a QML type via
  `#[qml_element]` and instantiated *in QML* (`AppController { id: appController }`), not as a
  context property — cxx-qt-lib 0.7 has no `setContextProperty`.
- **Bridge naming**: no `#[auto_cxx_name]`, so names are NOT case-converted. Rust `download_path`
  is `download_path` in QML; signal `theme_changed` is handled as `onTheme_changed`. Keep it
  snake_case to match the existing QML.
- **Bridge receivers**: `self: &AppController` or `self: Pin<&mut AppController>`, never `&self`.
  Writes go through `self.as_mut().rust_mut().field = v` (there is no `DerefMut`).
- **QtWebEngineQuick::initialize()** must run before `QGuiApplication` — see
  `include/webengine_init.h` and the call at the top of `main()`. Removing it aborts every
  `WebEngineView` at startup.
- **Settings path**: `~/.config/WhatsAppDesktop/settings.json` (see `get_config_path()`).
- **Platform gates**: `#[cfg(target_os = "...")]` for OS-specific code (`notification.rs`).
- **Edition 2024**: requires Rust 1.85+.

## Project Structure

```
src/           # Rust: main.rs, cxxqt_object.rs (the bridge), notification.rs, webengine_init.cpp
include/       # C++ headers for the bridge (webengine_init.h)
qml/           # QML frontend (main.qml, components/, styles/)
resources/     # Icons, tray images
build.rs       # CxxQtBuilder: bridge codegen + QML module + Qt linkage
```

## Dependencies (Key)

| Crate | Role |
|-------|------|
| cxx-qt / cxx-qt-lib | Qt6 bindings (QObject, properties, signals, invokables) |
| cxx-qt-build | Build-time codegen, QML module embedding, Qt linkage |
| notify-rust | Linux notifications |
| winrt-notification | Windows notifications (Windows-only dep) |

The tray is `Qt.labs.platform.SystemTrayIcon` in QML — there is no tray crate.
