# 07: README and landing page

**What to build:** a new README for Pairline. It has a one-line pitch ("several WhatsApp numbers side by side in one light Mac app"), a screenshot with two accounts (the user adds it), and install steps for an unsigned app on macOS 15 and later (open, then System Settings, Privacy and Security, Open Anyway). It also has a privacy statement (no telemetry), the "not affiliated" line, how to build from source with Tauri, the correct settings path, and platform status. The Qt build instructions are removed. Source: the research report, Lens 3, "Distribution".

Landing page dropped by the user, 2026-10-19. No GitHub Pages.

**Blocked by:** 06 (Rename the app to Pairline).

**Status:** resolved

- [x] The README describes the Tauri app. Nothing in it talks about Qt, QML or cxx-qt.
- [x] The README has the pitch, the install steps for "Open Anyway", the privacy statement and the "not affiliated" line.
- [ ] Screenshot with two accounts: the user adds it (the agent must not launch the GUI).
- [x] The settings path in the README is the real one (`<config dir>/WhatsAppDesktop/settings.json`).
- [ ] The build steps work on a clean macOS checkout.
- [x] Any memory or size numbers are measured, not guessed. If they are not measured, they are left out.
