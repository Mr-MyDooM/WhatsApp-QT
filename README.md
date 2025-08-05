
# WhatsApp-QT

🧙 A simple, privacy-friendly WhatsApp-like desktop client built in Rust + QML (Qt), inspired by ZapZap.  
Designed to be cross-platform (Linux & Windows) and lightweight — perfect for system tray usage, theming, and tabbed chat-like browsing.

---

## 📦 Features

- 🖼️ Beautiful QML frontend powered by Qt
- 🦀 Rust backend using [`qmetaobject`](https://crates.io/crates/qmetaobject)
- 🧭 Multi-tab support (like browser tabs)
- 🎨 Light/Dark/system themes
- 🔔 System tray support (GTK-based)
- 💾 Persistent settings (saved to `~/.config/WhatsApp-QT`)
- 🔧 Designed for Linux (tested on openSUSE Tumbleweed + Plasma), Windows coming soon

---

## 🚀 Getting Started

### 🧰 Prerequisites

> You should have Rust (stable), Qt, and GTK dev libraries installed.

```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh

# GTK & Qt dev (for openSUSE Tumbleweed)
sudo zypper install gtk3-devel libqt5-qtbase-devel libqt5-qtdeclarative-devel
```

---

### 📥 Clone & Build

```bash
git clone https://github.com/yourusername/whatsapp-qt.git
cd whatsapp-qt

# Build & run
cargo run
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
| `GTK has not been initialized` | Add `gtk::init()` before tray icon code |
| `settings` borrow/move panic | Avoid using `.clone()` inside `tokio::spawn`, or switch to sync I/O |
| Tray icon not showing | Ensure `gtk3` is installed and tray icon path is correct |
| GUI not launching | Ensure Qt5 dev packages are installed (`libqt5-*`) |

---

## 💡 Roadmap

- [x] Theme toggling
- [x] Tray integration
- [x] Save user settings
- [ ] Save + restore tabs
- [ ] Multi-account support
- [ ] Windows support
- [ ] Flatpak build

---

## 🤝 Contributing

Want to help make this better? Open an issue or submit a PR!
All help welcome — especially if you're new to Rust or QML.

---

## 🧙 Author

Made with ❤️ by M.J (`~~::Mr.MyDooM::~~`)  
Inspired by sysadmin-style tools, built for fun and learning.
