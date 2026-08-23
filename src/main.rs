use std::path::PathBuf;

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

fn main() {
    env_logger::init();

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
