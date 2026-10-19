# 03: Settings page, theme and zoom

**What to build:** Cmd+, opens a static settings page. The user can pick Light, Dark or System theme and change zoom; both apply to every account webview and survive a restart. Every setting saves when it changes, and a failed save shows an error.

**Blocked by:** 01

**Status:** resolved

- [ ] Static HTML settings page, opened with Cmd+,. Each control sends one setting-changed event to the core.
- [ ] Theme: Light, Dark, System. The theme CSS is injected into each account webview on change and on page load. System follows the OS appearance.
- [ ] Zoom: Cmd+=, Cmd+-, Cmd+0 and a settings control; applied to every account webview; saved.
- [ ] Every change saves at once; a save failure shows a friendly error and logs the detail.
- [ ] Theme CSS generation moves into the core as a pure function; the current theme CSS test cases carry over.
- [ ] Manual macOS check: each theme looks right; zoom level survives a restart.
