# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run

```bash
cargo run                # Build and run in development mode
cargo build --release    # Optimized release build (LTO, size-optimized, stripped)
RUST_LOG=debug cargo run # Run with verbose backend logging
```

Backend logging goes through the `log` crate with the `env_logger` backend, initialized at
the top of `main()`. Set `RUST_LOG` (`error`, `warn`, `info`, `debug`, `trace`) to control it;
with no `RUST_LOG` set, only errors are printed.

The release profile uses `opt-level = "z"` (size), `lto = true`, and `strip = "debuginfo"` for a small binary.

## Project Overview

WhatsApp-QT is a Linux-first desktop client that wraps WhatsApp Web (`https://web.whatsapp.com`) in a native Qt window. It is built with a **Rust backend** and a **QML/Qt6 frontend**, with multi-account tab support, system tray integration, and persistent settings.

Settings are saved to the platform config dir under `WhatsAppDesktop/settings.json`
(e.g. `~/.config/WhatsAppDesktop/settings.json` on Linux), per `get_config_path()` in `main.rs`.

## Architecture

### Rust Backend (`src/`)

- **`main.rs`**: Entry point only. Initializes logging, calls `whatsappqt_webengine_init()`, constructs
  `QGuiApplication` + `QQmlApplicationEngine`, loads the QML entry point from the Qt resource system,
  and runs the event loop. Also holds `find_data_dir()`, which locates `resources/` at runtime
  (used by the tray icon and `notification.rs`).
- **`cxxqt_object.rs`**: The `#[cxx_qt::bridge]` module defining `AppController` — every property,
  signal and invokable QML talks to — plus settings (de)serialization and theme CSS generation.
- **`notification.rs`**: Platform-specific notification delivery. Uses `notify-rust` on Linux, `winrt-notification` on Windows, platform-gated with `#[cfg(target_os)]`.
- **`webengine_init.cpp`** / **`include/webengine_init.h`**: Minimal C++ shim exposing
  `QtWebEngineQuick::initialize()`, which *must* run before `QGuiApplication` is constructed or
  every `WebEngineView` aborts at startup. cxx-qt-lib has no binding for it.
- **`settings.rs`**, **`css.rs`**: Empty placeholders; settings serialization and CSS generation both
  live in `cxxqt_object.rs`.

### QML Frontend (`qml/`)

- **`main.qml`**: Main window, tab bar, theme switching (Light/Dark/System), signal wiring to
  `AppController`, and the `Qt.labs.platform` `SystemTrayIcon` (Show/Hide + Quit, icon from the
  `appController.tray_icon_path` property).
- **`components/WebTab.qml`**: One `WebEngineView` per account, with per-tab `WebEngineProfile` for isolation and CSS injection to suppress WhatsApp Web's own theming.
- **`components/Sidebar.qml`**: Account-switcher sidebar with add/remove tab buttons.
- **`components/SettingsDialog.qml`**: Settings UI for theme, download path, and notification toggles.
- **`styles/AppStyle.qml`**: Centralized color palette for Light and Dark themes.

### Qt/Rust Bridge

QML and Rust communicate through **cxx-qt** (Qt6). `src/cxxqt_object.rs` contains a
`#[cxx_qt::bridge]` module declaring the `AppController` QObject; `build.rs` registers it into a QML
module named `AppController` via `#[qml_element]`, which is what makes the QML's bare
`import AppController` and `AppController { id: appController }` work. It is instantiated *in QML*,
not injected as a context property.

Conventions that matter when editing the bridge:

- **Names are not case-converted.** There is no `#[auto_cxx_name]` on the extern block, so a Rust
  property `download_path` is `download_path` in QML, and a signal `theme_changed` is handled as
  `onTheme_changed`. Keep declarations snake_case to match the existing QML.
- **Each property's `NOTIFY` is named explicitly** (`NOTIFY = theme_changed`) rather than letting
  cxx-qt synthesise `themeChanged`, because the QML already connects to the snake_case names.
- **Each property's `WRITE` points at the hand-written `#[qinvokable]` of the same name**
  (`WRITE = set_theme`). cxx-qt skips setter codegen for a custom `WRITE`, so there is no clash —
  and QML keeps being able to call `appController.set_theme(...)` directly.
