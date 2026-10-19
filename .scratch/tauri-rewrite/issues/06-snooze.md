# 06: Snooze

**What to build:** from the tray, the user can Snooze message notifications for all accounts for 1 hour or 8 hours, and end Snooze early. Snooze ends by itself and survives a restart.

**Blocked by:** 04, 05

**Status:** ready-for-agent

- [ ] Tray items: Snooze 1 hour, Snooze 8 hours, End Snooze. The tray uses the word "Snooze", not "Mute".
- [ ] Snooze is stored as an end time in `settings.json`; it is global, ends by itself at its end time, and survives a restart.
- [ ] No message notification is shown during Snooze.
- [ ] Core tests: ends at end time, survives save-and-load, End Snooze clears it, does not touch the unread count.
- [ ] Manual macOS check: no alert during Snooze; alerts come back after End Snooze.
