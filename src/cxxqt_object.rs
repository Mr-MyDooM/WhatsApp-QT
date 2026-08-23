//! The `AppController` QObject: the single bridge object the QML frontend talks to.
//!
//! Every property, signal and invokable name here is deliberately snake_case and
//! matches what `qml/**/*.qml` already calls. cxx-qt does NOT case-convert names by
//! default (no `#[auto_cxx_name]` on the extern block), so what is written below is
//! exactly what QML sees.

use cxx_qt::CxxQtType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;

use crate::notification::{NotificationService, NotificationSettings};

const DARK_BASE_CSS: &str = r#"
    /* Dark theme base styles */
    body, #app, [data-testid="conversation-panel-body"] {
        background-color: #1e1e1e !important;
        color: #ffffff !important;
        font-family: 'Fira Code', 'JetBrains Mono', 'monospace' !important;
    }
"#;
const DARK_CHAT_CSS: &str = r#"
    /* Dark theme chat styles */
    [data-testid="chat-list"] {
        background-color: #2d2d2d !important;
    }
    [data-testid="chat"] {
        background-color: #3d3d3d !important;
        border-bottom: 1px solid #555 !important;
    }
"#;
const LIGHT_BASE_CSS: &str = r#"
    /* Light theme base styles */
    body, #app, [data-testid="conversation-panel-body"] {
        background-color: #ffffff !important;
        color: #000000 !important;
        font-family: 'Fira Code', 'JetBrains Mono', 'monospace' !important;
    }
"#;
const LIGHT_CHAT_CSS: &str = r#"
    /* Light theme chat styles */
    [data-testid="chat-list"] {
        background-color: #f8f9fa !important;
    }
    [data-testid="chat"] {
        background-color: #ffffff !important;
        border-bottom: 1px solid #e9ecef !important;
    }
