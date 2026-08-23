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

- **`main.rs`**: The core of the application. Defines `AppController` (a `QObject` subclass via `qmetaobject`), registers it with the QML engine, exposes the QML sources and the tray icon path as context
  properties, and drives the Qt event loop. The system tray itself lives in QML (see below).
- **`notification.rs`**: Platform-specific notification delivery. Uses `notify-rust` on Linux, `winrt-notification` on Windows, platform-gated with `#[cfg(target_os)]`.
- **`qml_resources.rs`**: Embeds QML source files as string constants via `include_str!` for loading at runtime.
- **`settings.rs`**: Empty placeholder; settings serialization is handled inline in `main.rs` using `serde_json`.

### QML Frontend (`qml/`)

- **`main.qml`**: Main window, tab bar, theme switching (Light/Dark/System), signal wiring to
  `AppController`, and the `Qt.labs.platform` `SystemTrayIcon` (Show/Hide + Quit, icon from the
  `trayIconPath` context property set in `main.rs`).
- **`components/WebTab.qml`**: One `WebEngineView` per account, with per-tab `WebEngineProfile` for isolation and CSS injection to suppress WhatsApp Web's own theming.
- **`components/Sidebar.qml`**: Account-switcher sidebar with add/remove tab buttons.
- **`components/SettingsDialog.qml`**: Settings UI for theme, download path, and notification toggles.
- **`styles/AppStyle.qml`**: Centralized color palette for Light and Dark themes.

### Qt/Rust Bridge

QML and Rust communicate through `qmetaobject` signals and slots. `AppController` is exposed to QML as a context property. QML invokes slots (methods) on the controller; the controller emits signals that QML connects to for UI updates.

## Key Dependencies

| Crate | Role |
|-------|------|
| `qmetaobject` | Qt QObject/signal/slot bindings for Rust |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Settings persistence |
| `notify-rust` | Linux desktop notifications |
| `log` / `env_logger` | Leveled backend logging, filtered via `RUST_LOG` |

## Development Environment

A `shell.nix` is provided for Nix users. Required system packages: Qt6 (qtbase, qtdeclarative, qtwebengine, qttools), GTK3, and a Rust toolchain.

The `build.rs` script triggers rebuilds when `resources.qrc` or any file under `qml/` changes.

## Platform Notes

- Linux: first-class support
- Windows: best-effort (some notification paths are conditional). `winrt-notification` is declared under `[target.'cfg(windows)'.dependencies]` because it will not compile elsewhere.
- macOS: supported. Build with `nix-shell --run "cargo build --release"` or with system Qt5 (`brew install qt@5`, then `export QMAKE="$(brew --prefix qt@5)/bin/qmake"`). Package with `packaging/macos/build-macos-app.sh`.

## Packaging

`packaging/` holds distro packaging scripts, invoked independently of `cargo build`:

- `packaging/deb/build-deb.sh` — Debian package
- `packaging/rpm/whatsapp-qt.spec` + `build-rpm.sh` — RPM package
- `packaging/appimage/build-appimage.sh` + `AppRun` — AppImage
- `packaging/flatpak/io.github.mrdoom.WhatsAppQT.yml` + `generate-cargo-sources.sh` — Flatpak manifest and vendored Cargo sources
- `packaging/macos/build-macos-app.sh` + `Info.plist` — unsigned `.app` bundle and optional `.dmg` (Gatekeeper will warn on first launch)
- `packaging/WhatsApp-QT.desktop` — shared desktop entry used across the Linux package formats
