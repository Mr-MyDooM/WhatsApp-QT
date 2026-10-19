# 12: Profile picture avatars

**What to build:** each account's avatar in the sidebar, tray menu where possible, and Settings shows the logged-in user's own WhatsApp profile picture. Colored initials stay as the fallback (logged out, no picture, load error).

**Blocked by:** 11

**Status:** resolved

- [x] The injected script in each account webview reads the user's own profile picture once logged in, and again when it changes. It sends it through one new narrow remote IPC command: image only, as a `data:image/jpeg` or `data:image/png` URL, size capped (for example 64 KB, 96x96 px), rate limited. Anything else is refused.
- [x] The picture is stored on disk per account (in the app data folder, not in `settings.json`) and shown at next start before WhatsApp loads. Removing an account deletes it.
- [x] Sidebar and Settings show the picture; initials show when there is none.
- [x] ADR 0001 is amended: the remote origin gets exactly two commands (notify, profile picture), with the reason (user decision 2026-10-19).
- [x] Strict CSP stays; `img-src 'self' data:` covers it.
- [x] Core tests: size cap, wrong type refused, removal deletes the picture.
- [x] Manual macOS check: logged-in accounts show their picture; a QR-screen account shows initials.
