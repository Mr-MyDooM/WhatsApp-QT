use std::path::PathBuf;

mod autostart;
mod cxxqt_object;
mod notification;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

/// Root of the app's runtime data (`resources/`). QML itself is compiled into the
/// binary via the Qt resource system (see build.rs), so this is only used to locate
/// the tray icon and notification icon.
///
/// The `qml/main.qml` probe is kept as the "am I running from a repo checkout?"
/// marker, which is why the packaging scripts still ship the `qml/` tree.
pub fn find_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WHATSAPPQT_DATA_DIR") {
        return PathBuf::from(dir);
    }
    // Development: running from repo root
    if PathBuf::from("qml/main.qml").exists() {
        return PathBuf::from(".");
    }
    // Installed: binary is in bin/, data is in ../share/whatsapp-qt/
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let share = bin_dir.join("../share/whatsapp-qt");
            if let Ok(canonical) = share.canonicalize() {
                if canonical.join("qml/main.qml").exists() {
                    return canonical;
                }
            }
        }
    }
    PathBuf::from(".")
}

/// A `whatsapp:` / `https://wa.me/...` URL the OS passed on the command line, if any.
/// Filled once before the QML loads; QML pulls it via
/// `AppController.take_pending_deep_link()` and raises the window.
static PENDING_DEEP_LINK: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// argv is how Linux and Windows deliver a registered URI scheme. macOS uses an
/// Apple Event instead (see the deep-link note in CLAUDE.md).
fn deep_link_from_args() -> Option<String> {
    first_deep_link(std::env::args().skip(1))
}

fn first_deep_link(args: impl Iterator<Item = String>) -> Option<String> {
    let mut args = args;
    args.find(|a| {
        a.starts_with("whatsapp:")
            || a.starts_with("https://wa.me/")
            || a.starts_with("http://wa.me/")
    })
}

#[cfg(test)]
mod tests {
    use super::first_deep_link;

    fn args(v: &[&str]) -> std::vec::IntoIter<String> {
        v.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter()
    }

    #[test]
    fn recognises_both_deep_link_forms_and_ignores_other_args() {
        assert_eq!(
            first_deep_link(args(&["--flag", "whatsapp://send?phone=1"])),
            Some("whatsapp://send?phone=1".to_string())
        );
        assert_eq!(
            first_deep_link(args(&["https://wa.me/123"])),
            Some("https://wa.me/123".to_string())
        );
        // Not a deep link: an unrelated https URL must not raise the window.
        assert_eq!(first_deep_link(args(&["https://web.whatsapp.com"])), None);
        assert_eq!(first_deep_link(args(&["--start-minimized"])), None);
        assert_eq!(first_deep_link(args(&[])), None);
    }
}

/// Takes (and clears) the pending deep link. Clearing means a raise happens once
/// per delivery, not on every QML reload.
pub fn take_pending_deep_link() -> Option<String> {
    PENDING_DEEP_LINK.lock().ok()?.take()
}

fn main() {
    env_logger::init();

    if let Some(url) = deep_link_from_args() {
        log::info!("Deep link received on command line: {}", url);
        if let Ok(mut slot) = PENDING_DEEP_LINK.lock() {
            *slot = Some(url);
        }
    }

    // Before QGuiApplication, always. WebEngineView aborts otherwise.
    cxxqt_object::qobject::whatsappqt_webengine_init();

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // QML lives in the Qt resource system under the `AppController` QML module,
    // which is also where the AppController type is registered (build.rs).
    const QML_ENTRY: &str = "qrc:/qt/qml/AppController/qml/main.qml";
    log::info!("Loading QML entry point: {}", QML_ENTRY);

    engine.pin_mut().load(&QUrl::from(QML_ENTRY));
    app.pin_mut().exec();
}
