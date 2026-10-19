//! Pure app core: no Tauri types. One entry point, `Core::handle`: event in, effects out.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const DEFAULT_ACCOUNT_NAME: &str = "Account 1";
const MIN_ZOOM: f64 = 0.25;
const MAX_ZOOM: f64 = 3.0;
/// Caps for text from the web page, which is untrusted.
const MAX_ACCOUNT_NAME_CHARS: usize = 40;
/// A page that has not finished loading after this long counts as failed.
const LOAD_TIMEOUT_SECS: u64 = 30;
const RETRY_BASE_SECS: u64 = 5;
const RETRY_MAX_SECS: u64 = 300;
const MAX_TITLE_CHARS: usize = 100;
const MAX_BODY_CHARS: usize = 300;
const MAX_SOUND_PATH_CHARS: usize = 1024;
const HIDDEN_SENDER_BODY: &str = "New message";
const TEST_TITLE: &str = "Test";
const TEST_BODY: &str = "This is a test notification.";

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
    /// Message notifications are silenced for this account. Old files have no field: false.
    #[serde(default)]
    pub muted: bool,
}

/// One account line in the tray and Dock menus.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuAccount {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    /// Old Qt master switch. Kept only so the old app can still read the file. Ignored here:
    /// the new UI has no control for it, so only `show_message_notifications` decides.
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
    /// Sidebar shows names and counts (wide) or the avatar column only.
    pub sidebar_wide: bool,
    /// Sidebar hides at the window edge and shows over the page on mouse over.
    pub sidebar_auto_hide: bool,
    pub zoom_factor: f64,
    pub spellcheck_enabled: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub accounts: Vec<Account>,
    pub active_account: Option<String>,
    /// Snooze end time in Unix seconds. `None` means no Snooze.
    pub snooze_until: Option<u64>,
    /// Removed accounts whose data store is not deleted yet. Retried at each start.
    pub pending_deletions: Vec<String>,
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
            sidebar_wide: false,
            sidebar_auto_hide: false,
            zoom_factor: 1.0,
            spellcheck_enabled: true,
            window_width: 0,
            window_height: 0,
            window_x: 0,
            window_y: 0,
            accounts: Vec::new(),
            active_account: None,
            snooze_until: None,
            pending_deletions: Vec::new(),
        }
    }
}

/// Why a message notification is not shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationBlock {
    Off,
    Snoozed,
    Muted,
}

/// What a notification plays.
#[derive(Debug, Clone, PartialEq)]
pub enum Sound {
    Off,
    /// The platform default sound.
    Default,
    File(String),
}

/// Wait before automatic retry number `failures` (1 for the first): doubles, up to a maximum.
fn retry_delay_secs(failures: u32) -> u64 {
    RETRY_BASE_SECS
        .saturating_mul(1u64 << failures.saturating_sub(1).min(16))
        .min(RETRY_MAX_SECS)
}

