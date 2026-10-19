# 10: Cut to main

**What to build:** the app ships as a macOS `.app` and `.dmg`, Linux packaging builds the Tauri app, all Qt code is gone, the docs describe the new stack, and the branch merges to `main`.

**Blocked by:** 01, 02, 03, 04, 05, 06, 07, 08, 09, 11, and `.scratch/ux-polish/issues/06` (Rename the app to Pairline)

**Status:** resolved

- [x] Unsigned `.app` and `.dmg` built with `tauri bundle`; the macOS packaging script is replaced or removed.
- [ ] Linux packaging (deb, rpm, AppImage, Flatpak) builds the Tauri app and runs on X11/XWayland. Deferred by the user (2026-10-19) until Linux work starts.
- [x] Removed: QML, the C++ WebEngine shim, cxx-qt, Qt dependencies, and the unused crates from the cleanup backlog.
- [x] CLAUDE.md and README describe the Tauri stack; the cleanup backlog entry is removed.
- [x] `cargo build` and `cargo test` exit 0; full manual macOS check of every ticket above passes on the bundled app.
- [x] The branch merges to `main` after the user approves.
