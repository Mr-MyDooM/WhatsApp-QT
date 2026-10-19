//! Shows one native notification. The core already decided what to show.

use tauri::AppHandle;

use crate::core::Sound;

/// macOS: shows the notification without waiting for a click.
///
/// mac-notification-sys keeps one global click delegate that each send replaces, and a
/// waiting send blocks its thread forever. So a click cannot be routed to the right
/// account, and waiting would leak one thread per notification. We do not wait.
#[cfg(target_os = "macos")]
pub fn show(_app: AppHandle, _account_id: String, title: String, body: String, sound: Sound) {
    use mac_notification_sys::Notification;

    let mut notification = Notification::new();
    notification.title(&title).message(&body).asynchronous(true);
    match &sound {
        Sound::Off => {}
        Sound::Default => {
            notification.default_sound();
        }
        // A notification can only play system sounds by name, so play the file itself.
        Sound::File(path) => {
            if let Err(e) = std::process::Command::new("afplay").arg(path).spawn() {
                eprintln!("Could not play notification sound {path}: {e}");
            }
        }
    }
    if let Err(e) = notification.send() {
        eprintln!("Could not show notification: {e}");
    }
}

// ponytail: no click handling and no sound on Linux and Windows, add with their port.
#[cfg(not(target_os = "macos"))]
pub fn show(_app: AppHandle, _account_id: String, title: String, body: String, _sound: Sound) {
    if let Err(e) = notify_rust::Notification::new()
        .summary(&title)
        .body(&body)
        .show()
    {
        eprintln!("Could not show notification: {e}");
    }
}
