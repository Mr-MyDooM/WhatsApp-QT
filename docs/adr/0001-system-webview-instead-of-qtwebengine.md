---
status: accepted
---

# System webview (Tauri 2) instead of QtWebEngine

We leave Qt and QtWebEngine for Tauri 2 on the system webview (WKWebView, WebView2, WebKitGTK). Reasons: build and install weight, bundle size, memory, and native macOS integration. We rewrite in place on a long-lived branch. Qt stays on `main` until the rewrite reaches parity, then one change removes it. Users scan the QR code again once after the switch. The path and format of `settings.json` do not change.

## Considered Options

- **Tauri 2 multiwebview** (chosen): one window, one child webview per account (`Window::add_child`). On macOS a spike passed typing, accents, other input languages, the emoji picker, account switching, resize and login persistence.
- **wry + tao child webviews**: the spike also loaded WhatsApp. The emoji picker failed because there is no Edit menu, and Cmd+C and Cmd+V probably fail for the same reason. Each of these is a small fix, but Tauri also gives us tray, notification, deep-link, single-instance, autostart and window-state plugins. This is the fallback if Tauri fails.
- **One window per account with native macOS window tabs**: the fallback if child webviews fail typing in both stacks.

## Consequences

- Child webviews need Tauri's `unstable` feature, so their API can change in a minor release. Pin the exact Tauri version in `Cargo.toml`, and do each Tauri upgrade as a separate task.
- Minimum macOS 14: each account gets its own WKWebView data store (`data_store_identifier`), named by the account's UUID. Linux and Windows use a per-account `data_directory`.
- WhatsApp Web rejects the default WKWebView user agent ("WhatsApp works with Safari 15+"). The app sets a fixed Safari user agent in code. It is not a user setting.
- WKWebView has no `window.Notification`. One injected shim sends message notifications through IPC on all OSes. The `web.whatsapp.com` origin gets exactly two IPC commands: `notify` and `profile_picture`.
- The first real build must check isolation: with account A logged in, account B still shows the QR code.
- Linux starts on X11/XWayland only, because child webviews draw at the wrong bounds on native Wayland.

## Amendment 2026-10-19: profile pictures

The user decided that each account shows its own WhatsApp profile picture. A second injected script reads the user's own avatar in the left pane header and sends it as a small JPEG data URL. This needs one more remote command, so the remote origin now gets exactly two: `notify` and `profile_picture`. Reason: the picture exists only inside the page, and the Rust side cannot read it any other way.

Limits that keep the second command narrow:

- The account comes from the webview label (`account-<id>`), never from the page data. Calls from other webviews and for unknown accounts are ignored.
- Rust checks the data URL: only `data:image/jpeg;base64,` or `data:image/png;base64,`, at most 64 KB after decoding, valid base64, and the right magic bytes.
- One call per 10 seconds for each account. Other calls are dropped.
- The picture is a file per account in the app data folder (`avatars/<id>.txt`), not in `settings.json`. Removing the account deletes it.
- The CSP does not change: `img-src 'self' data:` already covers the picture.
- The page selector can break when WhatsApp changes its page (see `src/avatar_shim.js`). Then the app shows colored initials.
