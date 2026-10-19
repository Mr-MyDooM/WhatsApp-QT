# 05: Accounts in the tray and Dock menus

**What to build:** the tray menu and the macOS Dock menu list each account with its account name and unread count. Click an account to show the window and make it the active account. The lists update when an account is added, removed or renamed, and when an unread count changes. Source: the research report, ranked item 10, and the HIG on Dock menus.

**Blocked by:** 02 (Account menu, rename, reload and error page).

**Status:** ready-for-agent

**Note:** Tauri has no Dock menu API. The Dock menu probably needs a native `applicationDockMenu:` call through objc2. If that is not practical, ship the tray part, and write down the reason and the Dock part as a new ticket.

- [ ] The tray menu lists each account by name, with its unread count if it is above 0.
- [ ] Click an account in the tray to show the window and make that account active.
- [ ] On macOS the Dock menu has the same list and the same behaviour.
- [ ] The lists update after add, remove, rename and unread count changes.
- [ ] The existing tray items (Snooze, End Snooze, Quit) still work.
