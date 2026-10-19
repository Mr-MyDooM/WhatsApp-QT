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
    /// `fresh_id` is a new UUID for the new account.
    AddAccount {
        fresh_id: String,
    },
    /// `fresh_id` is a new UUID, used only when the last account is removed.
    RemoveAccount {
        id: String,
        fresh_id: String,
    },
    SwitchAccount {
        id: String,
    },
    /// Zero-based position in the sidebar list.
    SwitchToIndex(usize),
    /// Next (+1) or previous (-1) account, wrapping around.
    SwitchRelative(i32),
    /// The sidebar page loaded and wants the current list.
    SidebarReady,
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    CreateAccountWebview {
        id: String,
    },
    CloseAccountWebview {
        id: String,
    },
    /// Delete the account's data store for good.
    DeleteAccountData {
        id: String,
    },
    ShowAccount {
        id: String,
    },
    RenderSidebar {
        accounts: Vec<Account>,
        active: String,
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
            Event::AddAccount { fresh_id } => self.add(fresh_id),
            Event::RemoveAccount { id, fresh_id } => self.remove(&id, fresh_id),
            Event::SwitchAccount { id } => self.switch(&id),
            Event::SwitchToIndex(i) => match self.settings.accounts.get(i) {
                Some(a) => {
                    let id = a.id.clone();
                    self.switch(&id)
                }
                None => Vec::new(),
            },
            Event::SwitchRelative(delta) => {
                let n = self.settings.accounts.len() as i32;
                if n == 0 {
                    return Vec::new();
                }
                let current = self.active_index() as i32;
                let id = self.settings.accounts[(current + delta).rem_euclid(n) as usize]
                    .id
                    .clone();
                self.switch(&id)
            }
            Event::SidebarReady => vec![self.render()],
        }
    }

    fn active_index(&self) -> usize {
        let active = self.settings.active_account.as_ref();
        self.settings
            .accounts
            .iter()
            .position(|a| Some(&a.id) == active)
            .unwrap_or(0)
    }

    fn render(&self) -> Effect {
        Effect::RenderSidebar {
            accounts: self.settings.accounts.clone(),
            active: self.settings.active_account.clone().unwrap_or_default(),
        }
    }

    fn save(&self, effects: &mut Vec<Effect>) {
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

    /// Appends an account with the first free default name and makes it active.
    fn new_account(&mut self, id: String) {
        let name = (1..)
            .map(|n| format!("Account {n}"))
            .find(|name| self.settings.accounts.iter().all(|a| &a.name != name))
            .expect("unbounded range");
        self.settings.active_account = Some(id.clone());
        self.settings.accounts.push(Account { id, name });
    }

    fn add(&mut self, fresh_id: String) -> Vec<Effect> {
        self.new_account(fresh_id.clone());
        let mut effects = vec![
            Effect::CreateAccountWebview {
                id: fresh_id.clone(),
            },
            Effect::ShowAccount { id: fresh_id },
            self.render(),
        ];
        self.save(&mut effects);
        effects
    }

    fn switch(&mut self, id: &str) -> Vec<Effect> {
        if !self.settings.accounts.iter().any(|a| a.id == id) {
            return Vec::new();
        }
        self.settings.active_account = Some(id.to_string());
        let mut effects = vec![Effect::ShowAccount { id: id.to_string() }, self.render()];
        self.save(&mut effects);
        effects
    }

    fn remove(&mut self, id: &str, fresh_id: String) -> Vec<Effect> {
        let Some(pos) = self.settings.accounts.iter().position(|a| a.id == id) else {
            return Vec::new();
        };
        let was_active = self.settings.active_account.as_deref() == Some(id);
        self.settings.accounts.remove(pos);
        let mut effects = vec![
            Effect::CloseAccountWebview { id: id.to_string() },
            Effect::DeleteAccountData { id: id.to_string() },
        ];
        if self.settings.accounts.is_empty() {
            self.new_account(fresh_id.clone());
            effects.push(Effect::CreateAccountWebview { id: fresh_id });
        } else if was_active {
            let next = self.settings.accounts[pos.min(self.settings.accounts.len() - 1)]
                .id
                .clone();
            self.settings.active_account = Some(next);
        }
        effects.push(Effect::ShowAccount {
            id: self.settings.active_account.clone().expect("set above"),
        });
        effects.push(self.render());
        self.save(&mut effects);
        effects
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
        effects.push(self.render());
        if changed {
            self.save(&mut effects);
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
                Effect::RenderSidebar {
                    accounts: vec![
                        Account {
                            id: "a".into(),
                            name: "A".into()
                        },
                        Account {
                            id: "b".into(),
                            name: "B".into()
                        },
                    ],
                    active: "b".into(),
                },
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

    // --- accounts ---

    fn started(raw: &str) -> Core {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some(raw.into())),
            fresh_id: "unused".into(),
        });
        core
    }

    const TWO: &str = r#"{"accounts":[{"id":"a","name":"Account 1"},{"id":"b","name":"Account 2"}],"active_account":"a"}"#;

    fn has_save(effects: &[Effect]) -> bool {
        effects
            .iter()
            .any(|e| matches!(e, Effect::SaveSettings { .. }))
    }

    #[test]
    fn add_makes_new_active_account_with_default_name_and_webview() {
        let mut core = started(TWO);
        let effects = core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        });
        assert!(effects.contains(&Effect::CreateAccountWebview { id: "c".into() }));
        assert!(effects.contains(&Effect::ShowAccount { id: "c".into() }));
        let out = saved(&effects);
        assert_eq!(out["accounts"][2]["id"], "c");
        assert_eq!(out["accounts"][2]["name"], "Account 3");
        assert_eq!(out["active_account"], "c");
    }

    #[test]
    fn default_name_skips_names_already_taken() {
        let mut core =
            started(r#"{"accounts":[{"id":"a","name":"Account 2"}],"active_account":"a"}"#);
        let out = saved(&core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        }));
        assert_eq!(out["accounts"][1]["name"], "Account 1");
    }

    #[test]
    fn switch_shows_only_that_account_and_saves() {
        let mut core = started(TWO);
        let effects = core.handle(Event::SwitchAccount { id: "b".into() });
        assert!(effects.contains(&Effect::ShowAccount { id: "b".into() }));
        assert_eq!(saved(&effects)["active_account"], "b");
    }

    #[test]
    fn switch_to_unknown_account_does_nothing() {
        let mut core = started(TWO);
        assert!(core
            .handle(Event::SwitchAccount { id: "zz".into() })
            .is_empty());
    }

    #[test]
    fn switch_by_position_and_wrapping_neighbour() {
        let mut core = started(TWO);
        let e = core.handle(Event::SwitchToIndex(1));
        assert!(e.contains(&Effect::ShowAccount { id: "b".into() }));
        assert!(core.handle(Event::SwitchToIndex(9)).is_empty());
        let e = core.handle(Event::SwitchRelative(1));
        assert!(e.contains(&Effect::ShowAccount { id: "a".into() }));
        let e = core.handle(Event::SwitchRelative(-1));
        assert!(e.contains(&Effect::ShowAccount { id: "b".into() }));
    }

    #[test]
    fn remove_inactive_deletes_its_store_and_keeps_active() {
        let mut core = started(TWO);
        let effects = core.handle(Event::RemoveAccount {
            id: "b".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.contains(&Effect::CloseAccountWebview { id: "b".into() }));
        assert!(effects.contains(&Effect::DeleteAccountData { id: "b".into() }));
        let out = saved(&effects);
        assert_eq!(out["accounts"].as_array().unwrap().len(), 1);
        assert_eq!(out["active_account"], "a");
    }

    #[test]
    fn remove_active_activates_another_account() {
        let mut core = started(TWO);
        let effects = core.handle(Event::RemoveAccount {
            id: "a".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.contains(&Effect::ShowAccount { id: "b".into() }));
        assert_eq!(saved(&effects)["active_account"], "b");
    }

    #[test]
    fn remove_last_creates_new_default_account() {
        let mut core =
            started(r#"{"accounts":[{"id":"a","name":"Account 1"}],"active_account":"a"}"#);
        let effects = core.handle(Event::RemoveAccount {
            id: "a".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.contains(&Effect::DeleteAccountData { id: "a".into() }));
        assert!(effects.contains(&Effect::CreateAccountWebview { id: "n".into() }));
        assert!(effects.contains(&Effect::ShowAccount { id: "n".into() }));
        let out = saved(&effects);
        assert_eq!(out["accounts"].as_array().unwrap().len(), 1);
        assert_eq!(out["accounts"][0]["id"], "n");
    }

    #[test]
    fn remove_unknown_account_does_nothing() {
        let mut core = started(TWO);
        let effects = core.handle(Event::RemoveAccount {
            id: "zz".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.is_empty());
    }

    #[test]
    fn removed_id_does_not_come_back() {
        let mut core = started(TWO);
        core.handle(Event::RemoveAccount {
            id: "b".into(),
            fresh_id: "n".into(),
        });
        let out = saved(&core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        }));
        let ids: Vec<_> = out["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, ["a", "c"]);
    }

    #[test]
    fn saved_accounts_round_trip_through_a_new_core() {
        let mut core = started(TWO);
        core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        });
        let json = saved(&core.handle(Event::SwitchAccount { id: "b".into() })).to_string();
        let effects = load(Some(&json), "unused");
        assert_eq!(
            effects,
            vec![
                Effect::CreateAccountWebview { id: "a".into() },
                Effect::CreateAccountWebview { id: "b".into() },
                Effect::CreateAccountWebview { id: "c".into() },
                Effect::ShowAccount { id: "b".into() },
                Effect::RenderSidebar {
                    accounts: vec![
                        Account {
                            id: "a".into(),
                            name: "Account 1".into()
                        },
                        Account {
                            id: "b".into(),
                            name: "Account 2".into()
                        },
                        Account {
                            id: "c".into(),
                            name: "Account 3".into()
                        },
                    ],
                    active: "b".into(),
                },
            ]
        );
    }

    #[test]
    fn no_save_after_blocked_load() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{bad".into())),
            fresh_id: "u1".into(),
        });
        let effects = core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        });
        assert!(!has_save(&effects));
        let effects = core.handle(Event::SwitchAccount { id: "u1".into() });
        assert!(!has_save(&effects));
        let effects = core.handle(Event::RemoveAccount {
            id: "c".into(),
            fresh_id: "n".into(),
        });
        assert!(!has_save(&effects));
    }

    #[test]
    fn sidebar_ready_renders_current_state() {
        let mut core = started(TWO);
        let effects = core.handle(Event::SidebarReady);
        assert_eq!(
            effects,
            vec![Effect::RenderSidebar {
                accounts: vec![
                    Account {
                        id: "a".into(),
                        name: "Account 1".into()
                    },
                    Account {
                        id: "b".into(),
                        name: "Account 2".into()
                    },
                ],
                active: "a".into(),
            }]
        );
    }
}
