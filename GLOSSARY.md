---
documentLanguage: en
---

# Glossary

One entry per term: definition, boundaries, one resolved ambiguity. Agents write here the moment a term is settled. Vocabulary here is law for all specs, tickets, and code naming. Ship-specific terms only: a concept any sea chart carries does not get an entry.

## Account
- Definition: one logged-in WhatsApp identity inside the app, with its own login and data, isolated from every other account.
- Boundary: an account is the identity; its tab is only where it is shown. Its identity does not depend on the tab's position, and it persists across restarts until the user removes it.
- Avoid: "user", "session" (WhatsApp Web has its own sessions), "profile", "tab" when the identity is meant.
- Resolved ambiguity: removing an account deletes its login and data for good. Adding a new account never brings back a removed one.

## Account page
- Definition: the WhatsApp Web page loaded in one account's webview.
- Boundary: untrusted. The app changes it only through the injected shims and one page config (theme CSS, zoom, spellcheck, chat list width). The app chrome (sidebar, settings, title bar) is not an account page.
- Avoid: "remote page", "tab", "webview" when the page content is meant.
- Resolved ambiguity: every setting that changes the account page reaches it as the full page config, at load and on each change.

## Account name
- Definition: the label the user sees for an account, such as "Work". A new account gets a default name; the user can rename it.
- Boundary: a label, not an identity. Renaming never changes which account it is.
- Avoid: "tab title".
- Resolved ambiguity: the name is the user's way to tell accounts apart in every surface (sidebar, tray, notifications).

## Active account
- Definition: the account whose tab is currently shown.
- Boundary: exactly one account is active while any exist.
- Avoid: "current tab", "selected account".
- Resolved ambiguity: the active account receives deep links.

## Message notification
- Definition: an operating-system alert the app raises when any account receives a new WhatsApp message.
- Boundary: owned by the app and governed by the notification setting and Snooze. A message notification is not the same as WhatsApp Web's own in-page alert.
- Avoid: "alert", "toast", "popup".
- Resolved ambiguity: message notifications are part of the app's job. If the app does not deliver them, the notification setting and Snooze have no meaning. Clicking one brings the app to the front and makes the notifying account the active account.

## Unread count
- Definition: the number of unread WhatsApp chats an account reports.
- Boundary: each account has its own unread count, shown with that account. The app shows the total over all accounts in the tray and the Dock badge.
- Avoid: "badge" when the number is meant (the badge is only one place it is shown), "notifications".
- Resolved ambiguity: Snooze does not hide or change the unread count.

## Snooze
- Definition: a time-bounded silence of message notifications for all accounts at once.
- Boundary: Snooze ends by itself when its time runs out; turning notifications off does not. Snooze survives a restart. Snooze silences alerts only; it never hides the unread count.
- Avoid: "DND", "do not disturb", "mute" (a different thing: see Mute).
- Resolved ambiguity: Snooze is global, never per account.

## Mute
- Definition: a silence of message notifications for one account, until the user unmutes it.
- Boundary: Mute is per account and has no end time; it survives a restart. Mute silences alerts only; it never hides or changes the unread count. Snooze is separate: ending Snooze does not unmute, and unmuting does not end Snooze.
- Avoid: "DND", "do not disturb", "snooze" (global and time-bounded).
- Resolved ambiguity: a muted account still shows its unread count in the sidebar, the tray and the Dock badge.

## Deep link
- Definition: an external `whatsapp://` or `wa.me` link that asks the app to open a chat.
- Boundary: a deep link brings the app to the front and opens the chat; it never creates or removes an account.
- Avoid: "URL handler", "protocol link".
- Resolved ambiguity: a deep link opens in the active account, without asking which one.
