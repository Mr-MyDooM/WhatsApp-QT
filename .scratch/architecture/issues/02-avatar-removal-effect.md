# 02: Account removal clears the avatar through the core

**What to build:** the core lists the avatar step in the account lifecycle. Today the `DeleteAccountData` arm in `run_effects` also clears the avatar and calls `emit_avatars`, a rule the core does not know.

**Blocked by:** None (can start immediately).

**Status:** resolved

## Decisions

- `Core::remove` returns a new `RemoveAvatar { id }` Effect.
- Its `run_effects` arm calls `Avatars::remove` and `emit_avatars`. The `DeleteAccountData` arm only deletes data.
- The known-account check in `profile_picture` stays as it is.

## Done when

- [ ] Core test: removing an account returns `RemoveAvatar` for its id.
- [ ] Manual check in the bundled app: remove an account; its avatar is gone from the sidebar and from the app data `avatars` folder.

## Answer

Done on `refactor/architecture`. Commit 60cee6b. The four checks exit 0 at 831d0e4 (145 tests). Not done: the manual check in the bundled `.app`.
