use cstr::cstr;
use dirs::config_dir;
use qmetaobject::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
mod notification;
mod qml_resources;

use notification::{NotificationService, NotificationSettings};

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

#[derive(QObject)]
struct AppController {
    base: qt_base_class!(trait QObject),
    current_tab: qt_property!(i32; NOTIFY current_tab_changed),
    current_tab_changed: qt_signal!(),
    theme: qt_property!(QString; NOTIFY theme_changed),
    theme_changed: qt_signal!(),
    download_path: qt_property!(QString; NOTIFY download_path_changed),
    download_path_changed: qt_signal!(),
    css_cache: qt_property!(QString;),
    save_failed: qt_signal!(error: QString),
    tab_added: qt_signal!(name: QString, icon: QString),
    tab_removed: qt_signal!(index: i32),
    tab_renamed: qt_signal!(index: i32, new_name: QString),
    settings_saved: qt_signal!(),
    apply_theme_css: qt_signal!(css_code: QString),
    notification_service: NotificationService,
    notifications_enabled: qt_property!(bool; NOTIFY notifications_enabled_changed),
    notifications_enabled_changed: qt_signal!(),
    show_message_notifications: qt_property!(bool; NOTIFY show_message_notifications_changed),
    show_message_notifications_changed: qt_signal!(),
    show_call_notifications: qt_property!(bool; NOTIFY show_call_notifications_changed),
    show_call_notifications_changed: qt_signal!(),
    notification_sound_enabled: qt_property!(bool; NOTIFY notification_sound_enabled_changed),
    notification_sound_enabled_changed: qt_signal!(),
    show_sender: qt_property!(bool; NOTIFY show_sender_changed),
    show_sender_changed: qt_signal!(),
    tabs: Vec<TabInfo>,

    test_notification: qt_method!(
        fn test_notification(&self) {
            match self.notification_service.test_notification() {
                Ok(_) => println!("Test notification sent successfully"),
                Err(e) => println!("Failed to send test notification: {}", e),
            }
        }
    ),
    set_notifications_enabled: qt_method!(
        fn set_notifications_enabled(&mut self, enabled: bool) {
            self.notifications_enabled = enabled;
            self.notifications_enabled_changed();
            self.update_notification_settings();
            self.save_settings();
        }
    ),
    set_show_message_notifications: qt_method!(
        fn set_show_message_notifications(&mut self, enabled: bool) {
            self.show_message_notifications = enabled;
            self.show_message_notifications_changed();
            self.update_notification_settings();
            self.save_settings();
        }
    ),
    set_show_call_notifications: qt_method!(
        fn set_show_call_notifications(&mut self, enabled: bool) {
            self.show_call_notifications = enabled;
            self.show_call_notifications_changed();
            self.update_notification_settings();
            self.save_settings();
        }
    ),
    set_notification_sound_enabled: qt_method!(
        fn set_notification_sound_enabled(&mut self, enabled: bool) {
            self.notification_sound_enabled = enabled;
            self.notification_sound_enabled_changed();
            self.update_notification_settings();
            self.save_settings();
        }
    ),
    set_show_sender: qt_method!(
        fn set_show_sender(&mut self, enabled: bool) {
            self.show_sender = enabled;
            self.show_sender_changed();
            self.update_notification_settings();
            self.save_settings();
        }
    ),

    get_theme_css: qt_method!(
        fn get_theme_css(&self) -> QString {
            self.css_cache.clone()
        }
    ),

