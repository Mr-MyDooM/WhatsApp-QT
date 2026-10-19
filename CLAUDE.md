# CLAUDE.md

WhatsApp-QT is a macOS-first desktop client that wraps WhatsApp Web (`https://web.whatsapp.com`) in a native Qt window: Rust backend, QML/Qt6 frontend, one tab per account, system tray, persistent settings.

## Build and Run

1. Get Qt6 (qtbase, qtdeclarative, **qtwebengine**, qttools) and a Rust toolchain. On macOS, `brew install qt` (Qt 6; `qt@5` does not work) and `export QMAKE="$(brew --prefix qt)/bin/qmake"`, because Homebrew Qt is keg-only. On Linux, install your distro's Qt6 dev packages and point `QMAKE` at the Qt6 `qmake`. Done when `$QMAKE` points at a Qt6 `qmake`: `cxx-qt-build` finds Qt only through that variable.
2. `cargo run`. Done when the main window opens.
3. For backend logs, run `RUST_LOG=debug cargo run`. Without `RUST_LOG`, `env_logger` prints errors only, so a silent run is not proof nothing went wrong.

QtWebEngine is a build-time dependency, not only a runtime one: `build.rs` links `WebEngineQuick` for `QtWebEngineQuick::initialize()`.

There is no CI. Verify builds locally, on macOS first.

## Architecture

### Rust Backend (`src/`)

- **`main.rs`**: entry point. Initializes logging, calls `whatsappqt_webengine_init()`, constructs `QGuiApplication` + `QQmlApplicationEngine`, loads QML from the Qt resource system, parses argv for deep links. Holds `find_data_dir()`, which locates `resources/` at runtime for the tray icon and `notification.rs`.
- **`cxxqt_object.rs`**: the `#[cxx_qt::bridge]` module defining `AppController` (every property, signal and invokable QML uses), settings (de)serialization, theme CSS generation, and `get_config_path()`. Settings live at `<config dir>/WhatsAppDesktop/settings.json` (`~/.config/WhatsAppDesktop/` on Linux); `README.md` names a different path and is wrong.
- **`notification.rs`**: native notifications, gated by `#[cfg(target_os)]`: `notify-rust` on Linux, `winrt-notification` on Windows, `osascript` on macOS.
- **`autostart.rs`**: start-on-login registration per platform. `set_enabled` is idempotent and returns errors to the caller, which surfaces them via `AppController::save_failed`.
- **`webengine_init.cpp`** / **`include/webengine_init.h`**: C++ shim for `QtWebEngineQuick::initialize()`, which must run before `QGuiApplication` is constructed or every `WebEngineView` aborts at startup. cxx-qt-lib has no binding for it.
- **`settings.rs`**, **`css.rs`**: empty and not declared as modules in `main.rs`, so they are not compiled. Put new settings or CSS code in `cxxqt_object.rs`, where that logic lives.

### QML Frontend (`qml/`)

- **`main.qml`**: main window, tab bar, theme switching (Light/Dark/System), signal wiring to `AppController`, and the `Qt.labs.platform` `SystemTrayIcon` (icon from `appController.tray_icon_path`). The tray is QML; there is no tray crate.
- **`components/WebTab.qml`**: one `WebEngineView` per account, each with its own `WebEngineProfile`, plus CSS injection that suppresses WhatsApp Web's own theming. Profile data lives at `<AppDataLocation>/WhatsApp-QT-Profile/whatsapp_tab_<index>`, keyed by tab index, and the tab list is not restored on restart. Both contradict "Account" in `GLOSSARY.md` (stable identity, persists across restarts); treat the glossary as the target and do not build new features on the index key.
- **`components/Sidebar.qml`**: account switcher with add/remove tab buttons.
- **`components/SettingsDialog.qml`**: settings UI; each control calls an `AppController` setter.
- **`styles/AppStyle.qml`**: color palette for Light and Dark themes.

### Qt/Rust Bridge

`src/cxxqt_object.rs` declares the `AppController` QObject; `build.rs` registers it into a QML module named `AppController` via `#[qml_element]`, which is what makes the bare `import AppController` and `AppController { id: appController }` work. It is instantiated in QML, not injected as a context property.

Rules when editing the bridge:

- **Keep declarations snake_case.** There is no `#[auto_cxx_name]`, so a Rust property `download_path` is `download_path` in QML, and signal `theme_changed` is handled as `onTheme_changed`. Why: the QML already uses these names.
- **Name each property's `NOTIFY` explicitly** (`NOTIFY = theme_changed`). Why: otherwise cxx-qt synthesises `themeChanged`, which the QML does not connect to.
- **Point each property's `WRITE` at the hand-written `#[qinvokable]` of the same name** (`WRITE = set_theme`). Why: cxx-qt then skips setter codegen, so there is no clash, and QML can still call `appController.set_theme(...)` directly.
- **Write receivers as `self: &AppController` or `self: Pin<&mut AppController>`**, not `&self`. Emitting a signal or writing a field needs the pinned form: `self.as_mut().rust_mut().theme = v; self.as_mut().theme_changed();`. Why: there is no `DerefMut`.
- **Expose values QML needs at startup as `CONSTANT` properties**, as `tray_icon_path` does. Why: cxx-qt-lib 0.7 binds no `QQmlContext::setContextProperty`.

### QML resource embedding (gotcha)

QML is compiled into the binary through the Qt resource system and loaded from `qrc:/qt/qml/AppController/qml/main.qml`. **List every new `.qml` file in the `qml_files` array in `build.rs`**: an unlisted file does not exist at runtime, even though it is on disk. The `qml/components/backup/` files are deliberately unlisted.

`cxx-qt-build` 0.7 generates a `qmldir` with no `singleton` declarations, so `pragma Singleton` in `qml/styles/AppStyle.qml` does not register it as a singleton. This is harmless only while nothing references `AppStyle` by name; code that starts referencing it by name will hit this limit.

## Platform Notes

- macOS: first-class support. Check every change here first. Package with `packaging/macos/build-macos-app.sh` (unsigned `.app` and optional `.dmg`; Gatekeeper warns on first launch). Code signing and notarization are out of scope.
- Linux: supported, second priority.
- Windows: best-effort. `winrt-notification` sits under `[target.'cfg(windows)'.dependencies]` because it does not compile elsewhere. Windows packaging waits for the UI stack decision (a move off Qt is planned).

`packaging/` scripts (deb, rpm, AppImage, Flatpak, macOS) run independently of `cargo build`. The Linux scripts expect system Qt6 with `QMAKE` set. `packaging/WhatsApp-QT.desktop` is shared by all Linux formats.

### Deep links (`whatsapp://`)

The scheme is declared in `packaging/WhatsApp-QT.desktop` (`MimeType=x-scheme-handler/whatsapp;` plus `%u` in `Exec=`) and in `packaging/macos/Info.plist` (`CFBundleURLTypes`). Windows needs an installer-time `HKCU\Software\Classes\whatsapp` key with a `URL Protocol` value; there is no Windows installer script to hook that into, so it is unimplemented.

`main()` parses argv for a `whatsapp:` / `https://wa.me/` argument (how Linux and Windows deliver it), and QML pulls it via `take_pending_deep_link()` to raise the window. **macOS delivers URLs as an Apple Event (`application:openURLs:`), not argv, so the macOS path is unimplemented**: cxx-qt-lib 0.7 exposes no `NSApplicationDelegate` hook. There is no single-instance IPC, so a link clicked while the app runs starts a second process.

## Cleanup backlog

- `rand`, `tokio`, `chrono`, `futures`, and `blocking` are declared in `Cargo.toml` but unused in `src/`. They predate the cxx-qt migration; `tokio` with `features = ["full"]` costs real build time. Drop them in a dedicated `chore:` commit whenever convenient; done when `cargo build` still passes.

## TBD

Features left out of the parity bar for the move off Qt. Each needs its own decision before work starts:

- Spellcheck toggle (system webviews have their own spellcheck).
- Custom download path (`download_path` is stored today but nothing uses it).
- Account rename (stub at `qml/main.qml:258`).
- Native Wayland on Linux (child webviews draw at wrong bounds; start with X11/XWayland via `GDK_BACKEND=x11`).

## Shared background

- Glossary: `GLOSSARY.md`. On term conflicts `GLOSSARY.md` wins. Record a new term there the moment it settles, so specs, tickets, and code names stay consistent.
- Decisions: `docs/adr/`. 0001: move from QtWebEngine to Tauri 2 on the system webview. Read it before you start work on the rewrite.

## Agent skills

### Issue tracker

Local markdown under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default roles, recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Agent guide

- New docs surfaces (`docs/adr/`, `docs/standards/`, `docs/business/`, `design-system/`, `.omc/skills/`, `scripts/`) are created on first real use, and get an index line in this file at that moment. Why: an empty folder reads as an unfilled obligation.
