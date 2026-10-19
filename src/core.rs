//! Pure app core: no Tauri types. One entry point, `Core::handle`: event in, effects out.

use serde::{Deserialize, Serialize};

const DEFAULT_ACCOUNT_NAME: &str = "Account 1";
const MIN_ZOOM: f64 = 0.25;
const MAX_ZOOM: f64 = 3.0;

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

/// CSS to inject into account webviews. "system" and unknown names inject nothing.
pub fn theme_css(theme: &str) -> String {
    let mut css = String::new();
    if theme == "dark" {
        css.push_str(DARK_BASE_CSS);
        css.push_str(DARK_CHAT_CSS);
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

fn is_known_theme(theme: &str) -> bool {
    matches!(theme, "light" | "dark" | "system")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub show_message_notifications: bool,
    pub show_call_notifications: bool,
    pub sound_enabled: bool,
    pub show_sender: bool,
    /// Empty means the platform default sound.
    pub sound_path: String,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_message_notifications: true,
            show_call_notifications: true,
            sound_enabled: true,
            show_sender: true,
            sound_path: String::new(),
        }
    }
}

/// Unknown old fields (`tabs`, `current_tab`) are ignored on load and dropped on save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub download_path: String,
    pub notification_settings: NotificationSettings,
    pub autostart_enabled: bool,
    pub clear_cache_on_exit: bool,
    pub start_minimized: bool,
    pub close_to_tray: bool,
    pub zoom_factor: f64,
    pub spellcheck_enabled: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub accounts: Vec<Account>,
    pub active_account: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            download_path: String::new(),
            notification_settings: NotificationSettings::default(),
            autostart_enabled: false,
            clear_cache_on_exit: false,
            start_minimized: false,
            close_to_tray: false,
            zoom_factor: 1.0,
            spellcheck_enabled: true,
            window_width: 0,
            window_height: 0,
            window_x: 0,
            window_y: 0,
            accounts: Vec::new(),
            active_account: None,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    /// `raw` is the settings file content, `Ok(None)` when there is no file,
    /// `Err` when the file exists but cannot be read.
    /// `fresh_id` is a new UUID, used only when no account exists yet.
    SettingsLoaded {
        raw: Result<Option<String>, String>,
        fresh_id: String,
    },
    ThemeChanged {
        theme: String,
    },
    /// Absolute zoom level, clamped to `MIN_ZOOM..=MAX_ZOOM`.
    ZoomChanged {
        factor: f64,
    },
    /// Relative zoom, for the keyboard shortcuts.
    ZoomStepped {
        delta: f64,
    },
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    CreateAccountWebview {
        id: String,
    },
    ShowAccount {
        id: String,
    },
    /// `theme` is "light", "dark" or "system"; empty `css` removes any injected theme.
    ApplyTheme {
        theme: String,
        css: String,
    },
    ApplyZoom {
        factor: f64,
    },
    SaveSettings {
        json: String,
    },
    ReportError {
        message: String,
    },
}

#[derive(Default)]
pub struct Core {
    settings: Settings,
    /// Set when the settings file exists but could not be read or parsed.
    /// Saving would overwrite the user's file with defaults, so it is skipped.
    save_blocked: bool,
}