    get_user_agent: qt_method!(
        fn get_user_agent(&self) -> QString {
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
    ),

    add_tab: qt_method!(
        fn add_tab(&mut self, name: QString, icon: QString) {
            self.tab_added(name, icon);
            self.save_settings();
        }
    ),
    remove_tab: qt_method!(
        fn remove_tab(&mut self, index: i32) {
            self.tab_removed(index);
            self.save_settings();
        }
    ),
    rename_tab: qt_method!(
        fn rename_tab(&mut self, index: i32, new_name: QString) {
            self.tab_renamed(index, new_name);
            self.save_settings();
        }
    ),
    set_current_tab: qt_method!(
        fn set_current_tab(&mut self, index: i32) {
            self.current_tab = index;
            self.current_tab_changed();
        }
    ),

    set_theme: qt_method!(
        fn set_theme(&mut self, theme: QString) {
            println!("Setting theme to: {}", theme.to_string());
            self.theme = theme.clone();

            // Generate CSS before emitting signals
            let css = self.generate_css(theme.to_string());
            self.css_cache = QString::from(css.clone());
            println!("Generated CSS Length: {}", css.len());
            println!("CSS content: {}", css);
            // Emit signals to update UI
            self.theme_changed();
            self.apply_theme_css(self.css_cache.clone());
            // Save settings
            self.save_settings();
        }
    ),

    set_download_path: qt_method!(
        fn set_download_path(&mut self, path: QString) {
            self.download_path = path;
            self.download_path_changed();
            self.save_settings();
        }
    ),
    save_settings: qt_method!(
        fn save_settings(&self) {
            let mut settings = AppSettings {
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
                tabs: self
                    .tabs
                    .iter()
                    .map(|t| TabInfo {
                        name: t.name.to_string(),
                        icon: t.icon.to_string(),
                    })
                    .collect(),
            };

            settings.encrypt_fields();
            let cfg_path = get_config_path();
            let async_settings = settings.clone();

            if let Some(cfg) = get_config_path() {
                if let Ok(json) = serde_json::to_string_pretty(&settings) {
                    let _ = std::fs::write(cfg.join("settings.json"), &json);
                }
            }

            if let Some(cfg) = get_config_path() {
                if let Ok(json) = serde_json::to_string_pretty(&settings) {
                    match fs::write(cfg.join("settings.json"), &json) {
                        Ok(_) => self.settings_saved(),
                        Err(e) => self.save_failed(QString::from(e.to_string())),
                    }
                } else {
                    self.save_failed(QString::from("Failed to serialize settings"));
                }
            } else {
                self.save_failed(QString::from("Could not find config directory"));
            }
        }
    ),
    load_settings: qt_method!(
        fn load_settings(&mut self) {
            if let Some(cfg) = get_config_path() {
                if let Ok(data) = fs::read_to_string(cfg.join("settings.json")) {
                    if let Ok(s) = serde_json::from_str::<AppSettings>(&data) {
                        self.theme = s.theme.into();
                        self.download_path = s.download_path.into();
                        self.current_tab = s.current_tab;

                        // Load notification settings
                        self.notifications_enabled = s.notification_settings.enabled;
                        self.show_message_notifications =
                            s.notification_settings.show_message_notifications;
                        self.show_call_notifications =
                            s.notification_settings.show_call_notifications;
                        self.notification_sound_enabled = s.notification_settings.sound_enabled;
                        self.show_sender = s.notification_settings.show_sender;

                        // Emit all change signals
                        self.theme_changed();
                        self.download_path_changed();
                        self.current_tab_changed();
                        self.notifications_enabled_changed();
                        self.show_message_notifications_changed();
                        self.show_call_notifications_changed();
                        self.notification_sound_enabled_changed();
                        self.show_sender_changed();

                        // Update notification service
                        self.update_notification_settings();
                    }
                }
            }
        }
    ),
}

impl Default for AppController {
    fn default() -> Self {
        Self {
            base: Default::default(),
            current_tab: 0,
            current_tab_changed: Default::default(),
            theme: "system".into(),
            theme_changed: Default::default(),
            download_path: dirs::download_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .to_string_lossy()
                .to_string()
                .into(),
            download_path_changed: Default::default(),
            css_cache: QString::default(),
            save_failed: Default::default(),
            tab_added: Default::default(),
            tab_removed: Default::default(),
            tab_renamed: Default::default(),
            settings_saved: Default::default(),
            apply_theme_css: Default::default(),
            get_theme_css: Default::default(),
            get_user_agent: Default::default(),
            add_tab: Default::default(),
            remove_tab: Default::default(),
            rename_tab: Default::default(),
            set_current_tab: Default::default(),
            set_theme: Default::default(),
            set_download_path: Default::default(),
            save_settings: Default::default(),
            load_settings: Default::default(),
            notification_service: NotificationService::new(),
            notifications_enabled: true,
            notifications_enabled_changed: Default::default(),
            show_message_notifications: true,
            show_message_notifications_changed: Default::default(),
            show_call_notifications: true,
            show_call_notifications_changed: Default::default(),
            notification_sound_enabled: true,
            notification_sound_enabled_changed: Default::default(),
            show_sender: true,
            show_sender_changed: Default::default(),
            test_notification: Default::default(),
            set_notifications_enabled: Default::default(),
            set_show_message_notifications: Default::default(),
            set_show_call_notifications: Default::default(),
            set_notification_sound_enabled: Default::default(),
            set_show_sender: Default::default(),
            tabs: Vec::new(),
        }
    }
}

impl AppController {
    fn generate_css(&self, theme: String) -> String {
        let mut css = String::new();
        println!("Generating CSS for theme: {}", theme);

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

        println!("Generated CSS: {}", css);
        css
    }

