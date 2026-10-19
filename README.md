# Pairline

Several WhatsApp numbers side by side in one light Mac app.

Pairline is an unofficial desktop client for WhatsApp Web. Each account has its own login and data. A sidebar switches between accounts. The app has a tray icon, message notifications, a Dock badge with the unread count, and per-account Mute and global Snooze.

Not affiliated with or endorsed by WhatsApp or Meta. "WhatsApp" is a trademark of its owner.

## Platform status

- macOS 14 or later: supported.
- Linux and Windows: paused. The code is kept but not tested.

## Install (macOS)

Pairline is not notarized. macOS blocks it on first launch.

1. Build the app (see "Build from source") and open the `.app` or `.dmg` from `target/release/bundle/`.
2. Open Pairline once. macOS shows a warning. Close it.
3. On macOS 15 and later, open System Settings, then Privacy and Security. Scroll down and click Open Anyway.
4. Open Pairline again and confirm.

If you move from WhatsApp-QT, scan the QR code again one time. The settings file keeps its path and format.

## Privacy

- The app has no telemetry, no analytics and no update check.
- The app code makes no network call of its own. Each account loads `https://web.whatsapp.com` in a webview, and that is the only site the app opens in the app.
- Logins and chats stay on your Mac, in one separate data store per account.

## Settings

Settings are in `<config dir>/WhatsAppDesktop/settings.json`. On macOS this is `~/Library/Application Support/WhatsAppDesktop/settings.json`.

## Build from source

You need Rust (stable), Node, pnpm, and the Tauri CLI:

```bash
cargo install tauri-cli --version "^2" --locked
```

Run in dev mode with hot reload for the interface:

```bash
cargo tauri dev
```

Build the `.app` and `.dmg`:

```bash
cargo tauri build
# or
packaging/macos/build-macos-app.sh
```

Run without the Tauri CLI. Build the interface first:

```bash
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend build
cargo run --features custom-protocol
```

For backend logs, set `RUST_LOG=debug`. Without it, only errors print.

## Project layout

- `src/`: Rust backend (Tauri 2). Accounts, tray, notifications, settings.
- `frontend/`: React and TypeScript interface (sidebar, settings, rename window, error page), built with Vite.
- `docs/adr/`: design decisions.
- `GLOSSARY.md`: project terms.

## Note from the dev

I am an SRE first, coding my way into software development. Expect stability on macOS only.

## Contributing

Open an issue or a pull request.

## Author

Made by M.J (`~~::Mr.MyDooM::~~`).
