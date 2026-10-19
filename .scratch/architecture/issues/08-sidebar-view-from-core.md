# 08: The sidebar reads one sidebar view from the core

**What to build:** the core builds the account list the sidebar shows, and the TypeScript type for it is generated from Rust. Today `main.rs` builds the sidebar JSON by hand, and `Sidebar.tsx` repeats the shape.

**Blocked by:** 04 (it adds ts-rs and the drift test).

**Status:** ready-for-agent

## Decisions

- The core builds a serde `SidebarView` (id, name, initials, hue, unread count, active). The shell sends it unchanged.
- ts-rs exports it for `Sidebar.tsx`, with the same drift test as 04.
- The `AccountLoadBegan` re-dispatch from `CreateAccountWebview` stays: the load starts only if the webview was created.

## Done when

- [ ] Core tests: `SidebarView` contents for active, unread and renamed accounts.
- [ ] `main.rs` has no hand-built sidebar JSON.
- [ ] The drift test and `pnpm --dir frontend build` pass.
- [ ] Manual check in the bundled app: add, rename, switch and remove an account; unread counts show; the sidebar looks as before.
