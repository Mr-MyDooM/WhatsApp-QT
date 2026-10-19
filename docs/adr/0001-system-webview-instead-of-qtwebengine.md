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
- WKWebView has no `window.Notification`. One injected shim sends message notifications through IPC on all OSes. The `web.whatsapp.com` origin gets exactly one IPC command.
- The first real build must check isolation: with account A logged in, account B still shows the QR code.
- Linux starts on X11/XWayland only, because child webviews draw at the wrong bounds on native Wayland.
