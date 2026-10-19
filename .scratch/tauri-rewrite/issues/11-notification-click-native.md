# 11: Notification click opens its account (native macOS notifications)

**What to build:** clicking a message notification brings the app to the front and makes the notifying account the active account, as GLOSSARY.md "Message notification" says. The review fix removed click handling to stop a thread leak: mac-notification-sys has one global click delegate that each send replaces, and a waiting send blocks its thread forever. Replace it on macOS with UNUserNotificationCenter through objc2: one delegate for the whole process, set once, that reads the account id from each notification's `userInfo` and dispatches the click to the core. No thread per notification.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Note:** UNUserNotificationCenter needs an app bundle. It may not work under plain `cargo run`. Find out whether an unsigned or ad-hoc signed `.app` from the macOS packaging script is enough. If it needs a paid Developer ID signature, stop and report: code signing is out of scope.

- [ ] Clicking a message notification shows the window and makes the notifying account active.
- [ ] Each notification carries its account id. Overlapping notifications from different accounts each open the right account.
- [ ] No thread is created or blocked per notification.
- [ ] The app asks for notification permission on first use. A refusal is reported to the user, not swallowed.
- [ ] Snooze, show-sender, sound and the length caps still apply.
- [ ] The click routing (account id in, focus and switch effects out) is tested through the core.
- [ ] Works in the bundled `.app`. Behaviour under `cargo run` is documented.
- [ ] Linux and Windows notification code is left as it is (paused).
