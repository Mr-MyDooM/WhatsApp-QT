# 11: Collapsible sidebar, Settings in sidebar, unified title bar

**What to build:** the user can widen the sidebar to show account names and unread counts, and collapse it back to the avatar column. The sidebar has a Settings button. The title bar and the sidebar use one material, with no colour seam (user report: grey title bar strip above a different colour).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Sidebar has two states: avatar column (default) and wide (about 220 px) with avatar, account name, unread count and mute mark. A toggle button in the sidebar and View menu item "Toggle Sidebar" (Cmd+Ctrl+S) switch it. The account webview resizes to match. The state is saved in `settings.json` and restored.
- [ ] A Settings button (gear icon) at the bottom of the sidebar opens Settings, same as Cmd+,.
- [ ] macOS title bar follows HIG for sidebar apps: the sidebar material runs to the top of the window under the traffic lights; no grey title bar strip; traffic lights never cover account content or sidebar controls. The window stays draggable from the sidebar top area.
- [x] Old `settings.json` without the new field loads (default: avatar column). Core test covers it.
- [x] Keyboard and VoiceOver: the toggle and Settings buttons have names and focus rings.
- [ ] Manual macOS check: toggle both ways and restart; no colour seam in Light and Dark; window drags from the top of the sidebar.