impl Core {
    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::SettingsLoaded { raw, fresh_id } => self.load(raw, fresh_id),
            Event::ThemeChanged { theme } => self.set_theme(theme),
            Event::ZoomChanged { factor } => self.set_zoom(factor),
            Event::ZoomStepped { delta } => self.set_zoom(self.settings.zoom_factor + delta),
        }
    }

    pub fn theme(&self) -> &str {
        &self.settings.theme
    }

    pub fn zoom_factor(&self) -> f64 {
        self.settings.zoom_factor
    }

    /// Effects that bring the webviews in line with the current settings.
    pub fn appearance_effects(&self) -> Vec<Effect> {
        vec![
            Effect::ApplyTheme {
                theme: self.settings.theme.clone(),
                css: theme_css(&self.settings.theme),
            },
            Effect::ApplyZoom {
                factor: self.settings.zoom_factor,
            },
        ]
    }

    fn set_theme(&mut self, theme: String) -> Vec<Effect> {
        if !is_known_theme(&theme) {
            return vec![Effect::ReportError {
                message: format!("Unknown theme: {theme}"),
            }];
        }
        self.settings.theme = theme.clone();
        let css = theme_css(&theme);
        let mut effects = vec![Effect::ApplyTheme { theme, css }];
        self.push_save(&mut effects);
        effects
    }

    fn set_zoom(&mut self, factor: f64) -> Vec<Effect> {
        if !factor.is_finite() {
            return vec![Effect::ReportError {
                message: format!("Invalid zoom level: {factor}"),
            }];
        }
        // Round to 2 decimals so repeated 0.1 steps do not drift (1.2000000000000002).
        let factor = ((factor * 100.0).round() / 100.0).clamp(MIN_ZOOM, MAX_ZOOM);
        self.settings.zoom_factor = factor;
        let mut effects = vec![Effect::ApplyZoom { factor }];
        self.push_save(&mut effects);
        effects
    }

    fn push_save(&self, effects: &mut Vec<Effect>) {
        if self.save_blocked {
            return;
        }
        match serde_json::to_string_pretty(&self.settings) {
            Ok(json) => effects.push(Effect::SaveSettings { json }),
            Err(e) => effects.push(Effect::ReportError {
                message: format!("Could not serialize settings: {e}"),
            }),
        }
    }

    fn load(&mut self, raw: Result<Option<String>, String>, fresh_id: String) -> Vec<Effect> {
        let mut effects = Vec::new();
        let parsed = raw.and_then(|r| {
            r.map(|text| {
                serde_json::from_str::<Settings>(&text)
                    .map_err(|e| format!("Settings file is malformed: {e}"))
            })
            .transpose()
        });
        self.settings = match parsed {
            Ok(s) => s.unwrap_or_default(),
            Err(message) => {
                effects.push(Effect::ReportError {
                    message: format!("{message}. Using defaults; the file is not changed."),
                });
                self.save_blocked = true;
                Settings::default()
            }
        };

        let s = &mut self.settings;
        let mut changed = false;
        let zoom = if s.zoom_factor.is_finite() {
            s.zoom_factor.clamp(MIN_ZOOM, MAX_ZOOM)
        } else {
            1.0
        };
        if zoom != s.zoom_factor {
            s.zoom_factor = zoom;
            changed = true;
        }
        if s.accounts.is_empty() {
            s.accounts.push(Account {
                id: fresh_id,
                name: DEFAULT_ACCOUNT_NAME.into(),
            });
            changed = true;
        }
        let active_is_valid = s
            .active_account
            .as_ref()
            .is_some_and(|a| s.accounts.iter().any(|acc| &acc.id == a));
        if !active_is_valid {
            s.active_account = Some(s.accounts[0].id.clone());
            changed = true;
        }

        effects.extend(
            s.accounts
                .iter()
                .map(|a| Effect::CreateAccountWebview { id: a.id.clone() }),
        );
        effects.push(Effect::ShowAccount {
            id: s.active_account.clone().expect("set above"),
        });
        if changed && !self.save_blocked {
            match serde_json::to_string_pretty(&*s) {
                Ok(json) => effects.push(Effect::SaveSettings { json }),
                Err(e) => effects.push(Effect::ReportError {
                    message: format!("Could not serialize settings: {e}"),
                }),
            }
        }
        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(raw: Option<&str>, fresh: &str) -> Vec<Effect> {
        Core::default().handle(Event::SettingsLoaded {
            raw: Ok(raw.map(String::from)),
            fresh_id: fresh.into(),
        })
    }

    fn saved(effects: &[Effect]) -> serde_json::Value {
        effects
            .iter()
            .find_map(|e| match e {
                Effect::SaveSettings { json } => Some(serde_json::from_str(json).unwrap()),
                _ => None,
            })
            .expect("a SaveSettings effect")
    }

    const LEGACY: &str = r#"{
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
        "tabs": [{"name": "Work", "icon": ""}]
    }"#;

    #[test]
    fn legacy_settings_load_with_safe_defaults_and_keep_old_values() {
        let out = saved(&load(Some(LEGACY), "u1"));
        assert_eq!(out["theme"], "dark");
        assert_eq!(
            out["notification_settings"]["show_message_notifications"],
            false
        );
        assert_eq!(out["notification_settings"]["sound_enabled"], false);
        // Missing fields: sound path empty, zoom 1.0 (not 0.0), spellcheck on, autostart off.
        assert_eq!(out["notification_settings"]["sound_path"], "");
        assert_eq!(out["zoom_factor"], 1.0);
        assert_eq!(out["spellcheck_enabled"], true);
        assert_eq!(out["autostart_enabled"], false);
        assert_eq!(out["clear_cache_on_exit"], false);
        assert_eq!(out["start_minimized"], false);
        assert_eq!(out["close_to_tray"], false);
        assert_eq!(
            (out["window_width"].as_i64(), out["window_height"].as_i64()),
            (Some(0), Some(0))
        );
        // Old tab fields are dropped.
        assert!(out.get("tabs").is_none() && out.get("current_tab").is_none());
    }

    #[test]
    fn first_start_makes_one_default_account_and_shows_it() {
        let effects = load(Some(LEGACY), "u1");
        assert_eq!(
            effects[..2],
            [
                Effect::CreateAccountWebview { id: "u1".into() },
                Effect::ShowAccount { id: "u1".into() },
            ]
        );
        let out = saved(&effects);
        assert_eq!(out["accounts"][0]["id"], "u1");
        assert_eq!(out["active_account"], "u1");
    }

    #[test]
    fn missing_file_gives_defaults_and_one_account() {
        let out = saved(&load(None, "u1"));
        assert_eq!(out["theme"], "system");
        assert_eq!(out["zoom_factor"], 1.0);
        assert_eq!(out["accounts"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn saved_accounts_are_restored_without_a_new_one() {
        let raw =
            r#"{"accounts":[{"id":"a","name":"A"},{"id":"b","name":"B"}],"active_account":"b"}"#;
        assert_eq!(
            load(Some(raw), "unused"),
            vec![
                Effect::CreateAccountWebview { id: "a".into() },
                Effect::CreateAccountWebview { id: "b".into() },
                Effect::ShowAccount { id: "b".into() },
            ]
        );
    }

    #[test]
    fn unknown_active_account_falls_back_to_first() {
        let raw = r#"{"accounts":[{"id":"a","name":"A"}],"active_account":"gone"}"#;
        let effects = load(Some(raw), "unused");
        assert!(effects.contains(&Effect::ShowAccount { id: "a".into() }));
    }

    #[test]
    fn malformed_file_reports_error_and_still_starts_with_defaults() {
        let effects = load(Some("{not json"), "u1");
        assert!(matches!(effects[0], Effect::ReportError { .. }));
        assert!(effects.contains(&Effect::ShowAccount { id: "u1".into() }));
    }

    #[test]
    fn malformed_file_is_not_overwritten() {
        let effects = load(Some("{not json"), "u1");
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::SaveSettings { .. })));
    }

    #[test]
    fn unreadable_file_reports_error_and_is_not_overwritten() {
        let effects = Core::default().handle(Event::SettingsLoaded {
            raw: Err("permission denied".into()),
            fresh_id: "u1".into(),
        });
        assert!(matches!(effects[0], Effect::ReportError { .. }));
        assert!(effects.contains(&Effect::ShowAccount { id: "u1".into() }));
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::SaveSettings { .. })));
    }

    fn loaded_core() -> Core {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(None),
            fresh_id: "u1".into(),
        });
        core
    }

    fn theme(core: &mut Core, theme: &str) -> Vec<Effect> {
        core.handle(Event::ThemeChanged {
            theme: theme.into(),
        })
    }

    fn zoom(core: &mut Core, factor: f64) -> Vec<Effect> {
        core.handle(Event::ZoomChanged { factor })
    }

    fn has_save(effects: &[Effect]) -> bool {
        effects
            .iter()
            .any(|e| matches!(e, Effect::SaveSettings { .. }))
    }

    #[test]
    fn theme_css_covers_each_theme() {
        let dark = theme_css("dark");
        assert!(dark.contains("#1e1e1e"), "dark theme must set its base bg");
        assert!(
            dark.contains("chatlist-header"),
            "dark adds WhatsApp overrides"
        );

        let light = theme_css("light");
        assert!(
            light.contains("#ffffff"),
            "light theme must set its base bg"
        );
        assert!(
            !light.contains("#1e1e1e"),
            "light must not leak dark colors"
        );

        // "system" (and anything unrecognised) means "inject nothing".
        assert!(theme_css("system").is_empty());
        assert!(theme_css("").is_empty());
    }

    #[test]
    fn theme_change_applies_and_saves() {
        let mut core = loaded_core();
        let effects = theme(&mut core, "dark");
        assert!(matches!(&effects[0], Effect::ApplyTheme { theme, css }
            if theme == "dark" && css.contains("#1e1e1e")));
        assert_eq!(saved(&effects)["theme"], "dark");
    }

    #[test]
    fn system_theme_applies_empty_css() {
        let mut core = loaded_core();
        theme(&mut core, "dark");
        assert_eq!(
            theme(&mut core, "system")[0],
            Effect::ApplyTheme {
                theme: "system".into(),
                css: String::new()
            }
        );
    }

    #[test]
    fn unknown_theme_is_rejected_without_saving() {
        let mut core = loaded_core();
        let effects = theme(&mut core, "neon");
        assert!(matches!(effects[..], [Effect::ReportError { .. }]));
        assert_eq!(core.theme(), "system");
    }

    #[test]
    fn zoom_change_applies_and_saves() {
        let mut core = loaded_core();
        let effects = zoom(&mut core, 1.5);
        assert_eq!(effects[0], Effect::ApplyZoom { factor: 1.5 });
        assert_eq!(saved(&effects)["zoom_factor"], 1.5);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut core = loaded_core();
        assert_eq!(zoom(&mut core, 9.0)[0], Effect::ApplyZoom { factor: 3.0 });
        assert_eq!(zoom(&mut core, 0.0)[0], Effect::ApplyZoom { factor: 0.25 });
    }

    #[test]
    fn non_finite_zoom_is_rejected() {
        let mut core = loaded_core();
        let effects = zoom(&mut core, f64::NAN);
        assert!(matches!(effects[..], [Effect::ReportError { .. }]));
        assert_eq!(core.zoom_factor(), 1.0);
    }

    #[test]
    fn zoom_steps_without_float_drift() {
        let mut core = loaded_core();
        for _ in 0..3 {
            core.handle(Event::ZoomStepped { delta: 0.1 });
        }
        assert_eq!(core.zoom_factor(), 1.3);
        core.handle(Event::ZoomStepped { delta: -0.1 });
        assert_eq!(core.zoom_factor(), 1.2);
    }

    #[test]
    fn changes_are_not_saved_while_save_is_blocked() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{not json".into())),
            fresh_id: "u1".into(),
        });
        let effects = theme(&mut core, "dark");
        assert!(matches!(effects[0], Effect::ApplyTheme { .. }));
        assert!(!has_save(&effects));
        assert!(!has_save(&zoom(&mut core, 2.0)));
    }

    #[test]
    fn out_of_range_saved_zoom_is_clamped_on_load() {
        let out = saved(&load(Some(r#"{"zoom_factor":0.0}"#), "u1"));
        assert_eq!(out["zoom_factor"], 0.25);
    }

    #[test]
    fn appearance_effects_reflect_loaded_settings() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some(r#"{"theme":"light","zoom_factor":1.2}"#.into())),
            fresh_id: "u1".into(),
        });
        let effects = core.appearance_effects();
        assert!(matches!(&effects[0], Effect::ApplyTheme { theme, .. } if theme == "light"));
        assert_eq!(effects[1], Effect::ApplyZoom { factor: 1.2 });
    }
}
