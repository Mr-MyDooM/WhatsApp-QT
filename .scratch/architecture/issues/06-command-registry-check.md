# 06: Check the three command registries

**What to build:** a cargo test that fails when a command is missing from one of its three registrations. Today `build.rs`, `invoke_handler` in `main.rs` and `capabilities/*.json` each list the commands by hand, and only the CLAUDE.md checklist keeps them equal. A miss shows only at runtime.

**Blocked by:** 05 (it deletes 11 commands; write the check after).

**Status:** ready-for-agent

## Decisions

- A test in `main.rs` reads `build.rs`, `main.rs` and `capabilities/*.json` as text and compares the name sets: every command in `build.rs` is in `invoke_handler` and has an `allow-<kebab-name>` in at least one capability, and there are no extras.
- No new crate, no shared macro list.

## Done when

- [ ] The test passes on the current tree.
- [ ] Taking one name out of any of the three makes the test fail with a message that names the command and the file.
