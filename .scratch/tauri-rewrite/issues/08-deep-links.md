# 08: Deep links

**What to build:** a `whatsapp://` or `https://wa.me/` link brings the app to the front and opens the chat in the active account, whether the app is closed or running. A second copy never starts.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Deep-link plugin for `whatsapp://`, including the macOS Apple Event.
- [ ] Single-instance plugin: a link clicked while the app runs goes to the running copy.
- [ ] A deep link opens in the active account and never adds or removes an account.
- [ ] Deep-link recognition moves into the core; the current deep-link test cases carry over (other URLs and arguments are refused).
- [ ] Manual macOS check, with the bundled app in /Applications: a `wa.me` link opens the chat when the app is closed and when it runs.
