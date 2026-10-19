Status: ready-for-agent

# Spec: rewrite WhatsApp-QT on Tauri 2 and the system webview

Read first: `GLOSSARY.md` and `docs/adr/0001-system-webview-instead-of-qtwebengine.md`. This spec uses the glossary terms. On a conflict, the glossary and the ADR win.

## Problem Statement

The app wraps WhatsApp Web in Qt and QtWebEngine. It is heavy to build and install, the bundle is large, it uses a lot of memory, and it does not feel native on macOS, which is the first platform. Some parts that users expect do not work:

- Message notifications are never delivered. The page's alerts are not captured, so the notification setting and Snooze have no effect.
- The account list is not restored after a restart. Account data is keyed by tab position, so an account can lose its login or get another account's data when tabs move.
- A `wa.me` or `whatsapp://` deep link does nothing on macOS. A link clicked while the app runs starts a second copy.

## Solution

Rebuild the app on Tauri 2. Each account is a child webview on the system webview (WKWebView on macOS), inside one window, next to a static HTML sidebar. Each account has a UUID that names its own data store, so accounts stay isolated and come back after a restart. Message notifications, the unread count, the tray, Snooze, theme, zoom, shortcuts, deep links, autostart and window state all work at least as well as in the Qt app. Users scan the QR code once after the switch. `settings.json` keeps its path, and the old settings still load. Work happens on the `feat/tauri-rewrite` branch. Qt stays on `main` until parity, then one change removes it.

## User Stories

### Accounts

1. As a user, I want to add an account, so that I can use a second WhatsApp identity in the same app.
2. As a user, I want each new account to show the WhatsApp QR login page, so that I can link it to a phone.
3. As a user, I want each account to keep its own login and data, so that one account never shows another account's chats.
4. As a user, I want my accounts and their logins to come back after I quit and start the app again, so that I do not scan the QR code each time.
5. As a user, I want the app to open on the account that was active when I quit, so that I continue where I stopped.
6. As a user, I want to remove an account, so that I can stop using it in this app.
7. As a user, I want removing an account to delete its login and data for good, so that nothing of it stays on the disk.
8. As a user, I want a new account to never bring back a removed account's login, so that removal is final.
9. As a user, I want each new account to get a default account name, so that I can tell accounts apart in the sidebar, tray and notifications.
10. As a user, I want to switch the active account from the sidebar, so that I can move between identities in one click.
11. As a user, I want switching accounts to keep each account's state (open chat, draft, scroll), so that switching does not lose my work.
12. As a user, I want switching in the middle of typing to be safe, so that keys never go to the wrong account.
13. As a user, I want exactly one active account while any accounts exist, so that the app always shows a chat page.
14. As a user who removes the active account, I want another account to become active, so that the window is never empty while accounts exist.
15. As a user who removes the last account, I want the app to create a new default account, so that the app always has an account to show.

### Typing and editing

16. As a user, I want fast typing, arrow keys, backspace and accents (Option+e, then e) to work in a chat box, so that I can write normally.
17. As a user, I want other input languages and input methods to work, so that I can write in my language.
18. As a user, I want the emoji picker (Ctrl+Cmd+Space) to open, so that I can add emoji.
19. As a user, I want Cmd+C, Cmd+V, Cmd+X, Cmd+A and Cmd+Z to work in a chat box, so that I can edit text.
20. As a user, I want the system spellcheck in the chat box, so that I see typing errors.

### Loading WhatsApp

21. As a user, I want WhatsApp Web to load and not show "WhatsApp works with Safari 15+", so that the app works.
22. As a user, I want links to other sites in a chat to open in my default browser, so that the account webview stays on WhatsApp.

### Message notifications

