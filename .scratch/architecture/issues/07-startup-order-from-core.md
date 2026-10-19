# 07: Startup order comes from the core

**What to build:** the core decides what happens at start, and in which order. Today `setup` in `main.rs` makes separate calls for appearance, sidebar, the login item and the start window.

**Blocked by:** 01 (it changes the appearance Effects to `ApplyPageConfig` and `SetWindowScheme`).

**Status:** resolved

## Decisions

- `Event::SettingsLoaded` returns, in order, the appearance, sidebar, login item re-register and start window Effects. `setup` stops making separate calls.
- The debug-build flag goes in as event data, so the core decides whether to register the login item again. A dev run never points the login item at a dev binary.
- Quit and the cache clear stay in `begin_quit`: it holds the exit on a thread with a timeout. That is shell work.

## Done when

- [ ] Core tests: startup Effect order; no login item Effect when the debug flag is set.
- [ ] Manual check in the bundled app: start with the window hidden and shown, with auto-start on; the app starts as before.

## Answer

Done on `refactor/architecture`. Commit 3e2b5cd. Review fix 4ddeb0d changes the decision: `SettingsLoaded` returns the appearance, sidebar and login item Effects. A new `Event::TraySetUp`, sent after the tray and Dock setup, returns the start window Effects. This keeps the old start order. A failed login item re-register only logs (`Effect::ReregisterAutostart`). The four checks exit 0 at 831d0e4 (145 tests). Not done: the manual check in the bundled `.app`.
