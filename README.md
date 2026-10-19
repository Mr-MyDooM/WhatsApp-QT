
# WhatsApp-QT

🧙 A macOS first simple, privacy-friendly WhatsApp-like desktop client built in Rust + QML (Qt), inspired by [ZapZap](https://github.com/rafatosta/zapzap).  
Designed to be cross-platform (macOS, Linux & Windows) and lightweight — perfect for system tray usage, theming, and tabbed chat-like browsing.

---

## 📦 Features

- 🖼️ Beautiful QML frontend powered by Qt6
- 🦀 Rust backend using [`cxx-qt`](https://crates.io/crates/cxx-qt)
- 🧭 Multi-tab support (like browser tabs)
- 🎨 Light/Dark/system themes
- 🔔 System tray support (Qt-native, no GTK tray needed)
- 💾 Persistent settings (saved to `~/.config/WhatsApp-QT`)
- 🔧 Designed for macOS first, then Linux; Windows is best-effort

---

## 🚀 Getting Started

### 🧰 Prerequisites

> You should have Rust (stable) and Qt6 dev libraries (including QtWebEngine) installed.

```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh

# Qt6 dev (for openSUSE Tumbleweed)
sudo zypper install qt6-base-devel qt6-declarative-devel qt6-webengine-devel
```

QtWebEngine is a **build-time** dependency: `build.rs` links the `WebEngineQuick`
module so `QtWebEngineQuick::initialize()` can run before the app is constructed.

---

### 📥 Clone & Build

```bash
git clone https://github.com/yourusername/whatsapp-qt.git
cd whatsapp-qt

# Build the chrome (needs Node and pnpm), then run
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend build
cargo run --features custom-protocol

# Dev mode with hot reload for the chrome
cargo tauri dev
```

---

## 🛠️ Project Structure

```
.
├── src/                  # Rust backend (logic, integration, settings, tray)
│   └── main.rs
├── qml/                  # QML UI files
│   ├── main.qml
│   └── components/
├── resources/            # Icons, tray images, etc.
├── Cargo.toml
└── README.md
```

---

## 🔧 Settings Location

- Config file saved at: `~/.config/WhatsApp-QT/settings.json`
- Tabs, theme, and other preferences auto-save on exit

---

## 🐛 Troubleshooting

| Issue | Fix |
|------|-----|
| `settings` borrow/move panic | Avoid using `.clone()` inside `tokio::spawn`, or switch to sync I/O |
| Tray icon not showing | Ensure a tray/StatusNotifier host is running and `qml6-module-qt-labs-platform` is installed |
| GUI not launching | Ensure Qt6 dev packages are installed (`qt6-*`) |
| New `.qml` file not found at runtime | Add it to `qml_files` in `build.rs` — QML is compiled into the binary |

---

## 💡 Roadmap

- [x] Theme toggling
- [x] Tray integration
- [x] Save user settings
- [X] Save + restore tabs
- [X] Multi-account support
- [ ] Windows support
- [ ] Flatpak build

---

## 🖥️ Platform Support

### 🍏 macOS: First-Class Citizen
This app is developed and tested primarily on macOS.  
Install Qt with `brew install qt`, then `export QMAKE="$(brew --prefix qt)/bin/qmake"` before `cargo run`.  
Build an unsigned `.app` with `packaging/macos/build-macos-app.sh`.

### 🐧 Linux: Supported
Second priority. Earlier versions were tested on openSUSE Slowroll/Tumbleweed + KDE.  
Install your distro's Qt6 dev packages and set `QMAKE` to the Qt6 `qmake`.

### 🪟 Windows: Works (Mostly)
Basic features run fine, though some quirks may exist.  
Support is "best effort": patches welcome, rants optional.

---

> ⚠️ **Note from the dev:**  
> I'm a SRE first, coding my way into developerhood.  
> Expect stability on macOS, and some occasional chaos elsewhere.
 

---
## 🤝 Contributing

Want to help make this better? Open an issue or submit a PR!
All help welcome — especially if you're new to Rust or QML.

---

## 🧙 Author

Made with ❤️ by M.J (`~~::Mr.MyDooM::~~`)  

Inspired by SRE-style tools, built for fun and learning.
