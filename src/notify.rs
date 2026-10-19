//! Shows one native notification. The core already decided what to show.

use tauri::AppHandle;

use crate::core::Sound;

#[cfg(target_os = "macos")]
mod mac {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool, NSObject, NSObjectProtocol, ProtocolObject};
    use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
    use objc2_foundation::{NSBundle, NSDictionary, NSError, NSString};
    use objc2_user_notifications::{
        UNAuthorizationOptions, UNAuthorizationStatus, UNMutableNotificationContent,
        UNNotification, UNNotificationPresentationOptions, UNNotificationRequest,
        UNNotificationResponse, UNNotificationSettings, UNNotificationSound,
        UNUserNotificationCenter, UNUserNotificationCenterDelegate,
    };
    use tauri::AppHandle;

    use crate::core::{Event, Sound};
    use crate::{dispatch, report_error};

    const ACCOUNT_KEY: &str = "account_id";
    const REPLY_TIMEOUT: Duration = Duration::from_secs(60);
    const DENIED_MESSAGE: &str =
        "Notifications are turned off for this app. Turn them on in System Settings > Notifications.";

    struct Ivars {
        app: AppHandle,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "WhatsAppQtNotificationDelegate"]
        #[ivars = Ivars]
        struct NotificationDelegate;

        unsafe impl NSObjectProtocol for NotificationDelegate {}

        unsafe impl UNUserNotificationCenterDelegate for NotificationDelegate {
            // Without this, macOS hides the alert while the app is in front.
            #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
            fn will_present(
                &self,
                _center: &UNUserNotificationCenter,
                _notification: &UNNotification,
                handler: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
            ) {
                handler.call((UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::List
                    | UNNotificationPresentationOptions::Sound,));
            }

            #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
            fn did_receive(
                &self,
                _center: &UNUserNotificationCenter,
                response: &UNNotificationResponse,
                handler: &block2::DynBlock<dyn Fn()>,
            ) {
                let info = response.notification().request().content().userInfo();
                let key = NSString::from_str(ACCOUNT_KEY);
                let account_id = info
                    .objectForKey(&key)
                    .and_then(|v| v.downcast::<NSString>().ok())
                    .map(|s| s.to_string());
                if let Some(account_id) = account_id {
                    let app = self.ivars().app.clone();
                    let target = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        dispatch(&target, Event::NotificationClicked { account_id })
                    });
                }
                handler.call(());
            }
        }
    );

    /// Without an app bundle, `UNUserNotificationCenter` raises an exception, so skip it.
    fn has_bundle() -> bool {
        NSBundle::mainBundle().bundleIdentifier().is_some()
    }

    /// Call once, on the main thread, before the first notification.
    pub fn init(app: AppHandle) {
        if !has_bundle() {
            eprintln!("Not running from an app bundle: notifications and their clicks are off");
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            eprintln!("Notification delegate must be set on the main thread");
            return;
        };
        let this = mtm.alloc::<NotificationDelegate>().set_ivars(Ivars { app });
        let delegate: Retained<NotificationDelegate> = unsafe { msg_send![super(this), init] };
        UNUserNotificationCenter::currentNotificationCenter()
            .setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        // The center holds its delegate weakly. Leaked on purpose: it lives for the process.
        std::mem::forget(delegate);
    }

    pub fn show(app: AppHandle, account_id: String, title: String, body: String, sound: Sound) {
        if !has_bundle() {
            return;
        }
        request_authorization(move |granted| {
            if !granted {
                // Report once, not for every message.
                static REPORTED: AtomicBool = AtomicBool::new(false);
                if !REPORTED.swap(true, Ordering::Relaxed) {
                    report_error(&app, DENIED_MESSAGE, "Notification permission refused");
                }
                return;
            }
            add(&account_id, &title, &body, &sound);
        });
    }

    /// The system asks the user only the first time. Later calls answer at once.
    fn request_authorization(on_answer: impl Fn(bool) + 'static) {
        let block =
            RcBlock::new(move |granted: Bool, _error: *mut NSError| on_answer(granted.as_bool()));
        UNUserNotificationCenter::currentNotificationCenter()
            .requestAuthorizationWithOptions_completionHandler(
                UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
                &block,
            );
    }

    /// Maps a raw `UNAuthorizationStatus` to the key the settings page understands.
    pub(super) fn status_key(raw: isize) -> &'static str {
        match UNAuthorizationStatus(raw) {
            UNAuthorizationStatus::NotDetermined => "not_asked",
            UNAuthorizationStatus::Denied => "denied",
            _ => "allowed",
        }
    }

    /// Blocks until the system answers: call it off the main thread.
    pub fn permission_status() -> &'static str {
        if !has_bundle() {
            return "no_bundle";
        }
        let (tx, rx) = mpsc::channel();
        let block = RcBlock::new(move |settings: std::ptr::NonNull<UNNotificationSettings>| {
            let raw = unsafe { settings.as_ref() }.authorizationStatus().0;
            let _ = tx.send(raw);
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .getNotificationSettingsWithCompletionHandler(&block);
        rx.recv_timeout(REPLY_TIMEOUT)
            .map_or("not_asked", status_key)
    }

    /// Shows the system prompt when nobody answered yet. Returns the status after the answer.
    /// Blocks: call it off the main thread.
    pub fn request_permission() -> &'static str {
        let status = permission_status();
        if status != "not_asked" {
            return status;
        }
        let (tx, rx) = mpsc::channel();
        request_authorization(move |granted| {
            let _ = tx.send(granted);
        });
        let _ = rx.recv_timeout(REPLY_TIMEOUT);
        permission_status()
    }

    fn add(account_id: &str, title: &str, body: &str, sound: &Sound) {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(title));
        content.setBody(&NSString::from_str(body));
        let key = NSString::from_str(ACCOUNT_KEY);
        let value = NSString::from_str(account_id);
        let info: Retained<NSDictionary<AnyObject, AnyObject>> = unsafe {
            Retained::cast_unchecked(NSDictionary::from_retained_objects(&[&*key], &[value]))
        };
        unsafe { content.setUserInfo(&info) };
        match sound {
            Sound::Off => {}
            Sound::Default => content.setSound(Some(&UNNotificationSound::defaultSound())),
            // A notification can only play system sounds by name, so play the file itself.
            Sound::File(path) => {
                if let Err(e) = std::process::Command::new("afplay").arg(path).spawn() {
                    eprintln!("Could not play notification sound {path}: {e}");
                }
            }
        }
        let id = NSString::from_str(&uuid::Uuid::new_v4().to_string());
        let request =
            UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, None);
        let done = RcBlock::new(|error: *mut NSError| {
            if !error.is_null() {
                eprintln!("Could not show notification");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .addNotificationRequest_withCompletionHandler(&request, Some(&done));
    }
}

/// Sets the click delegate on macOS, once, on the main thread. Other platforms have no
/// click handling yet.
pub fn init(app: AppHandle) {
    #[cfg(target_os = "macos")]
    mac::init(app);
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}

/// One of "allowed", "denied", "not_asked", "no_bundle", "unsupported". Blocks on macOS.
pub fn permission_status() -> &'static str {
    #[cfg(target_os = "macos")]
    return mac::permission_status();
    #[cfg(not(target_os = "macos"))]
    "unsupported"
}

/// Asks the system for permission, then returns the new status. Blocks on macOS.
pub fn request_permission() -> &'static str {
    #[cfg(target_os = "macos")]
    return mac::request_permission();
    #[cfg(not(target_os = "macos"))]
    "unsupported"
}

/// macOS: one delegate for the whole process reads the account id from each notification
/// and sends the click to the core. No thread is created or kept waiting per notification.
#[cfg(target_os = "macos")]
pub fn show(app: AppHandle, account_id: String, title: String, body: String, sound: Sound) {
    mac::show(app, account_id, title, body, sound);
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::mac::status_key;

    #[test]
    fn maps_each_system_status_to_a_key() {
        assert_eq!(status_key(0), "not_asked");
        assert_eq!(status_key(1), "denied");
        assert_eq!(status_key(2), "allowed");
        assert_eq!(status_key(3), "allowed");
        assert_eq!(status_key(4), "allowed");
    }
}