    fn update_notification_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = NotificationSettings {
            enabled: self.notifications_enabled,
            show_message_notifications: self.show_message_notifications,
            show_call_notifications: self.show_call_notifications,
            sound_enabled: self.notification_sound_enabled,
            show_sender: self.show_sender,
        };
        let _icon_path = std::env::current_dir()
            .unwrap()
            .join("resources/icons/tray.png")
            .to_string_lossy()
            .to_string();
        self.notification_service.update_settings(settings);
        Ok(()) // ← REQUIRED!
    }
}
impl AppSettings {
    pub fn encrypt_fields(&mut self) {
        // TODO: Add encryption logic later
        // For now, this does nothing
    }
}
#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    theme: String,
    download_path: String,
    current_tab: i32,
    notification_settings: NotificationSettings,
    tabs: Vec<TabInfo>, // NEW FIELD
}
#[derive(Serialize, Deserialize, Clone)]
struct TabInfo {
    name: String,
    icon: String,
}
fn get_config_path() -> Option<PathBuf> {
    if let Some(mut path) = config_dir() {
        path.push("WhatsAppDesktop");
        let _ = fs::create_dir_all(&path);
        Some(path)
    } else {
        None
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tray_item::TrayItem;

    let should_show = Arc::new(Mutex::new(true));
    let should_show_clone = Arc::clone(&should_show);
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let icon_path = exe_dir.join("resources/icons/tray.png");

    thread::spawn(move || {
        let mut tray = TrayItem::new("WhatsApp", "&icon_path").unwrap();

        tray.add_label("WhatsApp Desktop").unwrap();

        let show_clone = Arc::clone(&should_show_clone);
        tray.add_menu_item("Show/Hide", move || {
            let mut visible = show_clone.lock().unwrap();
            *visible = !*visible;
            // you can implement toggling logic via global state later
            println!("Toggled window (you need to implement actual show/hide)");
        })
        .unwrap();

        tray.add_menu_item("Quit", || {
            println!("Quitting app from tray...");
            std::process::exit(0);
        })
        .unwrap();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    qml_register_type::<AppController>(cstr!("AppController"), 1, 0, cstr!("AppController"));
    let mut engine = QmlEngine::new();

    engine.set_property(
        "SidebarQML".into(),
        QVariant::from(QString::from(qml_resources::SIDEBAR_QML)),
    );
    engine.set_property(
        "WebTabQML".into(),
        QVariant::from(QString::from(qml_resources::WEBTAB_QML)),
    );
    engine.set_property(
        "SettingsDialogQML".into(),
        QVariant::from(QString::from(qml_resources::SETTINGS_DIALOG_QML)),
    );
    engine.set_property(
        "AppStyleQML".into(),
        QVariant::from(QString::from(qml_resources::APP_STYLE_QML)),
    );

    // Expose application directory path
    let app_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();
    engine.set_property(
        "applicationDirPath".into(),
        QVariant::from(QString::from(app_dir)),
    );

    // Load the main QML file (using include_str! approach)
    engine.load_file("qml/main.qml".into());
    engine.exec();
    Ok(())
}