23. As a user, I want a message notification when any account receives a new message, so that I see new messages when the app is in the background.
24. As a user with more than one account, I want the notification to name the account, so that I know which identity received the message.
25. As a user, I want clicking a message notification to bring the app to the front and make the notifying account the active account, so that I can answer at once.
26. As a user, I want to turn message notifications off in settings, so that the app stays quiet.
27. As a user, I want to choose whether a notification shows the sender, so that I can keep message details private on a shared screen.
28. As a user, I want to turn the notification sound on or off, so that alerts can be silent.
29. As a user, I want to choose a custom notification sound file, so that I can tell WhatsApp alerts apart from other apps.
30. As a user, I want an empty sound path to use the platform's default sound, so that sound works without set-up.
31. As a user, I want to Snooze message notifications for 1 hour or 8 hours from the tray, so that I can focus for a set time.
32. As a user, I want Snooze to end by itself when its time runs out, so that I do not miss messages after I forget to end it.
33. As a user, I want to end Snooze early from the tray, so that I get alerts again when I am ready.
34. As a user, I want Snooze to stay on after a restart until its time runs out, so that a restart does not end my quiet time.
35. As a user, I want Snooze to apply to all accounts at once, so that one action silences everything.
36. As a user, I want a toggle shortcut for message notifications, so that I can turn them on and off without the menu.
37. As a user, I want very long message text to be cut to a safe length, so that a bad message cannot flood my screen.

### Unread count

38. As a user, I want each account to show its own unread count in the sidebar, so that I see which identity has unread chats.
39. As a user, I want the total unread count over all accounts in the tray and on the Dock badge, so that I see unread chats when the window is hidden.
40. As a user, I want the Dock badge to clear when the total is zero, so that the badge is not stale.
41. As a user, I want Snooze to never hide or change the unread count, so that I still see that chats wait for me.

### Tray and window

42. As a user, I want a tray icon with Show/Hide, Snooze 1 hour, Snooze 8 hours, end Snooze, and Quit, so that I control the app without its window.
43. As a user, I want clicking the tray icon to show or hide the window, so that I can get to WhatsApp quickly.
44. As a user, I want closing the window to hide it in the tray when close-to-tray is on, so that I keep getting notifications.
45. As a user, I want closing the window to quit the app when close-to-tray is off, so that close means quit.
46. As a user, I want the app to start hidden in the tray when start-minimized is on, so that it does not open a window at login.
47. As a user, I want the window size and position to come back after a restart, so that I do not arrange the window each time.
48. As a user, I want the account webview to fill the area next to the sidebar when I resize the window, so that no space is wasted.

### Theme, zoom, shortcuts

49. As a user, I want Light, Dark and System themes, so that WhatsApp matches my desktop.
50. As a user, I want the System theme to follow the operating system, so that I do not change it by hand.
51. As a user, I want to zoom in, zoom out and reset zoom (Cmd+=, Cmd+-, Cmd+0), so that text is easy to read.
52. As a user, I want my zoom level to stay after a restart, so that I set it once.
53. As a user, I want Cmd+1 to Cmd+9 to switch to accounts 1 to 9, so that switching is fast.
54. As a user, I want Ctrl+Tab and Ctrl+Shift+Tab to go to the next and previous account, so that I can cycle through accounts.
55. As a user, I want Cmd+T to add an account and Cmd+W to remove the active account after I confirm, so that I can manage accounts from the keyboard.
56. As a user, I want Cmd+, to open settings and Cmd+Q to quit, so that the app acts like other Mac apps.

### Deep links

57. As a user, I want a `wa.me` or `whatsapp://` link to bring the app to the front and open the chat, so that links from other apps work.
58. As a user, I want the deep link to open in the active account without asking, so that it opens in one step.
59. As a user, I want a deep link to never create or remove an account, so that links cannot change my accounts.
60. As a macOS user, I want deep links to work when the app is closed and when it runs, so that every link click works.
61. As a user, I want a deep link clicked while the app runs to go to the running copy, so that a second copy never starts.
62. As a user, I want only `whatsapp://` and `https://wa.me/` links to count as deep links, so that other URLs never raise the window.

