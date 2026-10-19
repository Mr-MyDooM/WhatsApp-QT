# 05: Message notifications

**What to build:** when any account receives a new message, the app shows a message notification that names the account. The notification setting, show-sender and sound settings apply. Clicking the notification brings the app to the front and makes that account the active account.

**Blocked by:** 02, 03

**Status:** ready-for-agent

- [ ] An injected script replaces `window.Notification` in each account webview and sends title and body through one IPC command. Only the `https://web.whatsapp.com` origin can call it (capability with a remote URL).
- [ ] The core cuts title and body to fixed maximum lengths, and applies notifications on/off and show-sender.
- [ ] Sound on/off, and a custom sound file; an empty sound path uses the platform default sound.
- [ ] Clicking a notification focuses the window and activates the notifying account.
- [ ] Settings page controls: notifications on/off, show sender, sound on/off, sound file. A shortcut toggles notifications on and off.
- [ ] Core tests: off, show-sender, length caps, click activates the account.
- [ ] Manual macOS check: a message sent from another phone shows an alert with the account name; clicking it opens that account; no alert when notifications are off.
