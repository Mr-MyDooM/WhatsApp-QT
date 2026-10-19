# 04: Mute one account

**What to build:** the user can mute one account from its menu. A muted account raises no message notifications. Its unread count still shows in the sidebar, the tray and the Dock badge. The sidebar shows a mute mark on a muted account. The mute stays after a restart until the user unmutes the account. Snooze stays global and is separate from mute. Add "Mute" to GLOSSARY.md when this ticket starts: the Snooze entry already reserves the word for this. Source: the research report, ranked item 8.

**Blocked by:** 01 (Sidebar redesign), 02 (Account menu, rename, reload and error page).

**Status:** ready-for-agent

- [ ] The account menu has Mute or Unmute, depending on the account's state.
- [ ] A muted account raises no message notifications. The other accounts still do.
- [ ] A muted account's unread count still shows, and still counts in the tray and Dock total.
- [ ] The sidebar shows a mute mark on a muted account, and its accessible name says "muted".
- [ ] The mute is saved and stays after a restart.
- [ ] Mute and Snooze are independent. Ending Snooze does not unmute an account, and unmuting does not end Snooze.
- [ ] GLOSSARY.md has an entry for Mute.
- [ ] The mute rules are tested through the core.
