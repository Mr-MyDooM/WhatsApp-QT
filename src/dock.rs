//! macOS Dock menu: one line per account. Tauri has no Dock menu API.
//!
//! AppKit asks the app delegate for `applicationDockMenu:` each time the user opens the
//! Dock menu. The delegate is tao's own class, which does not define that method. We add
//! it to that class at run time, so Tauri's delegate stays in place and keeps all its
//! other methods. The menu is built fresh on each call from `ENTRIES`, which `set_entries`
//! keeps current, so it cannot go stale.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Imp, NSObject, Sel};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::{ns_string, NSString};
use tauri::AppHandle;

use crate::core::{Event, MenuAccount};
use crate::dispatch;

/// Account lines for the next Dock menu. Written from any thread, read on the main thread.
static ENTRIES: Mutex<Vec<MenuAccount>> = Mutex::new(Vec::new());
/// Address of the leaked `PickTarget`. Zero until `init` ran.
static TARGET: AtomicUsize = AtomicUsize::new(0);

pub fn set_entries(entries: Vec<MenuAccount>) {
    *ENTRIES.lock().expect("dock entries lock") = entries;
}

struct Ivars {
    app: AppHandle,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "WhatsAppQtDockTarget"]
    #[ivars = Ivars]
    struct PickTarget;

    impl PickTarget {
        #[unsafe(method(pickAccount:))]
        fn pick_account(&self, sender: &NSMenuItem) {
            let Some(id) = sender
                .representedObject()
                .and_then(|o| o.downcast::<NSString>().ok())
                .map(|s| s.to_string())
            else {
                return;
            };
            // Same effect as a notification click: show the window, make the account active.
            dispatch(
                &self.ivars().app,
                Event::NotificationClicked { account_id: id },
            );
        }
    }
);

/// The method AppKit calls. Returns an autoreleased menu, or nil when there is no account.
extern "C" fn dock_menu(_this: &AnyObject, _cmd: Sel, _app: &AnyObject) -> *mut NSMenu {
    let target = TARGET.load(Ordering::Relaxed) as *const PickTarget;
    let entries = ENTRIES.lock().expect("dock entries lock").clone();
    let Some(mtm) = MainThreadMarker::new() else {
        return std::ptr::null_mut();
    };
    if target.is_null() || entries.is_empty() {
        return std::ptr::null_mut();
    }
    // Safety: the pointer comes from a leaked `PickTarget` in `init` and is never freed.
    let target = unsafe { &*target };
    let menu = NSMenu::new(mtm);
    for entry in entries {
        let item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                &NSString::from_str(&entry.label),
                Some(sel!(pickAccount:)),
                ns_string!(""),
            )
        };
        unsafe { item.setTarget(Some(target)) };
        let id = NSString::from_str(&entry.id);
        unsafe { item.setRepresentedObject(Some(&id)) };
    }
    Retained::autorelease_return(menu)
}

/// Call once, on the main thread, after the app delegate exists.
pub fn init(app: AppHandle) {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("Dock menu must be set up on the main thread");
        return;
    };
    let Some(delegate) = NSApplication::sharedApplication(mtm).delegate() else {
        eprintln!("No app delegate: the Dock menu is off");
        return;
    };
    let delegate: &AnyObject = unsafe { &*(&*delegate as *const _ as *const AnyObject) };
    let class = delegate.class() as *const AnyClass as *mut AnyClass;
    // Safety: `dock_menu` has the signature `applicationDockMenu:` needs: (id, SEL, id) -> id.
    let imp: Imp = unsafe {
        std::mem::transmute::<extern "C" fn(&AnyObject, Sel, &AnyObject) -> *mut NSMenu, Imp>(
            dock_menu,
        )
    };
    let added = unsafe {
        objc2::ffi::class_addMethod(class, sel!(applicationDockMenu:), imp, c"@@:@".as_ptr())
    };
    if !added.as_bool() {
        eprintln!("The app delegate already has a Dock menu: the account list is off");
        return;
    }
    let this = mtm.alloc::<PickTarget>().set_ivars(Ivars { app });
    let target: Retained<PickTarget> = unsafe { msg_send![super(this), init] };
    // Leaked on purpose: menu items hold their target weakly and it lives for the process.
    TARGET.store(Retained::into_raw(target) as usize, Ordering::Relaxed);
}
