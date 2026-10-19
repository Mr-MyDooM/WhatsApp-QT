# 09: Rebuild the app chrome in React with the macOS look

**What to build:** the sidebar, settings window, rename window and error panel look and feel like a native macOS app (Apple Human Interface Guidelines), built with React, TypeScript, Vite, Tailwind and shadcn/ui as ADR 0002 says. The sidebar is translucent (window vibrancy). Text uses the system font. Light, Dark and System themes follow macOS. The Settings window has a toolbar with one icon and label per pane (General, Notifications, Accounts, About), like System Settings apps. Every behaviour from tickets 01, 02, 03, 05 (local parts), 04 and 08 stays the same: only the chrome changes.

**Blocked by:** 04 (Mute one account), because it changes the same chrome. Decision: the user picked "macOS native HIG" and "React+TS+Vite+shadcn" on 2026-10-19.

**Status:** ready-for-agent

- [x] The frontend is a Vite + React + TypeScript project with Tailwind and shadcn/ui. pnpm, exact versions, lock file committed.
- [ ] `cargo tauri build` and the macOS packaging script build the frontend first. `cargo tauri dev` runs with hot reload.
- [ ] The sidebar is translucent like the Mail or Messages sidebar, in Light and Dark.
- [ ] Avatars, active marker, unread pill, mute mark, tooltips, arrow keys and VoiceOver names work as tickets 01 and 04 say.
- [ ] The Settings window has a toolbar with the four panes and uses native-looking controls (switches, segmented controls, sliders).
- [ ] The rename window and the error panel use the same components and look.
- [ ] Animations follow `prefers-reduced-motion`. Text contrast meets 4.5:1 in both themes.
- [x] The strict CSP stays. The `web.whatsapp.com` origin still gets exactly one IPC command. No IPC command is added for the chrome.
- [x] Account logins are not touched: no QR re-scan after this change.
