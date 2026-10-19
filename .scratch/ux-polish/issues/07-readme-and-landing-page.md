# 07: README and landing page

**What to build:** a new README for Pairline, and a one-page landing page on GitHub Pages with the same content. It has a one-line pitch ("several WhatsApp numbers side by side in one light Mac app"), a screenshot with two accounts, and install steps for an unsigned app on macOS 15 and later (open, then System Settings, Privacy and Security, Open Anyway). It also has a privacy statement (no telemetry), the "not affiliated" line, how to build from source with Tauri, the correct settings path, and platform status. The Qt build instructions are removed. Source: the research report, Lens 3, "Distribution" and "Landing page".

**Blocked by:** 06 (Rename the app to Pairline).

**Status:** ready-for-agent

- [ ] The README describes the Tauri app. Nothing in it talks about Qt, QML or cxx-qt.
- [ ] The README has the pitch, a screenshot with two accounts, the install steps for "Open Anyway", the privacy statement and the "not affiliated" line.
- [ ] The settings path in the README is the real one (`<config dir>/WhatsAppDesktop/settings.json`).
- [ ] The build steps work on a clean macOS checkout.
- [ ] Any memory or size numbers are measured, not guessed. If they are not measured, they are left out.
- [ ] The landing page has the same content as the README and does not use the WhatsApp logo or green.
- [ ] Ask the user before enabling GitHub Pages on the repository.
