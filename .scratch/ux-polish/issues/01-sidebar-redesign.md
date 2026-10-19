# 01: Sidebar redesign

**What to build:** the sidebar shows each account as a round avatar with the initials of its account name. Each account keeps the same avatar colour, taken from its UUID. A marker on the leading edge shows the active account, so the fill colour is not the only sign. The unread count shows as a pill on the corner of the avatar. The sidebar follows the light and dark theme. Each account is a button that VoiceOver and the keyboard can reach. The tooltip shows the account name and its Cmd+N shortcut. Source: the research report, Lens 2, "Sidebar".

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Each account shows an avatar with initials from its account name. One-word names give one letter, longer names give two.
- [ ] The avatar colour comes from the account UUID. It does not change after a restart or a rename.
- [ ] The active account has a marker on its leading edge.
- [ ] The unread count shows as a pill on the avatar, with "99+" above 99. There is no pill at 0.
- [ ] The sidebar colours follow the Light, Dark and System themes.
- [ ] Each account is a `button`. Its accessible name has the account name and the unread count (for example "Work, 3 unread"). The active account has `aria-current`.
- [ ] The focus ring is visible. The up and down arrow keys move through the accounts.
- [ ] The tooltip shows the name and the shortcut (for example "Work, Cmd+1").
- [ ] The add-account button has an accessible name and sits right after the last account.
- [ ] Hover and switch animations respect `prefers-reduced-motion`.
- [ ] Text contrast meets 4.5:1 in both themes.
