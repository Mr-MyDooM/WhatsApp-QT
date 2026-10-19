# 02: Accounts

**What to build:** the user can add, remove and switch accounts. Each account is isolated, has its own data store named by its UUID, has a default account name, and comes back after a restart, with the active account restored.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Adding an account makes a new UUID, a default account name, and a new account webview that shows the QR login.
- [ ] The sidebar lists accounts by account name; clicking one makes it the active account. Switching shows only the active account's webview and keeps each account's state.
- [ ] Removing an account asks for confirmation, then deletes its data store for good. A later new account never reuses its UUID or its login.
- [ ] Removing the active account activates another account. Removing the last account creates a new default account.
- [ ] The ordered account list and the active account's UUID are saved in `settings.json` and restored on start. Old `tabs` and `current_tab` are ignored; first start after the switch makes one default account.
- [ ] Shortcuts: Cmd+T add, Cmd+W remove active (after confirmation), Cmd+1 to Cmd+9 switch, Ctrl+Tab and Ctrl+Shift+Tab next and previous.
- [ ] Core tests: add, remove, remove active, remove last, no UUID reuse, save-and-load round trip.
- [ ] Manual macOS check: account A logged in while account B still shows the QR code; both survive a restart; switching in the middle of typing never sends keys to the wrong account.