### Autostart and settings

63. As a user, I want to turn start-at-login on and off in settings, so that WhatsApp is ready when I log in.
64. As a user, I want an error message when the app cannot change start-at-login, so that I know the setting did not apply.
65. As a user, I want a settings page with the same controls as today, so that I lose no option in the switch.
66. As a user, I want every setting to save as soon as I change it, so that I never lose a change.
67. As a user, I want an error message when settings cannot save, so that I know my change is not stored.
68. As an existing user, I want my current `settings.json` to load, so that theme, notification, Snooze, zoom, window and autostart choices carry over.
69. As an existing user, I want a missing or new setting to get a safe default, so that an old file never breaks the app.
70. As a user, I want clear-cache-on-exit to keep working, so that cached files do not build up.

### Packaging and maintenance

71. As a macOS user, I want an unsigned `.app` and `.dmg`, so that I can install the app.
72. As a Linux user, I want the app to run on X11 or XWayland, so that account webviews draw at the right place.
73. As a maintainer, I want no Qt, QML or C++ in the build, so that the build is fast and has fewer dependencies.
74. As a maintainer, I want the Tauri version pinned, so that a minor Tauri release cannot break the `unstable` child-webview API without notice.
75. As a maintainer, I want the logic for accounts, notifications, Snooze, unread count, deep links and settings tested with plain `cargo test`, so that I can change it safely.

## Implementation Decisions

- **Stack**: Tauri 2 with the `unstable` feature for child webviews (`Window::add_child`). The version is pinned to an exact version. A Tauri upgrade is a separate task.
- **Window layout**: one main window. A static HTML/CSS/JS sidebar child webview on the left, with no npm and no framework. One child webview for each account fills the rest. Switching shows the active account's webview and hides the others. Resize sets the bounds of every child webview.
- **Webview settings**: every account webview loads `https://web.whatsapp.com` with this fixed Safari user agent: `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15`. The user agent is not a setting.
- **Account identity and data**: each account has a UUID, made when the account is added. On macOS the UUID's 16 bytes are the webview's data store identifier, so the minimum is macOS 14. On Linux and Windows the UUID names a data directory under the app data folder. Removing an account removes its data store (`remove_data_store` on macOS; delete the folder on Linux and Windows).
- **Menu**: keep Tauri's default macOS menu. Its Edit menu gives copy, paste and the emoji picker.
- **Core module (the one test seam)**: a pure Rust module with no Tauri types. It owns the app state and has one entry point that takes an event and returns a list of effects. The Tauri layer turns webview, tray, menu, shortcut and OS events into core events, and runs the effects.
  - Events: settings loaded (raw JSON), account added, account removed, account activated, page title changed (account, title), notification requested (account, title, body), notification clicked (account), deep link received (URL), Snooze set (minutes, now), Snooze cleared, time checked (now), setting changed.
  - Effects: create account webview (UUID), delete data store (UUID), show account (UUID), show notification (account name, title, body, sound or no sound), set unread total, focus window, open chat in the active account (URL), save settings (JSON), report error (message).
