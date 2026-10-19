# 10: Settings: test notification, sound file picker, Snooze, About

**What to build:** the Notifications pane gets a "Send Test Notification" button, a Choose button for the sound file, and Snooze controls. The About pane shows real details instead of a blank page.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [x] Notifications pane: a "Send Test Notification" button shows one message notification through the normal path (same permission, sound and show-sender rules). If notifications are off, denied or snoozed, the pane says why no alert came.
- [x] Sound file: a "Choose..." button opens the native file picker (dialog plugin), filtered to sound files (aiff, wav, mp3, m4a, caf). A "Default" button clears the path. The text field stays.
- [x] Snooze in Notifications pane: Snooze 1 hour, Snooze 8 hours, End Snooze, with the end time shown. Same core events as the tray items. Tray and settings stay in sync.
- [x] About pane: find and fix why it is blank (user report, release build). Show icon, name, version, one-line description, links (repository, licence), "Not affiliated" line, and the settings file path.
- [x] New commands get settings-capability permissions only; `capabilities/remote.json` unchanged.
- [x] Core tests for any new core events.
- [ ] Manual macOS check: test alert arrives; picker sets the path; Snooze from settings and tray agree; About shows details.
