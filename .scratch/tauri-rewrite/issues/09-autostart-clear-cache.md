# 09: Autostart and clear cache on exit

**What to build:** the user can turn start-at-login and clear-cache-on-exit on and off in settings. If start-at-login cannot change, the user sees an error.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Start-at-login setting through the autostart plugin; idempotent; an error shows a friendly message and logs the detail.
- [ ] Clear-cache-on-exit setting: on clears cached files of each account at quit, without removing logins.
- [ ] Manual macOS check: the app starts at login when on and not when off; logins survive a quit with clear-cache on.