- **Settings**: the path stays `<config dir>/WhatsAppDesktop/settings.json`. The old fields load with the same names, and `#[serde(default)]` gives safe defaults (zoom 1.0, spellcheck on, autostart off). New fields: an ordered list of accounts (UUID, account name) and the active account's UUID. The old `tabs` and `current_tab` fields are ignored. On first start after the switch, the app makes one new default account.
- **Message notifications**: an injected script replaces `window.Notification` in each account webview. It sends title and body to Rust through one IPC command. Only the `https://web.whatsapp.com` origin can call it, through a Tauri capability with a remote URL. The core cuts title and body to fixed maximum lengths and applies the notification setting, show-sender and Snooze. The notification plugin shows the alert. A click returns the account's UUID.
- **Unread count**: read from the page title of each account (WhatsApp Web puts `(N)` in the title). The core keeps a count for each account, and the total goes to the tray tooltip and `Window::set_badge_count`. Snooze never changes it.
- **Snooze**: stored as an end time in `settings.json`, so it survives a restart. It is global. "End Snooze" clears it. The tray has 1-hour and 8-hour items.
- **Theme**: the existing CSS generation moves into the core as a pure function. It is injected into each account webview when the theme changes and when a page loads. System follows the OS appearance.
- **Zoom**: applied to every account webview, saved in settings.
- **Shortcuts**: as in the Qt app, with Cmd on macOS: next/previous account, add account, remove account (after confirmation), settings, quit, notification toggle, account 1 to 9, zoom in/out/reset.
- **Deep links**: the deep-link plugin for `whatsapp://`, including the macOS Apple Event. The single-instance plugin sends links to the running copy. The existing deep-link recognition rules move into the core.
- **Autostart, window state, tray**: the autostart plugin, the window-state plugin and the core tray API. An autostart error becomes a report-error effect.
- **Settings UI**: a static HTML page with the same controls as the current settings dialog. Each control sends one setting-changed event.
- **Order of work**: shell, accounts, message notifications, unread count, tray and window, theme/zoom/shortcuts, deep links and autostart, settings UI, then the cut (bundle, Linux packaging, remove Qt, QML, the C++ shim and unused crates, update docs, merge to `main`). Each step is one commit, and each step ends with a manual check on macOS.

## Testing Decisions

- A good test drives the core module through its one entry point, and checks only the effects it returns and the settings JSON it saves. It never checks internal fields, so the core can change inside without test changes.
- Tested in the core with plain `cargo test`:
  - Settings: an old `settings.json` (with `tabs`, `current_tab` and nested `notification_settings`) loads, and missing fields get the safe defaults.
  - Accounts: add makes a new UUID and a create-webview effect. Remove makes a delete-data-store effect for that UUID only. Removing the active account activates another one. Removing the last account adds a default one. Add after remove never reuses a UUID.
  - Notifications: off, Snooze and show-sender give the right result. Long title and body are cut. A notification click focuses the window and activates the account.
  - Snooze: it ends at its end time, it survives a save-and-load cycle, and it does not change the unread total.
  - Unread count: titles with `(N)`, without a count, and with odd text. The total is the sum over accounts. Removing an account removes its count from the total.
  - Deep links: `whatsapp://` and `https://wa.me/` are accepted. Other URLs and arguments are refused. A deep link never adds or removes an account.
  - Theme CSS: each theme, as in the current test.
- Prior art: the current unit tests on pure functions, `recognises_both_deep_link_forms_and_ignores_other_args` in `main.rs`, and `settings_roundtrip_preserves_new_flags` and `generate_css_covers_each_theme` in `cxxqt_object.rs`. Carry over their cases.
- Not automated: webviews, typing, the emoji picker, tray, menus, window state, autostart and bundling. Each step has a manual macOS check, and the step's commit waits for it. The first check of the accounts step is: account A logged in, account B still shows the QR code, and both survive a restart.

## Out of Scope

- Everything in the TBD list in `CLAUDE.md`: spellcheck toggle, custom download path, account rename, and native Wayland on Linux.
- Call notifications. The `show_call_notifications` setting still loads and saves, but has no effect.
- Code signing and notarization.
- Windows packaging and the Windows `whatsapp://` registry key.
- Bringing back old Qt logins. Users scan the QR code again once.
- Renaming the app.
- A per-account silence (the glossary keeps "mute" for this).

## Further Notes

- The spikes showed that WhatsApp Web refuses the default WKWebView user agent, and that wry without an Edit menu loses the emoji picker. Both are covered above.
- The current tray says "Mute notifications". The new tray says "Snooze", per the glossary.
- If child webviews break typing in a later Tauri release, the fallback in ADR 0001 is one window per account with native macOS window tabs.
- `.gitignore` ignores `*.md`. Add this file and the tickets with `git add -f`.
