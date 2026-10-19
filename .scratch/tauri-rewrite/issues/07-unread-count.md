# 07: Unread count

**What to build:** each account shows its unread count in the sidebar. The total over all accounts shows in the tray and on the Dock badge. Snooze never hides or changes it.

**Blocked by:** 02, 04

**Status:** ready-for-agent

- [ ] The unread count of each account is read from its page title (`(N)` prefix).
- [ ] The sidebar shows each account's unread count.
- [ ] The total shows in the tray and on the Dock badge; the badge clears at zero.
- [ ] Removing an account removes its count from the total.
- [ ] Core tests: titles with a count, without a count, with odd text; total; removal.
- [ ] Manual macOS check: the Dock badge matches WhatsApp's unread chats, also during Snooze.
