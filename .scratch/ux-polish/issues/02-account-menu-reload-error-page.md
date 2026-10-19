# 02: Account menu, rename, reload and error page

**What to build:** right-click an account in the sidebar to open a native menu with Rename, Reload and Remove. Rename changes the account name, and the new name shows in the sidebar, the tray and message notifications. Reload loads the account's WhatsApp Web again, and Cmd+R reloads the active account. Remove asks for confirmation in a native dialog that names the account. This replaces the `confirm()` call. When an account's page does not load, or the Mac is offline, the account area shows "Cannot reach WhatsApp. Check your connection." with a Retry button. The app also tries again by itself, with backoff. Retry and the automatic retry use the same reload path. Sources: GLOSSARY.md "Account name", and the research report, Lens 2, "Sidebar" and "Error states".

**Blocked by:** 01 (Sidebar redesign).

**Status:** resolved

- [x] Right-click on an account opens a native menu with Rename, Reload and Remove.
- [x] Rename lets the user type a new account name. Empty or whitespace-only names are refused. The name is trimmed and has a maximum length.
- [x] A renamed account shows its new name in the sidebar, the tray and message notifications. The name is saved and stays after a restart.
- [x] Renaming never changes the account's UUID or data store, so the account stays logged in.
- [x] Reload in the menu reloads that account only. Cmd+R reloads the active account only.
- [x] Remove shows a native confirmation that names the account. Cancel leaves the account as it was.
- [x] A failed page load or an offline Mac shows the error panel in that account's area only. The other accounts are not affected.
- [x] Retry reloads the account. The automatic retry uses backoff with a maximum interval, and stops when the page loads.
- [x] The error panel goes away when the page loads.
- [x] Rename, remove and the retry schedule are tested through the core (event in, effects out).
- [x] Saves still follow the save-blocked rule.
