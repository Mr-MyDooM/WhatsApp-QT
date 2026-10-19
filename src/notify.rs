//! Shows one native notification. The core already decided what to show.

use tauri::AppHandle;

use crate::core::{Event, Sound};
use crate::dispatch;

/// macOS: waits for a click on a worker thread, then reports it to the core.
#[cfg(target_os = "macos")]
pub fn show(app: AppHandle, account_id: String, title: String, body: String, sound: Sound) {
    use mac_notification_sys::{Notification, NotificationResponse};

    std::thread::spawn(move || {
        let mut notification = Notification::new();
        notification
            .title(&title)
            .message(&body)
            .wait_for_click(true);
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
        match notification.send() {
            Ok(NotificationResponse::Click) => {
                dispatch(&app, Event::NotificationClicked { account_id })
            }
            Ok(_) => {}
            Err(e) => eprintln!("Could not show notification: {e}"),
        }
    });
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