"#;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "C++" {
        include!("webengine_init.h");
        /// Must be called before the QGuiApplication is constructed. See
        /// include/webengine_init.h.
        fn whatsappqt_webengine_init();
    }

    extern "RustQt" {
        /// Registered into the `AppController` QML module (see build.rs), which is what
        /// makes the existing bare `import AppController` + `AppController { ... }` work.
        ///
        /// Every WRITE points at the hand-written invokable of the same name rather than
        /// letting cxx-qt synthesise one. That keeps `appController.set_theme(...)`
        /// callable from QML (as it is today) while a direct property assignment runs the
        /// same logic. cxx-qt skips setter codegen for a custom WRITE, so there is no
        /// duplicate-definition clash with the `#[qinvokable]` of the same name.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, current_tab, READ, WRITE = set_current_tab, NOTIFY = current_tab_changed)]
        #[qproperty(QString, theme, READ, WRITE = set_theme, NOTIFY = theme_changed)]
        #[qproperty(QString, download_path, READ, WRITE = set_download_path, NOTIFY = download_path_changed)]
        #[qproperty(bool, notifications_enabled, READ, WRITE = set_notifications_enabled, NOTIFY = notifications_enabled_changed)]
        #[qproperty(bool, show_message_notifications, READ, WRITE = set_show_message_notifications, NOTIFY = show_message_notifications_changed)]
        #[qproperty(bool, show_call_notifications, READ, WRITE = set_show_call_notifications, NOTIFY = show_call_notifications_changed)]
        #[qproperty(bool, notification_sound_enabled, READ, WRITE = set_notification_sound_enabled, NOTIFY = notification_sound_enabled_changed)]
        #[qproperty(bool, show_sender, READ, WRITE = set_show_sender, NOTIFY = show_sender_changed)]
        #[qproperty(bool, clear_cache_on_exit, READ, WRITE = set_clear_cache_on_exit, NOTIFY = clear_cache_on_exit_changed)]
        #[qproperty(bool, start_minimized, READ, WRITE = set_start_minimized, NOTIFY = start_minimized_changed)]
        // Resolved once at construction and never changes, so CONSTANT: no setter and no
        // notify signal. Replaces the old `trayIconPath` context property, because
        // cxx-qt-lib 0.7 binds no QQmlContext::setContextProperty equivalent.
        #[qproperty(QString, tray_icon_path, READ, CONSTANT)]
        type AppController = super::AppControllerRust;

        // ── Signals ──────────────────────────────────────────────────────────────
        // The *_changed ones are the NOTIFY targets above; QML connects to them as
        // `onTheme_changed`, `onTab_added`, `onSave_failed`, ...
        #[qsignal]
        fn current_tab_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn theme_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn download_path_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn notifications_enabled_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn show_message_notifications_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn show_call_notifications_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn notification_sound_enabled_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn show_sender_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn clear_cache_on_exit_changed(self: Pin<&mut AppController>);
        #[qsignal]
        fn start_minimized_changed(self: Pin<&mut AppController>);

        #[qsignal]
        fn save_failed(self: Pin<&mut AppController>, error: QString);
        #[qsignal]
        fn settings_saved(self: Pin<&mut AppController>);
        #[qsignal]
        fn tab_added(self: Pin<&mut AppController>, name: QString, icon: QString);
        #[qsignal]
        fn tab_removed(self: Pin<&mut AppController>, index: i32);
        #[qsignal]
        fn tab_renamed(self: Pin<&mut AppController>, index: i32, new_name: QString);
        #[qsignal]
        fn apply_theme_css(self: Pin<&mut AppController>, css_code: QString);

        // ── Invokables ───────────────────────────────────────────────────────────
        #[qinvokable]
        fn test_notification(self: &AppController);
        #[qinvokable]
        fn get_theme_css(self: &AppController) -> QString;
        #[qinvokable]
        fn get_user_agent(self: &AppController) -> QString;

        #[qinvokable]
        fn set_notifications_enabled(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_show_message_notifications(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_show_call_notifications(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_notification_sound_enabled(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_show_sender(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_clear_cache_on_exit(self: Pin<&mut AppController>, enabled: bool);
        #[qinvokable]
        fn set_start_minimized(self: Pin<&mut AppController>, enabled: bool);

        #[qinvokable]
        fn add_tab(self: Pin<&mut AppController>, name: QString, icon: QString);
        #[qinvokable]
        fn remove_tab(self: Pin<&mut AppController>, index: i32);
        #[qinvokable]
        fn rename_tab(self: Pin<&mut AppController>, index: i32, new_name: QString);
        #[qinvokable]
        fn set_current_tab(self: Pin<&mut AppController>, index: i32);
        #[qinvokable]
        fn set_theme(self: Pin<&mut AppController>, theme: QString);
        #[qinvokable]
        fn set_download_path(self: Pin<&mut AppController>, path: QString);
        #[qinvokable]
        fn save_settings(self: Pin<&mut AppController>);
        #[qinvokable]
        fn load_settings(self: Pin<&mut AppController>);
    }
}

use cxx_qt_lib::QString;

/// Backing data for the `AppController` QObject. Fields listed in a `#[qproperty]`
/// above are reachable from QML; the rest are plain Rust state.
pub struct AppControllerRust {
    pub current_tab: i32,
    pub theme: QString,
    pub download_path: QString,
    pub notifications_enabled: bool,
    pub show_message_notifications: bool,
    pub show_call_notifications: bool,
    pub notification_sound_enabled: bool,
    pub show_sender: bool,
    pub clear_cache_on_exit: bool,
    pub start_minimized: bool,
    pub tray_icon_path: QString,
    // Not a Q_PROPERTY: QML only ever reads it through get_theme_css().
    pub css_cache: QString,
    pub notification_service: NotificationService,
    pub tabs: Vec<TabInfo>,
}

impl Default for AppControllerRust {
    fn default() -> Self {
        Self {
            current_tab: 0,
            theme: QString::from("system"),
            download_path: QString::from(&default_download_path()),
            notifications_enabled: true,
            show_message_notifications: true,
            show_call_notifications: true,
            notification_sound_enabled: true,
            show_sender: true,
            clear_cache_on_exit: false,
            start_minimized: false,
            tray_icon_path: QString::from(&tray_icon_url()),
            css_cache: QString::default(),
            notification_service: NotificationService::new(),
            tabs: Vec::new(),
        }
    }
}

impl AppControllerRust {
    fn update_notification_settings(&mut self) {
        let settings = NotificationSettings {
            enabled: self.notifications_enabled,
            show_message_notifications: self.show_message_notifications,
            show_call_notifications: self.show_call_notifications,
            sound_enabled: self.notification_sound_enabled,
            show_sender: self.show_sender,
        };
        self.notification_service.update_settings(settings);
    }
}

impl qobject::AppController {
    pub fn test_notification(&self) {
        match self.notification_service.test_notification() {
            Ok(_) => log::info!("Test notification sent successfully"),
            Err(e) => log::error!("Failed to send test notification: {}", e),
        }
    }

    pub fn get_theme_css(&self) -> QString {
        self.css_cache.clone()
    }

    pub fn get_user_agent(&self) -> QString {
        let user_agent = if cfg!(target_os = "linux") {
            "Mozilla/5.0 (X11; Linux x86_64; Chromium) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.4.0 Safari/537.38"
        } else if cfg!(target_os = "macos") {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1.1 Safari/605.1.15"
        } else if cfg!(target_os = "windows") {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
        } else {
            "Mozilla/5.0 (X11; Linux x86_64; Chromium) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.4.0 Safari/537.38"
        };

        QString::from(user_agent)
    }

    pub fn set_notifications_enabled(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().notifications_enabled = enabled;
        self.as_mut().notifications_enabled_changed();
        self.as_mut().rust_mut().update_notification_settings();
        self.as_mut().save_settings();
    }

    pub fn set_show_message_notifications(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().show_message_notifications = enabled;
        self.as_mut().show_message_notifications_changed();
        self.as_mut().rust_mut().update_notification_settings();
        self.as_mut().save_settings();
    }

    pub fn set_show_call_notifications(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().show_call_notifications = enabled;
        self.as_mut().show_call_notifications_changed();
        self.as_mut().rust_mut().update_notification_settings();
        self.as_mut().save_settings();
    }

    pub fn set_notification_sound_enabled(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().notification_sound_enabled = enabled;
        self.as_mut().notification_sound_enabled_changed();
        self.as_mut().rust_mut().update_notification_settings();
        self.as_mut().save_settings();
    }

    pub fn set_show_sender(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().show_sender = enabled;
        self.as_mut().show_sender_changed();
        self.as_mut().rust_mut().update_notification_settings();
        self.as_mut().save_settings();
    }

    pub fn set_clear_cache_on_exit(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().clear_cache_on_exit = enabled;
        self.as_mut().clear_cache_on_exit_changed();
        self.as_mut().save_settings();
    }

    pub fn set_start_minimized(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().start_minimized = enabled;
        self.as_mut().start_minimized_changed();
        self.as_mut().save_settings();
    }

    pub fn add_tab(mut self: Pin<&mut Self>, name: QString, icon: QString) {
        self.as_mut().tab_added(name, icon);
        self.as_mut().save_settings();
    }

    pub fn remove_tab(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().tab_removed(index);
        self.as_mut().save_settings();
    }

    pub fn rename_tab(mut self: Pin<&mut Self>, index: i32, new_name: QString) {
        self.as_mut().tab_renamed(index, new_name);
        self.as_mut().save_settings();
    }

    pub fn set_current_tab(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().current_tab = index;
        self.as_mut().current_tab_changed();
    }

    pub fn set_theme(mut self: Pin<&mut Self>, theme: QString) {
        log::info!("Setting theme to: {}", theme);
        self.as_mut().rust_mut().theme = theme.clone();

        // Generate CSS before emitting signals
        let css = generate_css(&theme.to_string());
        log::debug!("Generated CSS ({} bytes): {}", css.len(), css);
        let css_qs = QString::from(&css);
        self.as_mut().rust_mut().css_cache = css_qs.clone();
        // Emit signals to update UI
        self.as_mut().theme_changed();
        self.as_mut().apply_theme_css(css_qs);
        // Save settings
        self.as_mut().save_settings();
    }

    pub fn set_download_path(mut self: Pin<&mut Self>, path: QString) {
        self.as_mut().rust_mut().download_path = path;
        self.as_mut().download_path_changed();
        self.as_mut().save_settings();
    }

    pub fn save_settings(mut self: Pin<&mut Self>) {
        let settings = AppSettings {
            theme: self.theme.to_string(),
            download_path: self.download_path.to_string(),
            current_tab: self.current_tab,
            notification_settings: NotificationSettings {
                enabled: self.notifications_enabled,
                show_message_notifications: self.show_message_notifications,
                show_call_notifications: self.show_call_notifications,
                sound_enabled: self.notification_sound_enabled,
                show_sender: self.show_sender,
            },
            clear_cache_on_exit: self.clear_cache_on_exit,
            start_minimized: self.start_minimized,
            tabs: self
                .tabs
                .iter()
                .map(|t| TabInfo {
                    name: t.name.clone(),
                    icon: t.icon.clone(),
                })
                .collect(),
        };

        let Some(cfg) = get_config_path() else {
            log::error!("Could not find config directory; settings not saved");
            self.as_mut()
                .save_failed(QString::from("Could not find config directory"));
            return;
        };

        let json = match serde_json::to_string_pretty(&settings) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize settings: {}", e);
                self.as_mut()
                    .save_failed(QString::from("Failed to serialize settings"));
                return;
            }
        };

        match fs::write(cfg.join("settings.json"), &json) {
            Ok(_) => {
                log::debug!("Settings saved to {}", cfg.join("settings.json").display());
                self.as_mut().settings_saved();
            }
            Err(e) => {
                log::error!("Failed to write settings: {}", e);
                self.as_mut().save_failed(QString::from(&e.to_string()));
            }
        }
    }

    pub fn load_settings(mut self: Pin<&mut Self>) {
        let Some(cfg) = get_config_path() else {
            log::warn!("Could not find config directory; using default settings");
            return;
        };
        let settings_file = cfg.join("settings.json");
        let s = match fs::read_to_string(&settings_file) {
            Err(e) => {
                // Missing file on first run is normal; anything else is worth a warning.
                if e.kind() == std::io::ErrorKind::NotFound {
                    log::info!("No settings file yet, using defaults");
                } else {
                    log::warn!(
                        "Could not read {}: {}; using defaults",
                        settings_file.display(),
                        e
                    );
                }
                return;
            }
            Ok(data) => match serde_json::from_str::<AppSettings>(&data) {
                Err(e) => {
                    log::warn!(
                        "Malformed settings file {}: {}; using defaults",
                        settings_file.display(),
                        e
                    );
                    return;
                }
                Ok(s) => s,
            },
        };

        {
            let mut rust = self.as_mut().rust_mut();
            rust.theme = QString::from(&s.theme);
            rust.download_path = QString::from(&s.download_path);
            rust.current_tab = s.current_tab;

            // Load notification settings
            rust.notifications_enabled = s.notification_settings.enabled;
            rust.show_message_notifications = s.notification_settings.show_message_notifications;
            rust.show_call_notifications = s.notification_settings.show_call_notifications;
            rust.notification_sound_enabled = s.notification_settings.sound_enabled;
            rust.show_sender = s.notification_settings.show_sender;
            rust.clear_cache_on_exit = s.clear_cache_on_exit;
            rust.start_minimized = s.start_minimized;

            // Update notification service
            rust.update_notification_settings();
        }

        // Emit all change signals
        self.as_mut().theme_changed();
        self.as_mut().download_path_changed();
        self.as_mut().current_tab_changed();
        self.as_mut().notifications_enabled_changed();
        self.as_mut().show_message_notifications_changed();
        self.as_mut().show_call_notifications_changed();
        self.as_mut().notification_sound_enabled_changed();
        self.as_mut().show_sender_changed();
        self.as_mut().clear_cache_on_exit_changed();
        self.as_mut().start_minimized_changed();
    }
}

fn generate_css(theme: &str) -> String {
    let mut css = String::new();
    log::debug!("Generating CSS for theme: {}", theme);

    if theme == "dark" {
        css.push_str(DARK_BASE_CSS);
        css.push_str(DARK_CHAT_CSS);
        // Add more comprehensive dark theme CSS
        css.push_str(
            r#"
                /* Additional dark theme styles */
                ._3OtEr, .app, #app, .app-wrapper-web, [data-testid="app-wrapper"] {
                    background-color: #1e1e1e !important;
                }
                ._3j7s9 {
                    background-color: #2d2d2d !important;
                }
                /* WhatsApp specific dark theme */
                [data-testid="chatlist-header"], [data-testid="chat-header"] {
                    background-color: #2d2d2d !important;
                }
            "#,
        );
    } else if theme == "light" {
        css.push_str(LIGHT_BASE_CSS);
        css.push_str(LIGHT_CHAT_CSS);
        // Add more comprehensive light theme CSS
        css.push_str(
            r#"
                /* Additional light theme styles */
                ._3OtEr, .app, #app, .app-wrapper-web, [data-testid="app-wrapper"] {
                    background-color: #ffffff !important;
                }
                ._3j7s9 {
                    background-color: #f8f9fa !important;
                }
            "#,
        );
    }

    css
}