fn cut(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
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
    /// Title and body come from the web page: untrusted.
    /// `now` is the current time in Unix seconds, to check Snooze.
    NotificationRequested {
        account_id: String,
        title: String,
        body: String,
        now: u64,
    },
    /// The "Send Test Notification" button. Same rules as a message for the active account.
    TestNotificationRequested {
        now: u64,
    },
    /// The user clicked a message notification. An unknown or removed account just shows
    /// the window.
    NotificationClicked {
        account_id: String,
    },
    NotificationSettingsChanged {
        messages: bool,
        show_sender: bool,
        sound_enabled: bool,
        sound_path: String,
    },
    /// The shortcut: message notifications on or off.
    NotificationsToggled,
    AutostartChanged {
        enabled: bool,
    },
    /// The system refused an `AutostartChanged`. `enabled` is the state that still holds.
    AutostartFailed {
        enabled: bool,
    },
    ClearCacheOnExitChanged {
        enabled: bool,
    },
    /// The user asked to quit. Runs before the app exits.
    QuitRequested,
    /// The user closed the main window (red button).
    CloseRequested,
    /// Tray icon click or tray menu Show/Hide.
    ToggleWindow {
        window_visible: bool,
    },
    CloseToTrayChanged {
        enabled: bool,
    },
    StartMinimizedChanged {
        enabled: bool,
    },
    /// Toggle button or View menu item: go from the avatar column to the wide sidebar, to
    /// auto-hide, and back to the avatar column.
    SidebarToggled,
    SidebarAutoHideChanged {
        enabled: bool,
    },
    /// Snooze message notifications for all accounts. `now` is Unix seconds.
    SnoozeSet {
        minutes: u64,
        now: u64,
    },
    /// End Snooze early.
    SnoozeCleared,
    /// A URL the OS handed to the app. Untrusted: the core accepts only
    /// `whatsapp:` and `https://wa.me/` forms.
    DeepLinkOpened {
        url: String,
    },
    /// The shell finished deleting the data of a removed account.
    AccountDataDeleted {
        id: String,
        result: Result<(), String>,
    },
    /// The user renamed an account. The name is trimmed and cut; a blank name is refused.
    RenameAccount {
        id: String,
        name: String,
    },
    /// Mute the account if it is unmuted, or unmute it. Unknown accounts are ignored.
    MuteToggled {
        id: String,
    },
    /// The shell started to load an account's page for the first time.
    AccountLoadBegan {
        id: String,
    },
    /// An account's page finished loading.
    PageLoaded {
        id: String,
    },
    /// The load timer of `attempt` ran out.
    LoadCheckDue {
        id: String,
        attempt: u64,
    },
    /// The retry timer set after failed `attempt` ran out.
    RetryDue {
        id: String,
        attempt: u64,
    },
    /// Reload menu item, Cmd+R, or the Retry button. Never touches the account's data.
    ReloadRequested {
        id: String,
    },
    /// An account webview's page title changed. Untrusted: WhatsApp Web sets it.
    TitleChanged {
        id: String,
        title: String,
    },
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
    /// `theme` is "light", "dark" or "system"; empty `css` removes any injected theme.
    ApplyTheme {
        theme: String,
        css: String,
    },
    ApplyZoom {
        factor: f64,
    },
    /// Idempotent. If the system refuses, the shell sends `Event::AutostartFailed`.
    SetAutostart {
        enabled: bool,
    },
    /// Remove cached files only. Logins must stay.
    ClearAccountCaches {
        ids: Vec<String>,
    },
    /// The sidebar width changed: lay out the webviews and tell the sidebar page.
    ApplySidebar,
    RenderSidebar {
        accounts: Vec<Account>,
        active: String,
    },
    ShowNotification {
        account_id: String,
        title: String,
        body: String,
        sound: Sound,
    },
    SaveSettings {
        json: String,
    },
    ReportError {
        message: String,
    },
    ShowWindow,
    HideWindow,
    Quit,
    /// Raise the window and load `url` (always on web.whatsapp.com) in the account.
    OpenChat {
        id: String,
        url: String,
    },
    /// Load the account's WhatsApp Web page again. The data store stays as it is.
    ReloadAccount {
        id: String,
    },
    ScheduleLoadCheck {
        id: String,
        attempt: u64,
        secs: u64,
    },
    ScheduleRetry {
        id: String,
        attempt: u64,
        secs: u64,
    },
    ShowLoadError {
        id: String,
    },
    HideLoadError {
        id: String,
    },
    /// Unread count of each account that has unread chats, and the total over all accounts.
    ShowUnread {
        counts: BTreeMap<String, u32>,
        total: u32,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum LoadStatus {
    #[default]
    Loading,
    Loaded,
    Failed,
}

/// Load progress of one account's page. Not saved.
#[derive(Debug, Default)]
struct LoadState {
    /// Grows with each load try, so a timer from an older try can be told apart.
    attempt: u64,
    status: LoadStatus,
    /// Failed tries in a row; sets the retry delay.
    failures: u32,
}

#[derive(Default)]
pub struct Core {
    loads: BTreeMap<String, LoadState>,
    settings: Settings,
    /// Set when the settings file exists but could not be read or parsed.
    /// Saving would overwrite the user's file with defaults, so it is skipped.
    save_blocked: bool,
    /// The message of the last load error, for the settings page.
    load_error: Option<String>,
    /// Unread count per account UUID; accounts with none are absent. Not saved.
    unread: BTreeMap<String, u32>,
}

impl Core {
    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::SettingsLoaded { raw, fresh_id } => self.load(raw, fresh_id),
            Event::AddAccount { fresh_id } => self.add(fresh_id),
            Event::RemoveAccount { id, fresh_id } => self.remove(&id, fresh_id),
            Event::RenameAccount { id, name } => self.rename(&id, &name),
            Event::MuteToggled { id } => self.toggle_mute(&id),
            Event::AccountLoadBegan { id } => self.begin_load(id, false),
            Event::PageLoaded { id } => self.page_loaded(&id),
            Event::LoadCheckDue { id, attempt } => self.load_check_due(id, attempt),
            Event::RetryDue { id, attempt } => match self.loads.get(&id) {
                Some(l) if l.status == LoadStatus::Failed && l.attempt == attempt => {
                    self.begin_load(id, true)
                }
                _ => Vec::new(),
            },
            Event::ReloadRequested { id } => self.begin_load(id, true),
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
            // A fresh sidebar shows no counts, so send them only when there are some.
            Event::SidebarReady if self.unread.is_empty() => vec![self.render()],
            Event::SidebarReady => vec![self.render(), self.show_unread()],
            Event::ThemeChanged { theme } => self.set_theme(theme),
            Event::ZoomChanged { factor } => self.set_zoom(factor),
            Event::ZoomStepped { delta } => self.set_zoom(self.settings.zoom_factor + delta),
            Event::NotificationRequested {
                account_id,
                title,
                body,
                now,
            } => self.notify(&account_id, &title, &body, now),
            Event::TestNotificationRequested { now } => match self.active_account_id() {
                Some(id) => self.notify(id, TEST_TITLE, TEST_BODY, now),
                None => Vec::new(),
            },
            Event::NotificationClicked { account_id } => {
                let mut effects = vec![Effect::ShowWindow];
                effects.extend(self.switch(&account_id));
                effects
            }
            Event::NotificationSettingsChanged {
                messages,
                show_sender,
                sound_enabled,
                sound_path,
            } => {
                if sound_path.chars().count() > MAX_SOUND_PATH_CHARS {
                    return vec![Effect::ReportError {
                        message: "Sound file path is too long".into(),
                    }];
                }
                let n = &mut self.settings.notification_settings;
                n.show_message_notifications = messages;
                n.show_sender = show_sender;
                n.sound_enabled = sound_enabled;
                n.sound_path = sound_path;
                self.saved_effects()
            }
            Event::NotificationsToggled => {
                let n = &mut self.settings.notification_settings;
                n.show_message_notifications = !n.show_message_notifications;
                self.saved_effects()
            }
            // The OS login item must not drift from a settings file we cannot write.
            Event::AutostartChanged { .. } if self.save_blocked => vec![Effect::ReportError {
                message: "Cannot change start at login while the settings file is unusable".into(),
            }],
            Event::AutostartChanged { enabled } => {
                self.settings.autostart_enabled = enabled;
                let mut effects = Vec::new();
                self.save(&mut effects);
                // Last: if the system refuses, the revert save must come after this one.
                effects.push(Effect::SetAutostart { enabled });
                effects
            }
            Event::AutostartFailed { enabled } => {
                self.settings.autostart_enabled = enabled;
                let mut effects = Vec::new();
                self.save(&mut effects);
                effects
            }
            Event::ClearCacheOnExitChanged { enabled } => {
                self.settings.clear_cache_on_exit = enabled;
                let mut effects = Vec::new();
                self.save(&mut effects);
                effects
            }
            Event::QuitRequested if self.settings.clear_cache_on_exit => {
                vec![Effect::ClearAccountCaches {
                    ids: self
                        .settings
                        .accounts
                        .iter()
                        .map(|a| a.id.clone())
                        .collect(),
                }]
            }
            Event::QuitRequested => Vec::new(),
            Event::CloseRequested => vec![if self.settings.close_to_tray {
                Effect::HideWindow
            } else {
                Effect::Quit
            }],
            Event::ToggleWindow { window_visible } => vec![if window_visible {
                Effect::HideWindow
            } else {
                Effect::ShowWindow
            }],
            Event::CloseToTrayChanged { enabled } => {
                self.settings.close_to_tray = enabled;
                self.saved_effects()
            }
            Event::StartMinimizedChanged { enabled } => {
                self.settings.start_minimized = enabled;
                self.saved_effects()
            }
            Event::SidebarToggled => {
                let s = &mut self.settings;
                (s.sidebar_wide, s.sidebar_auto_hide) = match (s.sidebar_wide, s.sidebar_auto_hide)
                {
                    (_, true) => (false, false),
                    (false, false) => (true, false),
                    (true, false) => (false, true),
                };
                let mut effects = vec![Effect::ApplySidebar];
                self.save(&mut effects);
                effects
            }
            Event::SidebarAutoHideChanged { enabled } => {
                self.settings.sidebar_auto_hide = enabled;
                let mut effects = vec![Effect::ApplySidebar];
                self.save(&mut effects);
                effects
            }
            Event::DeepLinkOpened { url } => self.open_deep_link(&url),
            Event::TitleChanged { id, title } => self.set_unread(id, unread_count(&title)),
            Event::AccountDataDeleted { id, result } => match result {
                Ok(()) => {
                    self.settings.pending_deletions.retain(|p| *p != id);
                    self.saved_effects()
                }
                // The id stays listed, so the next start tries again.
                Err(e) => vec![Effect::ReportError {
                    message: format!("Could not delete the data of a removed account: {e}"),
                }],
            },
            Event::SnoozeSet { minutes, now } => {
                self.settings.snooze_until = Some(now.saturating_add(minutes.saturating_mul(60)));
                self.saved_effects()
            }
            Event::SnoozeCleared => {
                self.settings.snooze_until = None;
                self.saved_effects()
            }
        }
    }

    /// Why the settings file was not used, if it was not. Saving is off in that case.
    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn is_muted(&self, id: &str) -> bool {
        self.settings.accounts.iter().any(|a| a.id == id && a.muted)
    }

    pub fn account_name(&self, id: &str) -> Option<&str> {
        self.settings
            .accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.as_str())
    }

    /// The id of the account the user sees now. Always set after load.
    pub fn active_account_id(&self) -> Option<&str> {
        self.settings
            .accounts
            .get(self.active_index())
            .map(|a| a.id.as_str())
    }

    /// One entry per account for the tray and Dock menus, in sidebar order.
    /// The label is the account name, plus the unread count when it is above 0.
    pub fn menu_accounts(&self) -> Vec<MenuAccount> {
        self.settings
            .accounts
            .iter()
            .map(|a| MenuAccount {
                id: a.id.clone(),
                label: match self.unread.get(&a.id) {
                    Some(n) if *n > 0 => format!("{} ({n})", a.name),
                    _ => a.name.clone(),
                },
            })
            .collect()
    }

    pub fn notification_settings(&self) -> &NotificationSettings {
        &self.settings.notification_settings
    }

    fn notify(&self, account_id: &str, title: &str, body: &str, now: u64) -> Vec<Effect> {
        let n = &self.settings.notification_settings;
        let Some(account) = self.settings.accounts.iter().find(|a| a.id == account_id) else {
            return Vec::new();
        };
        if self.block_for(account, now).is_some() {
            return Vec::new();
        }
        let body = if n.show_sender {
            format!(
                "{}: {}",
                cut(title, MAX_TITLE_CHARS),
                cut(body, MAX_BODY_CHARS)
            )
        } else {
            HIDDEN_SENDER_BODY.to_string()
        };
        let sound = match (n.sound_enabled, n.sound_path.is_empty()) {
            (false, _) => Sound::Off,
            (true, true) => Sound::Default,
            (true, false) => Sound::File(n.sound_path.clone()),
        };
        vec![Effect::ShowNotification {
            account_id: account.id.clone(),
            title: account.name.clone(),
            body,
            sound,
        }]
    }

    /// Why the active account gets no message notification now, if it gets none.
    pub fn notification_block(&self, now: u64) -> Option<NotificationBlock> {
        let id = self.active_account_id()?;
        let account = self.settings.accounts.iter().find(|a| a.id == id)?;
        self.block_for(account, now)
    }

    fn block_for(&self, account: &Account, now: u64) -> Option<NotificationBlock> {
        if !self
            .settings
            .notification_settings
            .show_message_notifications
        {
            Some(NotificationBlock::Off)
        } else if self.active_snooze_end(now).is_some() {
            Some(NotificationBlock::Snoozed)
        } else if account.muted {
            Some(NotificationBlock::Muted)
        } else {
            None
        }
    }

    /// End of the Snooze in Unix seconds, or `None` when it is off or already over.
    pub fn active_snooze_end(&self, now: u64) -> Option<u64> {
        self.settings.snooze_until.filter(|&end| now < end)
    }

    pub fn accounts(&self) -> &[Account] {
        &self.settings.accounts
    }

    pub fn autostart_enabled(&self) -> bool {
        self.settings.autostart_enabled
    }

    pub fn clear_cache_on_exit(&self) -> bool {
        self.settings.clear_cache_on_exit
    }

    pub fn theme(&self) -> &str {
        &self.settings.theme
    }

    pub fn zoom_factor(&self) -> f64 {
        self.settings.zoom_factor
    }

    pub fn close_to_tray(&self) -> bool {
        self.settings.close_to_tray
    }

    pub fn sidebar_wide(&self) -> bool {
        self.settings.sidebar_wide
    }

    pub fn sidebar_auto_hide(&self) -> bool {
        self.settings.sidebar_auto_hide
    }

    pub fn start_minimized(&self) -> bool {
        self.settings.start_minimized
    }

    /// The main window is built hidden; show it unless the user wants to start in the tray.
    pub fn start_window_effects(&self) -> Vec<Effect> {
        if self.settings.start_minimized {
            Vec::new()
        } else {
            vec![Effect::ShowWindow]
        }
    }

    fn saved_effects(&self) -> Vec<Effect> {
        let mut effects = Vec::new();
        self.save(&mut effects);
        effects
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
        self.save(&mut effects);
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
        self.save(&mut effects);
        effects
    }

    /// Opens the chat in the active account. Never adds or removes an account.
    fn open_deep_link(&self, url: &str) -> Vec<Effect> {
        let Some(account) = self.settings.accounts.get(self.active_index()) else {
            return Vec::new();
        };
        match chat_url(url) {
            Some(url) => vec![Effect::OpenChat {
                id: account.id.clone(),
                url,
            }],
            None => Vec::new(),
        }
    }

    fn set_unread(&mut self, id: String, count: u32) -> Vec<Effect> {
        let old = self.unread.get(&id).copied().unwrap_or(0);
        if count == old || !self.settings.accounts.iter().any(|a| a.id == id) {
            return Vec::new();
        }
        if count == 0 {
            self.unread.remove(&id);
        } else {
            self.unread.insert(id, count);
        }
        vec![self.show_unread()]
    }

    fn show_unread(&self) -> Effect {
        Effect::ShowUnread {
            counts: self.unread.clone(),
            total: self
                .unread
                .values()
                .fold(0u32, |sum, n| sum.saturating_add(*n)),
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
        self.settings.accounts.push(Account {
            id,
            name,
            muted: false,
        });
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

    fn rename(&mut self, id: &str, name: &str) -> Vec<Effect> {
        let name = cut(name.trim(), MAX_ACCOUNT_NAME_CHARS);
        let name = name.trim_end();
        let Some(account) = self.settings.accounts.iter_mut().find(|a| a.id == id) else {
            return Vec::new();
        };
        if name.is_empty() {
            return Vec::new();
        }
        account.name = name.to_string();
        let mut effects = vec![self.render()];
        self.save(&mut effects);
        effects
    }

    fn toggle_mute(&mut self, id: &str) -> Vec<Effect> {
        let Some(account) = self.settings.accounts.iter_mut().find(|a| a.id == id) else {
            return Vec::new();
        };
        account.muted = !account.muted;
        let mut effects = vec![self.render()];
        self.save(&mut effects);
        effects
    }

    /// Starts a load try for a known account. `reload` also tells the shell to load the page.
    fn begin_load(&mut self, id: String, reload: bool) -> Vec<Effect> {
        if !self.settings.accounts.iter().any(|a| a.id == id) {
            return Vec::new();
        }
        let load = self.loads.entry(id.clone()).or_default();
        load.attempt += 1;
        load.status = LoadStatus::Loading;
        let mut effects = Vec::new();
        if reload {
            effects.push(Effect::ReloadAccount { id: id.clone() });
        }
        effects.push(Effect::ScheduleLoadCheck {
            id,
            attempt: load.attempt,
            secs: LOAD_TIMEOUT_SECS,
        });
        effects
    }

    fn page_loaded(&mut self, id: &str) -> Vec<Effect> {
        let Some(load) = self.loads.get_mut(id) else {
            return Vec::new();
        };
        let was_failed = load.status == LoadStatus::Failed;
        load.status = LoadStatus::Loaded;
        load.failures = 0;
        if was_failed {
            vec![Effect::HideLoadError { id: id.to_string() }]
        } else {
            Vec::new()
        }
    }

    fn load_check_due(&mut self, id: String, attempt: u64) -> Vec<Effect> {
        let Some(load) = self.loads.get_mut(&id) else {
            return Vec::new();
        };
        if load.status != LoadStatus::Loading || load.attempt != attempt {
            return Vec::new();
        }
        load.status = LoadStatus::Failed;
        load.failures = load.failures.saturating_add(1);
        let secs = retry_delay_secs(load.failures);
        vec![
            Effect::ShowLoadError { id: id.clone() },
            Effect::ScheduleRetry { id, attempt, secs },
        ]
    }

    fn remove(&mut self, id: &str, fresh_id: String) -> Vec<Effect> {
        let Some(pos) = self.settings.accounts.iter().position(|a| a.id == id) else {
            return Vec::new();
        };
        let was_active = self.settings.active_account.as_deref() == Some(id);
        self.settings.accounts.remove(pos);
        self.settings.pending_deletions.push(id.to_string());
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
        if self
            .loads
            .remove(id)
            .is_some_and(|l| l.status == LoadStatus::Failed)
        {
            effects.push(Effect::HideLoadError { id: id.to_string() });
        }
        if self.unread.remove(id).is_some() {
            effects.push(self.show_unread());
        }
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
                let message = format!("{message}. Using defaults; the file is not changed.");
                self.load_error = Some(message.clone());
                effects.push(Effect::ReportError { message });
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
                muted: false,
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

        // Before any webview exists, so a leftover store is not in use.
        effects.extend(
            s.pending_deletions
                .iter()
                .map(|id| Effect::DeleteAccountData { id: id.clone() }),
        );
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

/// True for the one origin an account webview may show. Everything else opens in the browser.
pub fn is_account_origin(url: &url::Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com") && url.port().is_none()
}

/// True for the WhatsApp hosts that WhatsApp Web loads in its own frames (flows.whatsapp.net,
/// webtp.whatsapp.net). The navigation check also runs for frames, so these must load in place.
pub fn is_whatsapp_frame_origin(url: &url::Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url.host_str().is_some_and(|h| h.ends_with(".whatsapp.net"))
}

/// Avatar letters: one word gives one letter, more words give the first letters of the first two.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

/// Avatar colour hue (0-359), the same for the same account id on every run.
/// FNV-1a, because the std hasher gives no stable result across Rust versions.
pub fn avatar_hue(id: &str) -> u32 {
    let hash = id.bytes().fold(0x811c_9dc5_u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    });
    hash % 360
}

/// WhatsApp Web puts the unread count first in the page title: "(3) WhatsApp".
/// Any other title, or a number that does not fit, counts as zero.
fn unread_count(title: &str) -> u32 {
    title
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .and_then(|(n, _)| n.parse().ok())
        .unwrap_or(0)
}

/// Rebuilds a deep link as a web.whatsapp.com chat URL, or `None` for anything that is
/// not a `whatsapp://send` or `https://wa.me/<phone>` link with a digits-only phone.
/// The result is built from the parsed parts, never from the input string, so the
/// origin is always web.whatsapp.com.
fn chat_url(raw: &str) -> Option<String> {
    let link = url::Url::parse(raw).ok()?;
    let phone = match (link.scheme(), link.host_str()) {
        ("whatsapp", Some("send")) => link
            .query_pairs()
            .find(|(k, _)| k == "phone")
            .map(|(_, v)| v.into_owned())?,
        ("https", Some("wa.me")) if link.username().is_empty() => {
            link.path().trim_start_matches('/').to_string()
        }
        _ => return None,
    };
    let phone = phone.strip_prefix('+').unwrap_or(&phone);
    if phone.is_empty() || !phone.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut chat = url::Url::parse("https://web.whatsapp.com/send").expect("constant URL");
    chat.query_pairs_mut().append_pair("phone", phone);
    if let Some((_, text)) = link.query_pairs().find(|(k, _)| k == "text") {
        chat.query_pairs_mut().append_pair("text", &text);
    }
    Some(chat.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_one_letter_for_one_word_two_for_more() {
        assert_eq!(initials("work"), "W");
        assert_eq!(initials("My Work Phone"), "MW");
        assert_eq!(initials("  "), "");
    }

    #[test]
    fn avatar_hue_is_stable_and_in_range() {
        let id = "3f2b8c1e-0000-4000-8000-000000000001";
        assert_eq!(avatar_hue(id), avatar_hue(id));
        assert_eq!(avatar_hue("a"), 0xe40c_292c_u32 % 360);
        assert!(avatar_hue(id) < 360);
        assert_ne!(
            avatar_hue(id),
            avatar_hue("3f2b8c1e-0000-4000-8000-000000000002")
        );
    }

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
        // Missing sidebar field: avatar column.
        assert_eq!(out["sidebar_wide"], false);
        assert_eq!(out["sidebar_auto_hide"], false);
        assert_eq!(
            (out["window_width"].as_i64(), out["window_height"].as_i64()),
            (Some(0), Some(0))
        );
        // Old tab fields are dropped.
        assert!(out.get("tabs").is_none() && out.get("current_tab").is_none());
    }

    #[test]
    fn sidebar_toggle_goes_narrow_wide_auto_hide_narrow_and_saves() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(None),
            fresh_id: "u1".into(),
        });
        assert!(!core.sidebar_wide() && !core.sidebar_auto_hide());
        let effects = core.handle(Event::SidebarToggled);
        assert!(core.sidebar_wide() && !core.sidebar_auto_hide());
        assert_eq!(effects[0], Effect::ApplySidebar);
        assert_eq!(saved(&effects)["sidebar_wide"], true);
        let effects = core.handle(Event::SidebarToggled);
        assert!(!core.sidebar_wide() && core.sidebar_auto_hide());
        assert_eq!(saved(&effects)["sidebar_auto_hide"], true);
        let effects = core.handle(Event::SidebarToggled);
        assert!(!core.sidebar_wide() && !core.sidebar_auto_hide());
        assert_eq!(saved(&effects)["sidebar_auto_hide"], false);
    }

    #[test]
    fn auto_hide_switch_keeps_the_width_and_saves() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some(r#"{"sidebar_wide": true}"#.into())),
            fresh_id: "u1".into(),
        });
        let effects = core.handle(Event::SidebarAutoHideChanged { enabled: true });
        assert!(core.sidebar_wide() && core.sidebar_auto_hide());
        assert_eq!(effects[0], Effect::ApplySidebar);
        assert_eq!(saved(&effects)["sidebar_auto_hide"], true);
        // The button pins an auto-hidden sidebar as the avatar column.
        core.handle(Event::SidebarToggled);
        assert!(!core.sidebar_wide() && !core.sidebar_auto_hide());
    }

    #[test]
    fn saved_wide_sidebar_loads_wide() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some(r#"{"sidebar_wide": true}"#.into())),
            fresh_id: "u1".into(),
        });
        assert!(core.sidebar_wide());
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
                            name: "A".into(),
                            muted: false
                        },
                        Account {
                            id: "b".into(),
                            name: "B".into(),
                            muted: false
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
    fn notification_click_shows_window_and_switches_account() {
        let mut core = started(TWO);
        let effects = core.handle(Event::NotificationClicked {
            account_id: "b".into(),
        });
        assert!(effects.contains(&Effect::ShowWindow));
        assert!(effects.contains(&Effect::ShowAccount { id: "b".into() }));
        assert_eq!(saved(&effects)["active_account"], "b");
    }

    #[test]
    fn notification_click_for_unknown_account_only_shows_window() {
        let mut core = started(TWO);
        let effects = core.handle(Event::NotificationClicked {
            account_id: "zz".into(),
        });
        assert_eq!(effects, vec![Effect::ShowWindow]);
    }

    fn menu(core: &Core) -> Vec<(String, String)> {
        core.menu_accounts()
            .into_iter()
            .map(|m| (m.id, m.label))
            .collect()
    }

    #[test]
    fn menu_lists_accounts_in_order_with_unread_counts_above_zero() {
        let mut core = started(TWO);
        assert_eq!(
            menu(&core),
            vec![
                ("a".to_string(), "Account 1".to_string()),
                ("b".to_string(), "Account 2".to_string())
            ]
        );
        title(&mut core, "b", "(4) WhatsApp");
        assert_eq!(menu(&core)[0].1, "Account 1");
        assert_eq!(menu(&core)[1].1, "Account 2 (4)");
        title(&mut core, "b", "WhatsApp");
        assert_eq!(menu(&core)[1].1, "Account 2");
    }

    #[test]
    fn menu_follows_rename_add_and_remove() {
        let mut core = started(TWO);
        core.handle(Event::RenameAccount {
            id: "a".into(),
            name: "Work".into(),
        });
        assert_eq!(menu(&core)[0].1, "Work");
        core.handle(Event::AddAccount {
            fresh_id: "c".into(),
        });
        assert_eq!(menu(&core).len(), 3);
        core.handle(Event::RemoveAccount {
            id: "a".into(),
            fresh_id: "d".into(),
        });
        let ids: Vec<_> = menu(&core).into_iter().map(|m| m.0).collect();
        assert_eq!(ids, ["b", "c"]);
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
                            name: "Account 1".into(),
                            muted: false
                        },
                        Account {
                            id: "b".into(),
                            name: "Account 2".into(),
                            muted: false
                        },
                        Account {
                            id: "c".into(),
                            name: "Account 3".into(),
                            muted: false
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
                        name: "Account 1".into(),
                        muted: false
                    },
                    Account {
                        id: "b".into(),
                        name: "Account 2".into(),
                        muted: false
                    },
                ],
                active: "a".into(),
            }]
        );
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

    // --- message notifications ---

    fn notify(core: &mut Core, id: &str, title: &str, body: &str) -> Vec<Effect> {
        core.handle(Event::NotificationRequested {
            account_id: id.into(),
            title: title.into(),
            body: body.into(),
            now: 0,
        })
    }

    fn shown(effects: &[Effect]) -> Option<(&str, &str, &Sound)> {
        effects.iter().find_map(|e| match e {
            Effect::ShowNotification {
                title, body, sound, ..
            } => Some((title.as_str(), body.as_str(), sound)),
            _ => None,
        })
    }

    #[test]
    fn notification_names_the_account_and_shows_sender_and_text() {
        let mut core = started(TWO);
        let effects = notify(&mut core, "b", "Alice", "hi there");
        assert_eq!(
            effects,
            vec![Effect::ShowNotification {
                account_id: "b".into(),
                title: "Account 2".into(),
                body: "Alice: hi there".into(),
                sound: Sound::Default,
            }]
        );
    }

    #[test]
    fn hidden_sender_hides_sender_and_text() {
        let mut core = started(TWO);
        core.handle(Event::NotificationSettingsChanged {
            messages: true,
            show_sender: false,
            sound_enabled: true,
            sound_path: String::new(),
        });
        let effects = notify(&mut core, "a", "Alice", "secret");
        let (title, body, _) = shown(&effects).unwrap();
        assert_eq!((title, body), ("Account 1", "New message"));
    }

    #[test]
    fn nothing_is_shown_when_message_notifications_are_off() {
        let mut core = started(TWO);
        core.handle(Event::NotificationsToggled);
        assert!(notify(&mut core, "a", "Alice", "hi").is_empty());
        core.handle(Event::NotificationsToggled);
        assert!(!notify(&mut core, "a", "Alice", "hi").is_empty());
    }

    #[test]
    fn unknown_account_gets_no_notification() {
        let mut core = started(TWO);
        assert!(notify(&mut core, "zz", "Alice", "hi").is_empty());
    }

    #[test]
    fn long_title_and_body_are_cut() {
        let mut core = started(TWO);
        let long = "x".repeat(10_000);
        let effects = notify(&mut core, "a", &long, &long);
        let (_, body, _) = shown(&effects).unwrap();
        assert_eq!(
            body.chars().count(),
            MAX_TITLE_CHARS + ": ".len() + MAX_BODY_CHARS
        );
    }

    #[test]
    fn close_hides_to_tray_when_enabled_and_quits_when_not() {
        let mut core = loaded_core();
        assert_eq!(core.handle(Event::CloseRequested), vec![Effect::Quit]);
        core.handle(Event::CloseToTrayChanged { enabled: true });
        assert_eq!(core.handle(Event::CloseRequested), vec![Effect::HideWindow]);
    }

    #[test]
    fn tray_click_toggles_window_visibility() {
        let mut core = loaded_core();
        assert_eq!(
            core.handle(Event::ToggleWindow {
                window_visible: true
            }),
            vec![Effect::HideWindow]
        );
        assert_eq!(
            core.handle(Event::ToggleWindow {
                window_visible: false
            }),
            vec![Effect::ShowWindow]
        );
    }

    #[test]
    fn cutting_never_splits_a_multibyte_character() {
        let mut core = started(TWO);
        let long = "\u{1F600}".repeat(MAX_BODY_CHARS + 5);
        let effects = notify(&mut core, "a", "A", &long);
        let (_, body, _) = shown(&effects).unwrap();
        assert_eq!(body.chars().count(), "A: ".len() + MAX_BODY_CHARS);
    }

    #[test]
    fn sound_follows_settings() {
        let mut core = started(TWO);
        let change = |core: &mut Core, on: bool, path: &str| {
            core.handle(Event::NotificationSettingsChanged {
                messages: true,
                show_sender: true,
                sound_enabled: on,
                sound_path: path.into(),
            });
            notify(core, "a", "A", "b")
        };
        let e = change(&mut core, false, "/s.wav");
        assert_eq!(shown(&e).unwrap().2, &Sound::Off);
        let e = change(&mut core, true, "/s.wav");
        assert_eq!(shown(&e).unwrap().2, &Sound::File("/s.wav".into()));
        let e = change(&mut core, true, "");
        assert_eq!(shown(&e).unwrap().2, &Sound::Default);
    }

    // --- test notification ---

    fn test_notification(core: &mut Core, now: u64) -> Vec<Effect> {
        core.handle(Event::TestNotificationRequested { now })
    }

    #[test]
    fn test_notification_uses_the_active_account_and_the_normal_rules() {
        let mut core = started(TWO);
        let effects = test_notification(&mut core, 0);
        let (title, body, sound) = shown(&effects).unwrap();
        assert_eq!(title, "Account 1");
        assert_eq!(body, "Test: This is a test notification.");
        assert_eq!(sound, &Sound::Default);
        core.handle(Event::NotificationSettingsChanged {
            messages: true,
            show_sender: false,
            sound_enabled: false,
            sound_path: String::new(),
        });
        let effects = test_notification(&mut core, 0);
        let (_, body, sound) = shown(&effects).unwrap();
        assert_eq!((body, sound), ("New message", &Sound::Off));
    }

    #[test]
    fn test_notification_names_why_nothing_is_shown() {
        let mut core = started(TWO);
        assert_eq!(core.notification_block(0), None);
        core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 0,
        });
        assert_eq!(core.notification_block(0), Some(NotificationBlock::Snoozed));
        assert!(test_notification(&mut core, 0).is_empty());
        assert_eq!(core.notification_block(HOUR), None);
        core.handle(Event::SnoozeCleared);
        core.handle(Event::NotificationsToggled);
        assert_eq!(core.notification_block(0), Some(NotificationBlock::Off));
        assert!(test_notification(&mut core, 0).is_empty());
    }

    #[test]
    fn test_notification_is_blocked_when_the_active_account_is_muted() {
        let mut core = started(TWO);
        core.handle(Event::MuteToggled { id: "a".into() });
        assert_eq!(core.notification_block(0), Some(NotificationBlock::Muted));
        assert!(test_notification(&mut core, 0).is_empty());
    }

    // --- Snooze ---

    const HOUR: u64 = 3600;

    fn notify_at(core: &mut Core, now: u64) -> Vec<Effect> {
        core.handle(Event::NotificationRequested {
            account_id: "a".into(),
            title: "Alice".into(),
            body: "hi".into(),
            now,
        })
    }

    #[test]
    fn snooze_end_is_reported_only_while_snooze_is_active() {
        let mut core = started(TWO);
        assert_eq!(core.active_snooze_end(1000), None);
        core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 1000,
        });
        assert_eq!(core.active_snooze_end(1000), Some(1000 + HOUR));
        assert_eq!(core.active_snooze_end(1000 + HOUR), None);
        core.handle(Event::SnoozeCleared);
        assert_eq!(core.active_snooze_end(1000), None);
    }

    #[test]
    fn snooze_silences_all_accounts_until_its_end_time() {
        let mut core = started(TWO);
        core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 1000,
        });
        assert!(notify_at(&mut core, 1000).is_empty());
        assert!(core
            .handle(Event::NotificationRequested {
                account_id: "b".into(),
                title: "Bob".into(),
                body: "yo".into(),
                now: 1000 + HOUR - 1,
            })
            .is_empty());
        assert!(!notify_at(&mut core, 1000 + HOUR).is_empty());
    }

    #[test]
    fn snooze_survives_save_and_load() {
        let mut core = started(TWO);
        let effects = core.handle(Event::SnoozeSet {
            minutes: 8 * 60,
            now: 1000,
        });
        assert_eq!(saved(&effects)["snooze_until"], 1000 + 8 * HOUR);
        let mut restarted = started(&saved(&effects).to_string());
        assert!(notify_at(&mut restarted, 1000 + 8 * HOUR - 1).is_empty());
        assert!(!notify_at(&mut restarted, 1000 + 8 * HOUR).is_empty());
    }

    #[test]
    fn end_snooze_clears_it_and_alerts_come_back() {
        let mut core = started(TWO);
        core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 1000,
        });
        let effects = core.handle(Event::SnoozeCleared);
        assert!(saved(&effects)["snooze_until"].is_null());
        assert!(!notify_at(&mut core, 1001).is_empty());
    }

    #[test]
    fn snooze_only_saves_and_does_not_touch_the_unread_count() {
        let mut core = started(TWO);
        title(&mut core, "a", "(3) WhatsApp");
        for effects in [
            core.handle(Event::SnoozeSet {
                minutes: 60,
                now: 1000,
            }),
            core.handle(Event::SnoozeCleared),
        ] {
            assert!(matches!(effects[..], [Effect::SaveSettings { .. }]));
        }
        core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 1000,
        });
        assert!(core
            .handle(Event::SidebarReady)
            .contains(&unread(&[("a", 3)], 3)));
    }

    #[test]
    fn snooze_works_but_is_not_saved_while_save_is_blocked() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{bad".into())),
            fresh_id: "a".into(),
        });
        let effects = core.handle(Event::SnoozeSet {
            minutes: 60,
            now: 1000,
        });
        assert!(!has_save(&effects));
        assert!(notify_at(&mut core, 1000).is_empty());
        assert!(!has_save(&core.handle(Event::SnoozeCleared)));
    }

    #[test]
    fn notification_settings_change_is_saved() {
        let mut core = started(TWO);
        let effects = core.handle(Event::NotificationSettingsChanged {
            messages: false,
            show_sender: false,
            sound_enabled: false,
            sound_path: "/s.wav".into(),
        });
        let out = saved(&effects);
        let n = &out["notification_settings"];
        assert_eq!(n["show_message_notifications"], false);
        assert_eq!(n["show_sender"], false);
        assert_eq!(n["sound_enabled"], false);
        assert_eq!(n["sound_path"], "/s.wav");
    }

    #[test]
    fn toggle_is_saved_and_not_saved_while_blocked() {
        let mut core = started(TWO);
        let out = saved(&core.handle(Event::NotificationsToggled));
        assert_eq!(
            out["notification_settings"]["show_message_notifications"],
            false
        );
        let mut blocked = Core::default();
        blocked.handle(Event::SettingsLoaded {
            raw: Ok(Some("{bad".into())),
            fresh_id: "u1".into(),
        });
        assert!(!has_save(&blocked.handle(Event::NotificationsToggled)));
        assert!(!has_save(&blocked.handle(
            Event::NotificationSettingsChanged {
                messages: true,
                show_sender: true,
                sound_enabled: true,
                sound_path: String::new(),
            }
        )));
    }

    #[test]
    fn overlong_sound_path_is_rejected() {
        let mut core = started(TWO);
        let effects = core.handle(Event::NotificationSettingsChanged {
            messages: true,
            show_sender: true,
            sound_enabled: true,
            sound_path: "x".repeat(MAX_SOUND_PATH_CHARS + 1),
        });
        assert!(matches!(effects[..], [Effect::ReportError { .. }]));
    }

    #[test]
    fn start_minimized_keeps_window_hidden_at_start() {
        let mut core = loaded_core();
        assert_eq!(core.start_window_effects(), vec![Effect::ShowWindow]);
        core.handle(Event::StartMinimizedChanged { enabled: true });
        assert_eq!(core.start_window_effects(), Vec::new());
    }

    #[test]
    fn window_settings_save_and_persist() {
        let mut core = loaded_core();
        let effects = core.handle(Event::CloseToTrayChanged { enabled: true });
        assert_eq!(saved(&effects)["close_to_tray"], true);
        let effects = core.handle(Event::StartMinimizedChanged { enabled: true });
        assert_eq!(saved(&effects)["start_minimized"], true);
        assert!(core.close_to_tray() && core.start_minimized());
    }

    #[test]
    fn window_settings_are_not_saved_while_save_is_blocked() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{not json".into())),
            fresh_id: "u1".into(),
        });
        assert!(!has_save(
            &core.handle(Event::CloseToTrayChanged { enabled: true })
        ));
        assert!(!has_save(
            &core.handle(Event::StartMinimizedChanged { enabled: true })
        ));
        assert!(core.close_to_tray());
    }

    fn link(url: &str) -> Vec<Effect> {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(None),
            fresh_id: "u1".into(),
        });
        core.handle(Event::DeepLinkOpened { url: url.into() })
    }

    fn open(url: &str) -> Effect {
        Effect::OpenChat {
            id: "u1".into(),
            url: url.into(),
        }
    }

    #[test]
    fn recognises_both_deep_link_forms_and_opens_them_on_web_whatsapp() {
        assert_eq!(
            link("whatsapp://send?phone=123&text=hi%20there"),
            vec![open(
                "https://web.whatsapp.com/send?phone=123&text=hi+there"
            )]
        );
        assert_eq!(
            link("https://wa.me/123"),
            vec![open("https://web.whatsapp.com/send?phone=123")]
        );
        assert_eq!(
            link("https://wa.me/+123?text=a%26b"),
            vec![open("https://web.whatsapp.com/send?phone=123&text=a%26b")]
        );
    }

    #[test]
    fn refuses_other_urls_and_never_navigates_off_whatsapp() {
        for url in [
            "https://web.whatsapp.com",
            "https://evil.example/send?phone=1",
            "http://wa.me/123",
            "https://wa.me.evil.example/123",
            "https://wa.me@evil.example/123",
            "https://wa.me/12x3",
            "https://wa.me/",
            "whatsapp://other?phone=1",
            "whatsapp://send?phone=1/../x",
            "--start-minimized",
            "",
        ] {
            assert_eq!(link(url), Vec::new(), "{url}");
        }
    }

    #[test]
    fn deep_link_targets_the_active_account_and_changes_no_account() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(None),
            fresh_id: "u1".into(),
        });
        core.handle(Event::AddAccount {
            fresh_id: "u2".into(),
        });
        let effects = core.handle(Event::DeepLinkOpened {
            url: "https://wa.me/5".into(),
        });
        assert_eq!(
            effects,
            vec![Effect::OpenChat {
                id: "u2".into(),
                url: "https://web.whatsapp.com/send?phone=5".into()
            }]
        );
    }

    // --- Mute ---

    fn mute(core: &mut Core, id: &str) -> Vec<Effect> {
        core.handle(Event::MuteToggled { id: id.into() })
    }

    #[test]
    fn mute_silences_only_that_account_and_toggles_back() {
        let mut core = started(TWO);
        let effects = mute(&mut core, "a");
        assert_eq!(saved(&effects)["accounts"][0]["muted"], true);
        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::RenderSidebar { accounts, .. } if accounts[0].muted && !accounts[1].muted
        )));
        assert!(core.is_muted("a") && !core.is_muted("b"));
        assert!(notify(&mut core, "a", "Alice", "hi").is_empty());
        assert!(shown(&notify(&mut core, "b", "Bob", "hi")).is_some());
        mute(&mut core, "a");
        assert!(!core.is_muted("a"));
        assert!(shown(&notify(&mut core, "a", "Alice", "hi")).is_some());
    }

    #[test]
    fn mute_keeps_the_unread_count_and_is_ignored_for_unknown_accounts() {
        let mut core = started(TWO);
        mute(&mut core, "a");
        assert_eq!(
            title(&mut core, "a", "(3) WhatsApp"),
            vec![unread(&[("a", 3)], 3)]
        );
        assert!(mute(&mut core, "zz").is_empty());
    }

    #[test]
    fn mute_and_snooze_are_independent() {
        let mut core = started(TWO);
        mute(&mut core, "a");
        core.handle(Event::SnoozeSet {
            minutes: 1,
            now: 1000,
        });
        core.handle(Event::SnoozeCleared);
        assert!(core.is_muted("a"));
        assert!(notify_at(&mut core, 1000).is_empty());
        core.handle(Event::SnoozeSet {
            minutes: 1,
            now: 1000,
        });
        mute(&mut core, "a");
        assert!(notify_at(&mut core, 1000).is_empty());
        assert!(shown(&notify_at(&mut core, 1000 + 61)).is_some());
    }

    #[test]
    fn mute_stays_after_restart_and_old_files_load_unmuted() {
        let mut core = started(TWO);
        let json = saved(&mute(&mut core, "b")).to_string();
        let restarted = started(&json);
        assert!(restarted.is_muted("b") && !restarted.is_muted("a"));
    }

    // --- unread count ---

    fn title(core: &mut Core, id: &str, title: &str) -> Vec<Effect> {
        core.handle(Event::TitleChanged {
            id: id.into(),
            title: title.into(),
        })
    }

    fn unread(pairs: &[(&str, u32)], total: u32) -> Effect {
        Effect::ShowUnread {
            counts: pairs.iter().map(|(id, n)| (id.to_string(), *n)).collect(),
            total,
        }
    }

    #[test]
    fn title_with_count_sets_account_unread_count_and_total() {
        let mut core = started(TWO);
        assert_eq!(
            title(&mut core, "a", "(3) WhatsApp"),
            vec![unread(&[("a", 3)], 3)]
        );
    }

    #[test]
    fn title_without_count_clears_the_account_count() {
        let mut core = started(TWO);
        title(&mut core, "a", "(3) WhatsApp");
        assert_eq!(title(&mut core, "a", "WhatsApp"), vec![unread(&[], 0)]);
    }

    #[test]
    fn odd_titles_count_as_zero() {
        for odd in [
            "",
            "WhatsApp (3)",
            " (3) WhatsApp",
            "(abc) WhatsApp",
            "(-1) WhatsApp",
            "(3.5) WhatsApp",
            "(3 WhatsApp",
            "()",
            "(99999999999999999999) WhatsApp",
        ] {
            let mut core = started(TWO);
            title(&mut core, "a", "(2) WhatsApp");
            assert_eq!(title(&mut core, "a", odd), vec![unread(&[], 0)], "{odd}");
        }
    }

    #[test]
    fn unchanged_count_emits_nothing() {
        let mut core = started(TWO);
        assert!(title(&mut core, "a", "WhatsApp").is_empty());
        title(&mut core, "a", "(1) WhatsApp");
        assert!(title(&mut core, "a", "(1) WhatsApp - chat").is_empty());
    }

    #[test]
    fn total_is_the_sum_over_accounts() {
        let mut core = started(TWO);
        title(&mut core, "a", "(3) WhatsApp");
        assert_eq!(
            title(&mut core, "b", "(4) WhatsApp"),
            vec![unread(&[("a", 3), ("b", 4)], 7)]
        );
    }

    #[test]
    fn title_of_unknown_account_is_ignored() {
        let mut core = started(TWO);
        assert!(title(&mut core, "zz", "(5) WhatsApp").is_empty());
    }

    #[test]
    fn removing_an_account_removes_its_count_from_the_total() {
        let mut core = started(TWO);
        title(&mut core, "a", "(3) WhatsApp");
        title(&mut core, "b", "(4) WhatsApp");
        let effects = core.handle(Event::RemoveAccount {
            id: "b".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.contains(&unread(&[("a", 3)], 3)));
    }

    #[test]
    fn sidebar_ready_also_sends_unread_counts() {
        let mut core = started(TWO);
        title(&mut core, "b", "(2) WhatsApp");
        assert!(core
            .handle(Event::SidebarReady)
            .contains(&unread(&[("b", 2)], 2)));
    }

    #[test]
    fn unread_count_is_not_saved() {
        let mut core = started(TWO);
        let effects = title(&mut core, "a", "(3) WhatsApp");
        assert!(!has_save(&effects));
    }

    #[test]
    fn autostart_change_applies_and_saves() {
        let mut core = loaded_core();
        let effects = core.handle(Event::AutostartChanged { enabled: true });
        assert_eq!(
            effects.last(),
            Some(&Effect::SetAutostart { enabled: true })
        );
        assert_eq!(saved(&effects)["autostart_enabled"], true);
    }

    #[test]
    fn failed_autostart_change_is_reverted_and_saved() {
        let mut core = loaded_core();
        core.handle(Event::AutostartChanged { enabled: true });
        let effects = core.handle(Event::AutostartFailed { enabled: false });
        assert_eq!(saved(&effects)["autostart_enabled"], false);
        assert!(!core.autostart_enabled());
    }

    #[test]
    fn clear_cache_setting_is_saved() {
        let mut core = loaded_core();
        let effects = core.handle(Event::ClearCacheOnExitChanged { enabled: true });
        assert_eq!(saved(&effects)["clear_cache_on_exit"], true);
        assert!(core.clear_cache_on_exit());
    }

    #[test]
    fn quit_with_clear_cache_on_clears_every_account() {
        let mut core = started(TWO);
        core.handle(Event::ClearCacheOnExitChanged { enabled: true });
        assert_eq!(
            core.handle(Event::QuitRequested),
            vec![Effect::ClearAccountCaches {
                ids: vec!["a".into(), "b".into()]
            }]
        );
    }

    #[test]
    fn quit_with_clear_cache_off_does_nothing() {
        let mut core = started(TWO);
        assert!(core.handle(Event::QuitRequested).is_empty());
    }

    #[test]
    fn autostart_and_cache_settings_are_not_saved_while_save_is_blocked() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{not json".into())),
            fresh_id: "u1".into(),
        });
        let effects = core.handle(Event::AutostartChanged { enabled: true });
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::SetAutostart { .. })));
        assert!(!has_save(
            &core.handle(Event::AutostartFailed { enabled: false })
        ));
        assert!(!has_save(
            &core.handle(Event::ClearCacheOnExitChanged { enabled: true })
        ));
    }

    // --- Review fixes ---

    #[test]
    fn old_master_switch_off_does_not_silence_notifications() {
        let raw = r#"{"accounts":[{"id":"a","name":"A"}],"active_account":"a","notification_settings":{"enabled":false}}"#;
        let mut core = started(raw);
        assert!(shown(&notify(&mut core, "a", "Alice", "hi")).is_some());
    }

    #[test]
    fn bad_settings_file_keeps_its_error_and_blocks_autostart() {
        let mut core = Core::default();
        core.handle(Event::SettingsLoaded {
            raw: Ok(Some("{not json".into())),
            fresh_id: "n".into(),
        });
        assert!(core.load_error().is_some());
        let effects = core.handle(Event::AutostartChanged { enabled: true });
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::SetAutostart { .. })));
        assert!(matches!(effects[0], Effect::ReportError { .. }));
    }

    #[test]
    fn good_settings_file_has_no_load_error() {
        assert!(started(TWO).load_error().is_none());
    }

    #[test]
    fn removed_account_stays_pending_until_its_data_is_deleted() {
        let mut core = started(TWO);
        let effects = core.handle(Event::RemoveAccount {
            id: "b".into(),
            fresh_id: "n".into(),
        });
        assert_eq!(
            saved(&effects)["pending_deletions"],
            serde_json::json!(["b"])
        );

        let failed = core.handle(Event::AccountDataDeleted {
            id: "b".into(),
            result: Err("busy".into()),
        });
        assert!(matches!(failed[0], Effect::ReportError { .. }));
        assert!(!has_save(&failed));

        let done = core.handle(Event::AccountDataDeleted {
            id: "b".into(),
            result: Ok(()),
        });
        assert_eq!(saved(&done)["pending_deletions"], serde_json::json!([]));
    }

    #[test]
    fn pending_deletions_are_retried_before_any_webview_is_created() {
        let raw = r#"{"accounts":[{"id":"a","name":"A"}],"active_account":"a","pending_deletions":["old"]}"#;
        let effects = load(Some(raw), "unused");
        let delete = effects
            .iter()
            .position(|e| *e == Effect::DeleteAccountData { id: "old".into() })
            .expect("retry delete");
        let create = effects
            .iter()
            .position(|e| matches!(e, Effect::CreateAccountWebview { .. }))
            .expect("create webview");
        assert!(delete < create);
    }

    #[test]
    fn account_helpers_name_and_find_the_active_account() {
        let core = started(TWO);
        assert_eq!(core.account_name("b"), Some("Account 2"));
        assert_eq!(core.account_name("zz"), None);
        assert_eq!(core.active_account_id(), Some("a"));
    }

    #[test]
    fn only_https_whatsapp_web_is_an_account_origin() {
        let ok = |u: &str| is_account_origin(&url::Url::parse(u).unwrap());
        assert!(ok("https://web.whatsapp.com/"));
        assert!(ok("https://web.whatsapp.com/send?phone=1"));
        assert!(!ok("http://web.whatsapp.com/"));
        assert!(!ok("https://web.whatsapp.com.evil.com/"));
        assert!(!ok("https://example.com/"));
        assert!(!ok("https://web.whatsapp.com:8443/"));
    }

    #[test]
    fn only_https_whatsapp_net_subdomains_are_frame_origins() {
        let ok = |u: &str| is_whatsapp_frame_origin(&url::Url::parse(u).unwrap());
        assert!(ok("https://flows.whatsapp.net/flows/cache_management/"));
        assert!(ok("https://webtp.whatsapp.net/pdf-viewer/?locale=en_GB"));
        assert!(!ok("http://flows.whatsapp.net/"));
        assert!(!ok("https://whatsapp.net.evil.com/"));
        assert!(!ok("https://evilwhatsapp.net/"));
        assert!(!ok("https://flows.whatsapp.net:8443/"));
    }

    #[test]
    fn rename_trims_and_saves_and_keeps_the_id() {
        let mut core = started(TWO);
        let effects = core.handle(Event::RenameAccount {
            id: "b".into(),
            name: "  Work  ".into(),
        });
        let out = saved(&effects);
        assert_eq!(out["accounts"][1]["id"], "b");
        assert_eq!(out["accounts"][1]["name"], "Work");
        assert!(effects.iter().any(
            |e| matches!(e, Effect::RenderSidebar { accounts, .. } if accounts[1].name == "Work")
        ));
        assert_eq!(core.account_name("b"), Some("Work"));
    }

    #[test]
    fn rename_to_blank_or_unknown_account_does_nothing() {
        let mut core = started(TWO);
        for (id, name) in [("a", ""), ("a", " \t "), ("zz", "Work")] {
            let effects = core.handle(Event::RenameAccount {
                id: id.into(),
                name: name.into(),
            });
            assert!(effects.is_empty());
        }
        assert_eq!(core.account_name("a"), Some("Account 1"));
    }

    #[test]
    fn rename_cuts_long_names() {
        let mut core = started(TWO);
        core.handle(Event::RenameAccount {
            id: "a".into(),
            name: "x".repeat(MAX_ACCOUNT_NAME_CHARS + 20),
        });
        assert_eq!(
            core.account_name("a").unwrap().chars().count(),
            MAX_ACCOUNT_NAME_CHARS
        );
    }

    #[test]
    fn rename_with_blocked_save_renames_but_does_not_save() {
        let mut core = started("{not json");
        let id = core.active_account_id().unwrap().to_string();
        let effects = core.handle(Event::RenameAccount {
            id,
            name: "Work".into(),
        });
        assert!(!has_save(&effects));
        assert!(effects
            .iter()
            .any(|e| matches!(e, Effect::RenderSidebar { .. })));
    }

    #[test]
    fn retry_delay_doubles_and_stops_at_the_maximum() {
        let delays: Vec<_> = (1..=8).map(retry_delay_secs).collect();
        assert_eq!(delays, [5, 10, 20, 40, 80, 160, 300, 300]);
    }

    fn fail_load(core: &mut Core, id: &str) -> Vec<Effect> {
        let began = core.handle(Event::AccountLoadBegan { id: id.into() });
        let attempt = match began[..] {
            [Effect::ScheduleLoadCheck { attempt, .. }] => attempt,
            _ => panic!("expected one load check, got {began:?}"),
        };
        core.handle(Event::LoadCheckDue {
            id: id.into(),
            attempt,
        })
    }

    #[test]
    fn page_that_never_loads_shows_error_and_schedules_retry() {
        let mut core = started(TWO);
        let effects = fail_load(&mut core, "a");
        assert!(effects.contains(&Effect::ShowLoadError { id: "a".into() }));
        assert!(effects
            .iter()
            .any(|e| matches!(e, Effect::ScheduleRetry { id, secs: 5, .. } if id == "a")));
    }

    #[test]
    fn loaded_page_ignores_the_load_check() {
        let mut core = started(TWO);
        let began = core.handle(Event::AccountLoadBegan { id: "a".into() });
        core.handle(Event::PageLoaded { id: "a".into() });
        let Effect::ScheduleLoadCheck { attempt, .. } = began[0] else {
            panic!("load check")
        };
        let effects = core.handle(Event::LoadCheckDue {
            id: "a".into(),
            attempt,
        });
        assert!(effects.is_empty());
    }

    #[test]
    fn load_failure_affects_only_that_account() {
        let mut core = started(TWO);
        core.handle(Event::AccountLoadBegan { id: "b".into() });
        core.handle(Event::PageLoaded { id: "b".into() });
        let effects = fail_load(&mut core, "a");
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::ShowLoadError { id } if id == "b")));
    }

    #[test]
    fn retry_reloads_the_account_and_backs_off_on_the_next_failure() {
        let mut core = started(TWO);
        let first = fail_load(&mut core, "a");
        let Some(Effect::ScheduleRetry { attempt, .. }) = first
            .iter()
            .find(|e| matches!(e, Effect::ScheduleRetry { .. }))
        else {
            panic!("retry")
        };
        let effects = core.handle(Event::RetryDue {
            id: "a".into(),
            attempt: *attempt,
        });
        assert!(effects.contains(&Effect::ReloadAccount { id: "a".into() }));
        let Some(Effect::ScheduleLoadCheck { attempt, .. }) = effects
            .iter()
            .find(|e| matches!(e, Effect::ScheduleLoadCheck { .. }))
        else {
            panic!("check")
        };
        let again = core.handle(Event::LoadCheckDue {
            id: "a".into(),
            attempt: *attempt,
        });
        assert!(again
            .iter()
            .any(|e| matches!(e, Effect::ScheduleRetry { secs: 10, .. })));
    }

    #[test]
    fn page_load_clears_the_error_and_stops_retrying() {
        let mut core = started(TWO);
        let first = fail_load(&mut core, "a");
        let Some(Effect::ScheduleRetry { attempt, .. }) = first
            .iter()
            .find(|e| matches!(e, Effect::ScheduleRetry { .. }))
        else {
            panic!("retry")
        };
        let attempt = *attempt;
        let effects = core.handle(Event::PageLoaded { id: "a".into() });
        assert_eq!(effects, [Effect::HideLoadError { id: "a".into() }]);
        assert!(core
            .handle(Event::RetryDue {
                id: "a".into(),
                attempt
            })
            .is_empty());
    }

    #[test]
    fn manual_reload_uses_the_same_path_and_never_touches_data() {
        let mut core = started(TWO);
        let effects = core.handle(Event::ReloadRequested { id: "a".into() });
        assert!(effects.contains(&Effect::ReloadAccount { id: "a".into() }));
        assert!(!effects.iter().any(|e| matches!(
            e,
            Effect::DeleteAccountData { .. } | Effect::ClearAccountCaches { .. }
        )));
        assert!(core
            .handle(Event::ReloadRequested { id: "zz".into() })
            .is_empty());
    }

    #[test]
    fn removing_a_failed_account_hides_its_error() {
        let mut core = started(TWO);
        fail_load(&mut core, "b");
        let effects = core.handle(Event::RemoveAccount {
            id: "b".into(),
            fresh_id: "n".into(),
        });
        assert!(effects.contains(&Effect::HideLoadError { id: "b".into() }));
    }
}
