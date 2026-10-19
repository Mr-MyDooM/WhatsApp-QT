# 04: The settings window reads one settings view from the core

**What to build:** the core owns the shape the settings window reads, and the TypeScript type for it is generated from Rust. Today `get_settings` is a hand-built JSON that renames fields, and `types.ts` repeats the shape by hand. Nothing checks that the two agree.

**Blocked by:** None (can start immediately).

**Status:** resolved

## Decisions

- `Core::settings_view(now) -> SettingsView`, a serde struct with the field names the frontend uses now (`notifications`, `clear_cache_supported`, `load_error`, `snooze_until` as the active Snooze end, `accounts`). `get_settings` becomes one line. `settings.json` on disk does not change.
- Add `ts-rs`. A cargo test writes the settings view type into `frontend/src/settings/types.ts`. The test fails when the file on disk was different, and writes the new file. Commit the generated file.
- Delete core accessors that only `get_settings` used.
- The `set_*` commands do not change in this ticket.

## Done when

- [ ] Core tests cover the `SettingsView` shape, derived fields included.
- [ ] The ts-rs drift test passes, and `pnpm --dir frontend build` passes with the generated type.
- [ ] Manual check in the bundled app: the settings window shows each current value, the Snooze end and the account list as before.

## Answer

Done on `refactor/architecture`. Commit 306435e. The four checks exit 0 at 831d0e4 (145 tests). Not done: the manual check in the bundled `.app`.