- **Receivers are `self: &AppController` or `self: Pin<&mut AppController>`** — never `&self`.
  Emitting a signal or writing a field needs the pinned mutable form:
  `self.as_mut().rust_mut().theme = v; self.as_mut().theme_changed();`.
- `tray_icon_path` is a `CONSTANT` property on `AppController` rather than a context property,
  because **cxx-qt-lib 0.7 binds no `QQmlContext::setContextProperty` equivalent**.

### QML resource embedding (gotcha)

QML is **compiled into the binary** through the Qt resource system by `cxx-qt-build`, and loaded from
`qrc:/qt/qml/AppController/qml/main.qml`. Every `.qml` file must be listed in the `qml_files` array in
`build.rs`; **a new `.qml` file that is not listed there does not exist at runtime**, no matter that it
is on disk. The `qml/components/backup/` files are deliberately not listed.

Known limitation: `cxx-qt-build` 0.7 generates a `qmldir` with no `singleton` declarations, so a
`pragma Singleton` QML file (`qml/styles/AppStyle.qml`) cannot be registered as a proper QML
singleton. This is currently harmless only because nothing references `AppStyle` by name.

## Key Dependencies

| Crate | Role |
|-------|------|
| `cxx-qt` / `cxx-qt-lib` | Qt6 QObject/signal/slot bindings for Rust (`qt_full` feature) |
| `cxx-qt-build` | Build-time bridge codegen, QML module embedding, Qt linkage |
| `tokio` | Async runtime (currently unused — see Cleanup below) |
| `serde` / `serde_json` | Settings persistence |
| `notify-rust` | Linux desktop notifications |
| `log` / `env_logger` | Leveled backend logging, filtered via `RUST_LOG` |

## Development Environment

A `shell.nix` is provided for Nix users. Required system packages: Qt6 (qtbase, qtdeclarative,
qtwebengine, qttools) and a Rust toolchain. GTK3 is only there for Qt's GTK platform theme —
the tray no longer needs libappindicator.

`cxx-qt-build` locates Qt through the `$QMAKE` environment variable, which `shell.nix` sets.

**QtWebEngine is now a build-time dependency**, not just a runtime one: `build.rs` calls
`.qt_module("WebEngineQuick")` to link it for `QtWebEngineQuick::initialize()`. nixpkgs sometimes
marks `qt6.qtwebengine` insecure; if `nix-shell` refuses to evaluate, use
`NIXPKGS_ALLOW_INSECURE=1 nix-shell --impure` or add it to `permittedInsecurePackages`.

`build.rs` handles its own rebuild tracking via `cxx-qt-build` (bridge files listed in `rust_files`,
QML in `qml_files`).

## Cleanup backlog

`rand`, `tokio`, `chrono`, `futures`, and `blocking` are declared in `Cargo.toml` but referenced
nowhere in `src/`. They predate the cxx-qt migration and were left in place to keep that migration's
diff scoped; `tokio = { features = ["full"] }` in particular costs real build time. Safe to drop.

## Platform Notes

- Linux: first-class support
- Windows: best-effort (some notification paths are conditional). `winrt-notification` is declared under `[target.'cfg(windows)'.dependencies]` because it will not compile elsewhere.
- macOS: supported. Build with `nix-shell --run "cargo build --release"` or with system Qt6 (`brew install qt`, then `export QMAKE="$(brew --prefix qt)/bin/qmake"`). Homebrew's `qt` formula is Qt 6; `qt@5` will not work post-migration. Package with `packaging/macos/build-macos-app.sh`.

## Packaging

`packaging/` holds distro packaging scripts, invoked independently of `cargo build`:

- `packaging/deb/build-deb.sh` — Debian package
- `packaging/rpm/whatsapp-qt.spec` + `build-rpm.sh` — RPM package
- `packaging/appimage/build-appimage.sh` + `AppRun` — AppImage
- `packaging/flatpak/io.github.mrdoom.WhatsAppQT.yml` + `generate-cargo-sources.sh` — Flatpak manifest and vendored Cargo sources
- `packaging/macos/build-macos-app.sh` + `Info.plist` — unsigned `.app` bundle and optional `.dmg` (Gatekeeper will warn on first launch)
- `packaging/WhatsApp-QT.desktop` — shared desktop entry used across the Linux package formats
