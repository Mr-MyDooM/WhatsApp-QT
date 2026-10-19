# 03: Sidebar layout comes from the core

**What to build:** the core decides where the sidebar ends and the account pages start, in every sidebar mode. Today the `SIDEBAR_SHOWN` static, `sidebar_widths` and the sidebar width constants live in `main.rs`, where no test reaches them.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

## Decisions

- The reveal state, `sidebar_widths` and the sidebar width constants move into the core.
- `reveal_sidebar` sends a `SidebarRevealed { shown }` Event.
- Width changes (wide, auto-hide, reveal) return a `Layout` Effect with the page start and the sidebar width. `run_effects` only sets the bounds.
- No trait seam or fake for Tauri.

## Done when

- [ ] Core tests: `Layout` widths for each mode (narrow, wide, auto-hide shown, auto-hide hidden).
- [ ] `main.rs` has no `SIDEBAR_SHOWN` static and no width math.
- [ ] Manual check in the bundled app: toggle wide, turn on auto-hide, move the mouse over the edge; the sidebar and pages lay out as before.
