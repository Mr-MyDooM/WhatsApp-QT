# 08: Notification permission in Settings

**What to build:** the settings window shows whether macOS allows the app to show message notifications: "Allowed", "Not allowed" or "Not asked yet". When the status is "Not asked yet", an "Allow notifications" button asks macOS for permission, and the status updates with the answer. macOS asks only one time. When the status is "Not allowed", the button text is "Open System Settings" and the button opens the Notifications pane of System Settings, so the user can change the answer there. The status refreshes each time the settings window gets focus. Under plain `cargo run` (no app bundle), the status is "Not available outside the app bundle" and there is no button.

**Blocked by:** None (can start immediately). Decision: the user asked for this on 2026-10-19.

**Status:** ready-for-agent

- [ ] Settings shows the current permission status in one of the four states above.
- [ ] "Allow notifications" shows the macOS prompt when the status is "Not asked yet". The status then shows the user's answer.
- [ ] "Open System Settings" opens the Notifications pane when the status is "Not allowed".
- [ ] The status is correct after the user changes it in System Settings and returns to the app.
- [ ] Under `cargo run` the app does not crash, and the settings show "Not available outside the app bundle".
- [ ] The existing request at the first message notification still works.
- [ ] Only the settings webview can call the new commands. The `web.whatsapp.com` origin still gets exactly one IPC command.
