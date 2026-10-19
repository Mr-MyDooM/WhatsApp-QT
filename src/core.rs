//! Pure app core: no Tauri types. One entry point, `Core::handle`: event in, effects out.

use serde::{Deserialize, Serialize};

const DEFAULT_ACCOUNT_NAME: &str = "Account 1";

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
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    CreateAccountWebview { id: String },
    ShowAccount { id: String },
    SaveSettings { json: String },
    ReportError { message: String },
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
}
