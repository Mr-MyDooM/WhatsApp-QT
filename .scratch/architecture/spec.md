# Spec: deepen the shell modules

**Status:** resolved

Source: architecture review of 2026-10-19 (main @ 7c46e60) and the grilling session after it. Tickets: `issues/01` to `issues/08`.

## Problem Statement

The developer of Pairline adds features often, and each one costs more than it should. A new boolean setting touches about 11 places across Rust, the build script, capabilities and TypeScript, and nothing checks that these places agree. A setting that changes the account page reaches it by four different paths, with JS names and limits spelled by hand in both Rust and the shims. Many decisions (sidebar layout, startup order, the sidebar data, the avatar step of account removal) live in the Tauri shell, where no test can reach them. Bugs in these places show only when the developer runs the bundled app by hand.

## Solution

Move each decision behind one tested interface and give each fact one source:

- One account page module turns one page config into everything the account page receives, at load and on each change.
- The core owns the settings view the settings window reads and one typed change path for every setting.
- The TypeScript types for the settings view, the setting change and the sidebar view are generated from Rust, and a test fails on drift.
- A test checks that every command is registered in all three places.
- The core returns the avatar step of account removal, the sidebar layout, the startup order and the sidebar view as Effects. The shell only carries them out.

The user of the app sees no change. The developer gets fewer touch points per feature and tests that catch the mistakes that today show only at runtime.

## User Stories

1. As the developer, I want one page config to carry theme CSS, zoom, spellcheck and chat list width, so that I add a new account page setting in one place.
2. As the developer, I want the account page to get the same config at load and on each change, so that a reloaded page never misses a setting.
3. As the developer, I want the JS global names written in only two places, so that a rename cannot break one side quietly.
4. As the developer, I want the chat list width limits to have one source, so that the shim and the core never disagree.
5. As the developer, I want the WhatsApp DOM selectors out of the core, so that the core holds only app state and rules.
6. As the developer, I want a test to prove that each page setting change returns the page config for all accounts, so that I know every account page gets it.
7. As the developer, I want a test to prove that a page load returns the page config for that one account, so that new and reloaded pages are covered.
8. As the developer, I want the init script and the live update to be testable without Tauri, so that I can check the JS text in `cargo test`.
9. As the developer, I want the main window color scheme separate from the account page config, so that each Effect has one job.
10. As the developer, I want the settings window to read one settings view built by the core, so that the shape it gets is tested.
11. As the developer, I want the settings view to keep the field names the frontend uses now, so that the frontend does not need renames.
12. As the developer, I want one typed command to change any setting, so that I do not add a command, a build entry and a permission per setting.
13. As the developer, I want bad setting input to fail when it is read, so that the core never sees a value of the wrong type.
14. As the developer, I want each notification setting to be its own change, so that a save error shows under the right control.
15. As the developer, I want the TypeScript settings types generated from Rust, so that the frontend build fails when they differ.
16. As the developer, I want the drift test to write the new file when it fails, so that one more `cargo test` run fixes it.
17. As the developer, I want save errors keyed by the setting change tag, so that I do not keep a separate list of field names by hand.
18. As the developer, I want the core accessors that only fed the settings reply to go away, so that the core interface is smaller.
19. As a user of the app, I want the account page to keep calling only the narrow chat list width command, so that the untrusted page gets no new power.
20. As a user of the app, I want only the settings window to change settings, so that the account page cannot change them.
21. As the developer, I want `cargo test` to fail when a command is missing from the build script, the handler or the capabilities, so that I find it before I run the app.
22. As the developer, I want that failure to name the command and the place it is missing, so that the fix is obvious.
23. As the developer, I want account removal to return the avatar step from the core, so that "remove clears the avatar" is a tested rule.
24. As a user of the app, I want a removed account's avatar gone from the sidebar and from disk, so that no trace of the account stays.
25. As the developer, I want the auto-hide sidebar state in the core, so that it is not a hidden static in the shell.
26. As the developer, I want the core to return the page start and sidebar width for each sidebar mode, so that layout math has tests.
27. As a user of the app, I want the sidebar and account pages to lay out as before in narrow, wide and auto-hide modes, so that the refactor changes nothing I see.
28. As the developer, I want the startup order to be the Effects of loading settings, so that the order is in one place and tested.
29. As the developer, I want the core to decide whether to register the login item again, with the debug-build flag as input, so that a dev run never points the login item at a dev binary.
30. As the developer, I want the core to build the sidebar view (name, initials, hue, unread count, active), so that the sidebar data shape is tested.
31. As the developer, I want the sidebar view TypeScript type generated from Rust, so that the sidebar and the core never disagree.
32. As the developer, I want the shell to stay a direct adapter with no fake, so that I do not keep a seam that only tests use.
33. As a user of the app, I want my logins kept through all of this, so that I never scan the QR code again because of a refactor.
34. As a user of the app, I want my saved settings file read as before, so that no setting resets.
35. As the developer, I want each ticket in its own commit, so that I can revert one without the others.

