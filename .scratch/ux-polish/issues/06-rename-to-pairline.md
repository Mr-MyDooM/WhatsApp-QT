# 06: Rename the app to Pairline

**What to build:** the app is called Pairline everywhere a user sees it: window title, menu bar, tray tooltip, About pane, notifications, bundle name, `.app` and `.dmg` names. The bundle ID and app ID change to match. The icon does not look like the WhatsApp logo and does not use its green. The description says "an unofficial desktop client for WhatsApp Web" and "Not affiliated with or endorsed by WhatsApp or Meta". Reason: Meta's brand rules forbid "WhatsApp" in another product's name, Flathub refuses such names, and "QT" is wrong after leaving Qt. Source: the research report, Lens 3, "Name and trademark".

**Blocked by:** None (can start immediately). Decision: the user picked the name "Pairline".

**Status:** ready-for-agent

**Note:** this must land before the cut to main (`.scratch/tauri-rewrite/issues/10`). Users scan the QR code again at the cut, so a bundle ID change after it would make them scan a second time. ADR 0001 says the path and format of `settings.json` do not change: keep the `WhatsAppDesktop` settings path.

- [ ] The user sees "Pairline" in every place where the app shows its own name.
- [x] The bundle ID and app ID are changed. All places that use the ID agree (config, deep link registration, autostart, packaging).
- [x] The `settings.json` path is not changed. Existing settings still load.
- [x] The new icon has no WhatsApp phone-in-bubble shape and no WhatsApp green.
- [x] The "not affiliated" line is in the bundle description and the About pane.
- [ ] Deep links and autostart still work in the bundled .app after the change.
- [ ] Ask the user before renaming the Git repository or the crate. Those are outside the app's visible name.