#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    theme: String,
    download_path: String,
    current_tab: i32,
    notification_settings: NotificationSettings,
    #[serde(default)]
    clear_cache_on_exit: bool,
    #[serde(default)]
    start_minimized: bool,
    tabs: Vec<TabInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TabInfo {
    name: String,
    icon: String,
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(mut path) = dirs::config_dir() {
        path.push("WhatsAppDesktop");
        let _ = fs::create_dir_all(&path);
        Some(path)
    } else {
        None
    }
}

fn default_download_path() -> String {
    dirs::download_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| {
            log::warn!("No download dir and no accessible working dir; using \".\"");
            PathBuf::from(".")
        })
        .to_string_lossy()
        .to_string()
}

/// Tray icon as a `file://` URL so QML's `icon.source` can load it directly.
/// Empty string if the icon is missing, which QML treats as "no icon".
fn tray_icon_url() -> String {
    let tray_icon = crate::find_data_dir().join("resources/icons/tray.png");
    match tray_icon.canonicalize() {
        Ok(p) => format!("file://{}", p.to_string_lossy()),
        Err(e) => {
            log::warn!("Tray icon not found at {}: {}", tray_icon.display(), e);
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The only real branching logic in this file: theme -> CSS. Anything else here is
    // Qt plumbing that needs a running QApplication to exercise.
    #[test]
    fn generate_css_covers_each_theme() {
        let dark = generate_css("dark");
        assert!(dark.contains("#1e1e1e"), "dark theme must set its base bg");
        assert!(
            dark.contains("chatlist-header"),
            "dark adds WhatsApp overrides"
        );

        let light = generate_css("light");
        assert!(
            light.contains("#ffffff"),
            "light theme must set its base bg"
        );
        assert!(
            !light.contains("#1e1e1e"),
            "light must not leak dark colors"
        );

        // "system" (and anything unrecognised) means "inject nothing".
        assert!(generate_css("system").is_empty());
        assert!(generate_css("").is_empty());
    }

    #[test]
    fn settings_roundtrip_preserves_new_flags() {
        let json = serde_json::json!({
            "theme": "dark",
            "download_path": "/tmp",
            "current_tab": 2,
            "notification_settings": {
                "enabled": true,
                "show_message_notifications": false,
                "show_call_notifications": true,
                "sound_enabled": false,
                "show_sender": true
            },
            "tabs": []
        })
        .to_string();

        // clear_cache_on_exit / start_minimized are #[serde(default)], so an older
        // settings.json without them must still load.
        let s: AppSettings = serde_json::from_str(&json).expect("legacy settings must load");
        assert_eq!(s.current_tab, 2);
        assert!(!s.clear_cache_on_exit);
        assert!(!s.start_minimized);
    }
}
