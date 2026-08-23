# AGENTS.md

## Build Commands

```bash
cargo run                # Development
cargo build --release   # Optimized release (LTO, size-optimized)
```

## Key Facts

- **QML not compiled**: Files in `qml/` are embedded via `include_str!` at runtime, not via `resources.qrc`. `build.rs` triggers rebuilds on `qml/` changes.
- **Settings path**: `~/.config/WhatsApp-QT/settings.json`
- **Entry point**: `src/main.rs` defines `AppController` via `qmetaobject`, exposed to QML as context property.
- **Platform gates**: Use `#[cfg(target_os = "linux")]` / `#[cfg(target_os = "windows")]` for OS-specific code (notification.rs).
- **Edition 2024**: Requires nightly Rust.

## Project Structure

```
src/           # Rust (main.rs, notification.rs, qml_resources.rs)
qml/           # QML frontend (main.qml, components/, styles/)
resources/     # Icons, tray images
```

## Dependencies (Key)

| Crate | Role |
|-------|------|
| qmetaobject | Qt bindings (QObject, signals, slots) |
| cxx-qt | Rust/C++ Qt bridging |
| notify-rust | Linux notifications |
| winrt-notification | Windows notifications |
| tray-item | System tray |