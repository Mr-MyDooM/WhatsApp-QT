# 03: Settings in panes

**What to build:** the settings window has four panes, following the macOS Human Interface Guidelines. General: theme, zoom, start at login, start minimized, close to tray, clear cache on exit. Notifications: on/off, show sender, sound, sound file, and Snooze status. Accounts: the list of accounts, with rename and remove. About: version, links, and the line "Not affiliated with or endorsed by WhatsApp or Meta". The window title is the name of the pane. The window opens on the last pane used. An error shows next to the control that failed. Source: the research report, Lens 2, "Settings".

**Blocked by:** 02 (Account menu, rename, reload and error page).

**Status:** resolved

- [x] The settings window has the panes General, Notifications, Accounts and About. Each existing control is in one pane and still works.
- [x] The window title is the pane name.
- [x] The window opens on the last pane used.
- [x] Notifications shows whether Snooze is on, and until what time.
- [x] Accounts lists each account by name. Rename and remove use the same behaviour as the account menu.
- [x] About shows the app version and the "not affiliated" line.
- [x] Zoom has preset buttons (90, 100, 110 and 125 percent) as well as the slider.
- [x] A failed save shows its error next to the control that caused it.
- [x] Cmd+, still opens the settings window.
