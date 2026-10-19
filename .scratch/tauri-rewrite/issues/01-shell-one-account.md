# 01: Shell: one account in a Tauri window

**What to build:** the app starts as a Tauri 2 app instead of Qt. One window shows a static HTML sidebar and one account webview with WhatsApp Web. Typing, input methods, the emoji picker and copy/paste work. The core module (pure Rust, no Tauri types, one entry point: event in, effects out) exists and loads the old `settings.json`. See spec: `.scratch/tauri-rewrite/spec.md` and ADR 0001.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The Qt, cxx-qt and QtWebEngine build is replaced by Tauri 2, pinned to an exact version, with the `unstable` feature. The old Qt source files stay on disk, unbuilt, until ticket 10.
- [ ] One main window: static sidebar child webview on the left (no npm, no framework), one account child webview filling the rest, bounds updated on resize.
- [ ] The account webview loads `https://web.whatsapp.com` with the fixed Safari user agent from the spec.
- [ ] On macOS the account webview has its own data store identifier; on Linux and Windows its own data directory.
- [ ] Tauri's default macOS menu (with Edit) is kept.
- [ ] Core module: an old `settings.json` (with `tabs`, `current_tab`, nested `notification_settings`) loads; missing fields get safe defaults (zoom 1.0, spellcheck on, autostart off). Covered by `cargo test`, carrying over the cases of the current settings round-trip test.
- [ ] `cargo build` and `cargo test` exit 0.
- [ ] Manual macOS check: the QR code shows (no "WhatsApp works with Safari 15+"); fast typing, arrows, backspace, Option+e accent, another input language, Ctrl+Cmd+Space emoji picker, Cmd+C and Cmd+V all work; the login survives a restart.
