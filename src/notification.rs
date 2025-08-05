use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub show_message_notifications: bool,
    pub show_call_notifications: bool,
    pub sound_enabled: bool,
    pub show_sender: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_message_notifications: true,
            show_call_notifications: true,
            sound_enabled: true,
            show_sender: true,
        }
    }
}
#[derive(Debug)]
#[allow(dead_code)]
pub enum NotificationError {
    SystemNotSupported,
    PermissionDenied,
    SendFailed(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationError::SystemNotSupported => write!(f, "Notification system not supported"),
            NotificationError::PermissionDenied => write!(f, "Notification permission denied"),
            NotificationError::SendFailed(msg) => write!(f, "Failed to send notification: {}", msg),
        }
    }
}
impl std::error::Error for NotificationError {}

pub struct NotificationService {
    settings: NotificationSettings,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            settings: NotificationSettings::default(),
        }
    }

    pub fn update_settings(&mut self, settings: NotificationSettings) {
        self.settings = settings;
    }

    pub fn send_notification(
        &self,
        title: &str,
        message: &str,
        icon_path: Option<&str>,
    ) -> Result<(), NotificationError> {
        if !self.settings.enabled {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            self.send_linux_notification(title, message, icon_path)
        }

        #[cfg(target_os = "windows")]
        {
            self.send_windows_notification(title, message, icon_path)
        }

        #[cfg(target_os = "macos")]
        {
            self.send_macos_notification(title, message, icon_path)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(NotificationError::SystemNotSupported)
        }
    }

    #[cfg(target_os = "linux")]
    fn send_linux_notification(
        &self,
        title: &str,
        message: &str,
        icon_path: Option<&str>,
    ) -> Result<(), NotificationError> {
        use notify_rust::Notification;

        let mut notification = Notification::new();
        notification
            .summary(title)
            .body(message)
            .appname("WhatsApp-QT")
            .timeout(notify_rust::Timeout::Milliseconds(5000));
        if let Some(icon) = icon_path {
            notification.icon(icon);
        }

        if self.settings.sound_enabled {
            notification.sound_name("message-new-instant");
        }

        notification
            .show()
            .map_err(|e| NotificationError::SendFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn send_windows_notification(
        &self,
        title: &str,
        message: &str,
        _icon_path: Option<&str>,
    ) -> Result<(), NotificationError> {
        use winrt_notification::{Duration, Toast};

        let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(message)
            .duration(Duration::Short);

        if let Some(icon) = icon_path {
            toast = toast.icon(icon);
        }

        toast
            .show()
            .map_err(|e| NotificationError::SendFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn send_macos_notification(
        &self,
        title: &str,
        message: &str,
        _icon_path: Option<&str>,
    ) -> Result<(), NotificationError> {
        let script = format!(
            r#"display notification "{}" with title "{}""#,
            message.replace("\"", "\\\""),
            title.replace("\"", "\\\"")
        );

        let result = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(NotificationError::SendFailed(e.to_string())),
        }
    }

    pub fn test_notification(&self) -> Result<(), NotificationError> {
         let icon_path = std::env::current_dir()
            .unwrap()
            .join("resources/icons/tray.png")
            .to_string_lossy()
            .to_string();
        self.send_notification(
            "WhatsApp-QT",
            "Test Message from WhatsApp-QT",
            Some(&icon_path),
        )
    }
}