## Implementation Decisions

- **Account page module** (new, pure, no Tauri types). Interface: page config in; init script and live update JS out. It owns the theme CSS and the WhatsApp DOM selectors, which move out of the core.
- **Page config** holds theme CSS, zoom, spellcheck, chat list width, and the chat list min and max width. The core keeps the only definition of the width limits.
- **One Effect, `ApplyPageConfig`**, with a target of all accounts or one account id. It replaces the page part of `ApplyTheme` and all of `ApplyZoom`. The shell does one native zoom call and one script run per account page.
- **New Effect, `SetWindowScheme`**, for the main window color scheme.
- **Page load:** `PageLoaded` returns `ApplyPageConfig` for that account. The page load handler stops reading the core.
- **JS contract:** one config global at init and one apply entry for live updates. One small bootstrap shim holds the entry. The theme, spellcheck and chat list shims register handlers with it. A new theme shim replaces the Rust-built CSS injection. The avatar and notification shims do not change.
- **Settings view:** `Core::settings_view(now)` returns a serde struct with the current frontend field names, derived fields included (clear cache support, load error, active Snooze end, accounts). The stored settings file does not change.
- **Setting change:** one command takes a serde-tagged `SettingChange` enum with one variant per field. Notification fields are split, and the download path is included. It maps to one core Event and replaces the 11 per-setting commands with their build entries, capabilities and generated permissions. Only the settings window capability grants it.
- **Chat list width** stays its own command, because the untrusted account page calls it. It still checks that the caller is an account page and updates every account page.
- **Type generation:** add `ts-rs`. It exports `SettingsView`, `SettingChange` and `SidebarView`. The generated TypeScript files are committed.
- **Avatar:** `Core::remove` also returns `RemoveAvatar { id }`. Data deletion no longer touches avatars. The known-account check for incoming profile pictures stays as it is.
- **Sidebar layout:** the reveal state and the width rules move into the core. A `SidebarRevealed` event and the wide and auto-hide changes return a `Layout` Effect with the page start and the sidebar width.
- **Startup:** `SettingsLoaded` returns the appearance, sidebar, login item and start window Effects in order. The debug-build flag is event data.
- **Sidebar view:** the core builds it. The shell sends it unchanged.
- **Unchanged:** quit and the cache clear stay in the shell, because they hold the exit on a thread with a timeout. The `AccountLoadBegan` re-dispatch after webview creation stays, because the load starts only if creation worked.
- **No Tauri trait or fake.** The shell stays a direct adapter.
- **Dropped:** a single notification gate. `block_for` is already the one gate.

## Testing Decisions

- A good test drives a module through its interface and checks what comes out: Effects, a view, or script text. It never checks private state or call order inside a module.
- **Main seam: `Core::handle`** (Event in, Effects out). It covers the page config Effects for each page setting and for page load, each `SettingChange` variant, `settings_view`, `RemoveAvatar` on removal, `Layout` for each sidebar mode (narrow, wide, auto-hide shown and hidden), startup Effect order, and `SidebarView` contents. Prior art: the existing core tests, for example `appearance_effects_reflect_loaded_settings`, the zoom clamp tests and the load retry tests.
- **Account page module:** the init script and the live update hold the config as JSON, and the theme CSS covers each theme. Prior art: `theme_css_covers_each_theme`, which moves here.
- **Drift test:** generates the TypeScript files, fails if the file on disk differed, and writes the new one.
- **Registry check:** reads the build script, the handler list and the capability files as text and compares the command name sets. A missing or extra name fails with the command and the place.
- **Not unit-tested:** `run_effects` and the Tauri glue. They stay thin, one Tauri call per Effect, and each ticket has a manual check in the bundled `.app`.

## Out of Scope

- One notification gate (dropped).
- Moving quit and the cache clear into the core.
- A Tauri trait or a fake shell for tests.
- Any change to the stored settings format or the data store location.
- The avatar and notification shims.
- Linux and Windows checks (paused).
- New user-visible features.

## Further Notes

- Tickets: 01 account page config, 02 avatar removal Effect, 03 sidebar layout, 04 settings view, 05 setting change command, 06 command registry check, 07 startup order, 08 sidebar view. Blocking: 05 after 04, 06 after 05, 07 after 01, 08 after 04. Tickets 01 to 04 can start at any time. The edges show only real dependencies, so parallel work can conflict in the shell.
- Each ticket is one commit. It is done only when each of these exits 0: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `pnpm --dir frontend build`. Then the manual check in the bundled `.app` on macOS.
- Never move or reset an account's data store, because that logs every account out.
- `ts-rs` is the only new dependency.
- Glossary: **Account page** was added to GLOSSARY.md during the grilling.
