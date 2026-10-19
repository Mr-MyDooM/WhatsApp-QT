mod avatar;
mod core;
#[cfg(target_os = "macos")]
mod dock;
mod notify;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{AboutMetadata, ContextMenu, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::{PageLoadEvent, WebviewBuilder};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Theme, WebviewUrl,
    WebviewWindowBuilder, WindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::avatar::Avatars;
use crate::core::{
    avatar_hue, initials, is_account_origin, is_page_local_url, is_whatsapp_frame_origin,
    theme_css, Core, Effect, Event, MenuAccount, NotificationBlock,
};

/// Avatar column. Wider than the macOS traffic lights (about 70 px), so they never reach the
/// account page. Keep the top strip in `frontend/src/sidebar/Sidebar.tsx` for them.
const SIDEBAR_NARROW: f64 = 72.0;
const SIDEBAR_WIDE: f64 = 220.0;
/// Width of the auto-hidden sidebar: the strip at the window edge that shows it on mouse over.
const SIDEBAR_EDGE: f64 = 6.0;
/// True while the auto-hidden sidebar shows over the page. Not saved.
static SIDEBAR_SHOWN: AtomicBool = AtomicBool::new(false);
const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;
const ACCOUNT_URL: &str = "https://web.whatsapp.com";
const ACCOUNT_LABEL_PREFIX: &str = "account-";
const ERROR_LABEL_PREFIX: &str = "error-";
const RENAME_LABEL: &str = "rename";
const SIDEBAR_LABEL: &str = "sidebar";
/// Drag strip above the account pages. The remote page may not move the window, so the
/// strip is a local page of its own.
const TITLE_BAR_LABEL: &str = "titlebar";
/// macOS title bar height. Other platforms keep their native title bar, so no strip.
#[cfg(target_os = "macos")]
const TITLE_BAR_HEIGHT: f64 = 28.0;
#[cfg(not(target_os = "macos"))]
const TITLE_BAR_HEIGHT: f64 = 0.0;
const MAX_SWITCH_SHORTCUTS: usize = 9;
const SETTINGS_LABEL: &str = "settings";
const ZOOM_STEP: f64 = 0.1;
/// Replaces `window.Notification` in each account page and sends the alert to `notify`.
const NOTIFICATION_SCRIPT: &str = include_str!("notification_shim.js");
/// Reads the user's own avatar in each account page and sends it to `profile_picture`.
const AVATAR_SCRIPT: &str = include_str!("avatar_shim.js");
/// Lets the user drag the edge of WhatsApp's chat list to resize it.
const CHAT_LIST_SCRIPT: &str = include_str!("chat_list_shim.js");
/// Longest wait for the cache clear at quit. The app exits after this even if WebKit is slow.
const CACHE_CLEAR_TIMEOUT: Duration = Duration::from_secs(3);
const AUTOSTART_FAILED_MESSAGE: &str = "Could not change start at login. The setting is unchanged.";
const SAVE_FAILED_MESSAGE: &str =
    "Could not save your settings. The change is active until you quit the app.";
/// WhatsApp Web rejects the default WKWebView user agent ("works with Safari 15+").
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("WhatsAppDesktop").join("settings.json"))
}

fn account_label(id: &str) -> String {
    format!("{ACCOUNT_LABEL_PREFIX}{id}")
}

/// Label of the local error panel that covers one account's area.
fn error_label(id: &str) -> String {
    format!("{ERROR_LABEL_PREFIX}{id}")
}

/// Runs `event` after `secs`. One short-lived thread per timer: there are few of them.
fn dispatch_later(app: &AppHandle, secs: u64, event: Event) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(secs));
        dispatch(&app, event);
    });
}

/// Covers the account's area with the local error panel. Does nothing if it is there.
fn show_load_error(app: &AppHandle, id: &str) {
    let label = error_label(id);
    if app.get_webview(&label).is_some() {
        return;
    }
    let Some(window) = app.get_window("main") else {
        return;
    };
    let size = match (window.inner_size(), window.scale_factor()) {
        (Ok(size), Ok(scale)) => size.to_logical::<f64>(scale),
        _ => return,
    };
    let (position, size) = content_bounds(sidebar_widths(app).0, size);
    let result = window.add_child(
        WebviewBuilder::new(label, WebviewUrl::App("error.html".into())),
        position,
        size,
    );
    if let Err(e) = result {
        eprintln!("Could not show the error panel for account {id}: {e}");
        return;
    }
    // The panel starts visible: hide it again if the account is not the active one.
    let active = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .active_account_id()
        .map(str::to_string);
    if let Some(active) = active {
        show_account(app, &active);
    }
}

#[cfg(not(target_os = "macos"))]
fn account_data_dir(app: &AppHandle, uuid: &uuid::Uuid) -> tauri::Result<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()?
        .join("accounts")
        .join(uuid.to_string()))
}

/// Removes only the disk and memory cache of one account. Sends `()` once when the clear is
/// done or failed.
///
/// Allowed data types, and why. Never add a type that holds the session:
/// - `WKWebsiteDataTypeDiskCache`, `WKWebsiteDataTypeMemoryCache`: HTTP caches. Safe.
/// - NOT allowed: cookies, local storage, session storage, IndexedDB, service worker
///   registrations, offline web application cache, WebSQL, file system. WhatsApp Web keeps
///   the login in IndexedDB and local storage, so removing any of them logs the user out.
#[cfg(target_os = "macos")]
fn clear_account_cache(webview: &tauri::Webview, done: std::sync::mpsc::Sender<()>) {
    use block2::RcBlock;
    use objc2_foundation::{NSDate, NSSet};
    use objc2_web_kit::{WKWebView, WKWebsiteDataTypeDiskCache, WKWebsiteDataTypeMemoryCache};

    let tx = done.clone();
    let queued = webview.with_webview(move |platform| {
        // SAFETY: `inner()` is the live WKWebView of this account. This closure runs on the
        // main thread, and the webview is not closed before the app exits.
        unsafe {
            let view = &*(platform.inner() as *const WKWebView);
            let store = view.configuration().websiteDataStore();
            let types =
                NSSet::from_slice(&[WKWebsiteDataTypeDiskCache, WKWebsiteDataTypeMemoryCache]);
            let since = NSDate::dateWithTimeIntervalSince1970(0.0);
            let handler = RcBlock::new(move || {
                let _ = tx.send(());
            });
            store.removeDataOfTypes_modifiedSince_completionHandler(&types, &since, &handler);
        }
    });
    if let Err(e) = queued {
        eprintln!("Could not clear cache: {e}");
        let _ = done.send(());
    }
}

/// Starts the cache clear for each id and returns how many completions to wait for.
/// Always 0 off macOS: there is no way to clear the cache there without removing logins.
fn clear_account_caches(
    app: &AppHandle,
    ids: &[String],
    done: &std::sync::mpsc::Sender<()>,
) -> usize {
    #[cfg(target_os = "macos")]
    {
        let mut pending = 0;
        for id in ids {
            if let Some(webview) = app.get_webview(&account_label(id)) {
                clear_account_cache(&webview, done.clone());
                pending += 1;
            }
        }
        pending
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, ids, done);
        0
    }
}

/// Runs on quit. If a clear is pending, holds the exit until it is done or the timeout hits.
fn begin_quit(app: &AppHandle, api: Option<&tauri::ExitRequestApi>) {
    static HANDLED: AtomicBool = AtomicBool::new(false);
    if HANDLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let effects = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .handle(Event::QuitRequested);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut pending = 0;
    for effect in effects {
        if let Effect::ClearAccountCaches { ids } = effect {
            pending += clear_account_caches(app, &ids, &tx);
        }
    }
    if pending == 0 {
        if api.is_none() {
            app.exit(0);
        }
        return;
    }
    if let Some(api) = api {
        api.prevent_exit();
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + CACHE_CLEAR_TIMEOUT;
        for _ in 0..pending {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            match rx.recv_timeout(left) {
                Ok(()) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    eprintln!("Cache clear did not finish in time; quitting anyway");
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("Cache clear sender dropped before it finished; quitting anyway");
                    break;
                }
            }
        }
        app.exit(0);
    });
}

/// Deletes the account's data store for good. The account webview is already closed.
/// Reports the result to the core, which keeps the id listed until the delete worked.
fn delete_account_data(app: &AppHandle, id: &str) {
    let uuid = match uuid::Uuid::parse_str(id) {
        Ok(uuid) => uuid,
        Err(e) => {
            let result = Err(format!("bad account id: {e}"));
            return dispatch(
                app,
                Event::AccountDataDeleted {
                    id: id.to_string(),
                    result,
                },
            );
        }
    };
    // WebKit refuses while the closed webview still holds the store, and the close finishes
    // a moment later. So try again for a few seconds before reporting the error.
    #[cfg(target_os = "macos")]
    {
        const ATTEMPTS: u32 = 5;
        let app = app.clone();
        let id = id.to_string();
        let bytes = *uuid.as_bytes();
        std::thread::spawn(move || {
            let mut result = Ok(());
            for attempt in 1..=ATTEMPTS {
                std::thread::sleep(Duration::from_secs(1));
                result = tauri::async_runtime::block_on(app.remove_data_store(bytes))
                    .map_err(|e| e.to_string());
                if result.is_ok() {
                    break;
                }
                eprintln!("Account data {id} still in use, attempt {attempt} of {ATTEMPTS}");
            }
            dispatch(&app, Event::AccountDataDeleted { id, result });
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let result = account_data_dir(app, &uuid)
            .map_err(|e| e.to_string())
            .and_then(|dir| match std::fs::remove_dir_all(dir) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
                _ => Ok(()),
            });
        dispatch(
            app,
            Event::AccountDataDeleted {
                id: id.to_string(),
                result,
            },
        );
    }
}

/// Opens http and https links in the default browser. Other schemes are dropped.
fn open_in_browser(url: &url::Url) {
    if !matches!(url.scheme(), "http" | "https") {
        eprintln!("Blocked navigation to {url}");
        return;
    }
    if let Err(e) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
        eprintln!("Could not open {url} in the browser: {e}");
    }
}

fn add_account_webview(
    app: &AppHandle,
    window: &tauri::Window,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::parse_str(id)?;
    let chat_list_script = format!(
        "window.__PAIRLINE_CHAT_LIST_WIDTH__ = {};\n{CHAT_LIST_SCRIPT}",
        serde_json::json!(chat_list_width(app))
    );
    let builder = WebviewBuilder::new(
        account_label(id),
        WebviewUrl::External(ACCOUNT_URL.parse()?),
    )
    .user_agent(USER_AGENT)
    // Only WhatsApp Web itself loads in the account webview. Links to other sites open
    // in the default browser. This check also runs for frames, so WhatsApp's own frame
    // hosts load in place.
    .on_navigation(|url| {
        if is_account_origin(url) || is_whatsapp_frame_origin(url) || is_page_local_url(url) {
            return true;
        }
        open_in_browser(url);
        false
    })
    .on_new_window(|url, _features| {
        if is_account_origin(&url) {
            return tauri::webview::NewWindowResponse::Allow;
        }
        open_in_browser(&url);
        tauri::webview::NewWindowResponse::Deny
    })
    .initialization_script(NOTIFICATION_SCRIPT)
    .initialization_script(AVATAR_SCRIPT)
    .initialization_script(&chat_list_script)
    // The unread count comes from the page title, read here on the Rust side, so the
    // remote page needs no IPC command for it.
    .on_document_title_changed(|webview, title| {
        if let Some(id) = webview.label().strip_prefix(ACCOUNT_LABEL_PREFIX) {
            let id = id.to_string();
            dispatch(webview.app_handle(), Event::TitleChanged { id, title });
        }
    });
    #[cfg(target_os = "macos")]
    let builder = builder.data_store_identifier(*uuid.as_bytes());
    #[cfg(not(target_os = "macos"))]
    let builder = builder.data_directory(account_data_dir(app, &uuid)?);

    let size = window
        .inner_size()?
        .to_logical::<f64>(window.scale_factor()?);
    let (position, size) = content_bounds(sidebar_widths(app).0, size);
    window.add_child(builder, position, size)?;
    Ok(())
}

/// Where an account page or its error panel sits: right of the sidebar, under the title bar.
fn content_bounds(
    sidebar: f64,
    size: LogicalSize<f64>,
) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    (
        LogicalPosition::new(sidebar, TITLE_BAR_HEIGHT),
        LogicalSize::new(
            (size.width - sidebar).max(1.0),
            (size.height - TITLE_BAR_HEIGHT).max(1.0),
        ),
    )
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn show_account(app: &AppHandle, id: &str) {
    let active = account_label(id);
    let active_error = error_label(id);
    for (label, webview) in app.webviews() {
        if label.starts_with(ACCOUNT_LABEL_PREFIX) || label.starts_with(ERROR_LABEL_PREFIX) {
            let _ = if label == active || label == active_error {
                webview.show()
            } else {
                webview.hide()
            };
        }
    }
}

/// Returns where the page starts and how wide the sidebar webview is. An auto-hidden sidebar
/// shows over the page, so the page starts at the edge strip.
fn sidebar_widths(app: &AppHandle) -> (f64, f64) {
    let core = app.state::<Mutex<Core>>();
    let core = core.lock().expect("core lock");
    let full = if core.sidebar_wide() {
        SIDEBAR_WIDE
    } else {
        SIDEBAR_NARROW
    };
    if !core.sidebar_auto_hide() {
        return (full, full);
    }
    if SIDEBAR_SHOWN.load(Ordering::Relaxed) {
        (SIDEBAR_EDGE, full)
    } else {
        (SIDEBAR_EDGE, SIDEBAR_EDGE)
    }
}

fn layout(app: &AppHandle, window: &tauri::Window, size: tauri::PhysicalSize<u32>) {
    let size = size.to_logical::<f64>(window.scale_factor().unwrap_or(1.0));
    let (left, sidebar) = sidebar_widths(app);
    for (label, webview) in app.webviews() {
        if label == SIDEBAR_LABEL {
            let _ = webview.set_size(LogicalSize::new(sidebar, size.height));
        } else if label == TITLE_BAR_LABEL {
            let _ = webview.set_position(LogicalPosition::new(left, 0.0));
            let _ = webview.set_size(LogicalSize::new(
                (size.width - left).max(1.0),
                TITLE_BAR_HEIGHT,
            ));
        } else if label.starts_with(ACCOUNT_LABEL_PREFIX) || label.starts_with(ERROR_LABEL_PREFIX) {
            let (position, bounds) = content_bounds(left, size);
            let _ = webview.set_position(position);
            let _ = webview.set_size(bounds);
        }
    }
}

/// Tells the sidebar page which state it is in.
fn emit_sidebar_state(app: &AppHandle) {
    let (wide, auto_hide) = {
        let core = app.state::<Mutex<Core>>();
        let core = core.lock().expect("core lock");
        (core.sidebar_wide(), core.sidebar_auto_hide())
    };
    let result = app
        .emit_to(SIDEBAR_LABEL, "sidebar-wide", wide)
        .and_then(|()| app.emit_to(SIDEBAR_LABEL, "sidebar-auto-hide", auto_hide));
    if let Err(e) = result {
        eprintln!("Could not update the sidebar state: {e}");
    }
}

fn apply_sidebar(app: &AppHandle) {
    SIDEBAR_SHOWN.store(false, Ordering::Relaxed);
    if let Some(window) = app.get_window("main") {
        if let Ok(size) = window.inner_size() {
            layout(app, &window, size);
        }
    }
    emit_sidebar_state(app);
}

/// Logs the detail, and shows the friendly message on the settings page if it is open.
pub(crate) fn report_error(app: &AppHandle, friendly: &str, detail: &str) {
    eprintln!("{detail}");
    let _ = app.emit_to(SETTINGS_LABEL, "app-error", friendly);
}

/// Empty `css` removes the injected theme.
fn inject_css(webview: &tauri::Webview, css: &str) {
    let literal = serde_json::to_string(css).expect("string serializes");
    let _ = webview.eval(format!(
        "(function(){{var e=document.getElementById('app-theme-style');if(e)e.remove();\
         var c={literal};if(!c)return;var s=document.createElement('style');\
         s.id='app-theme-style';s.textContent=c;document.head.appendChild(s);}})();"
    ));
}

/// Sidebar gets each account's count; tray tooltip and Dock badge get the total.
fn show_unread(app: &AppHandle, counts: BTreeMap<String, u32>, total: u32) {
    if let Err(e) = app.emit_to(SIDEBAR_LABEL, "unread", counts) {
        eprintln!("Could not update unread counts in the sidebar: {e}");
    }
    if let Some(window) = app.get_window("main") {
        // `None` removes the badge.
        let badge = (total > 0).then_some(i64::from(total));
        if let Err(e) = window.set_badge_count(badge) {
            eprintln!("Could not set the Dock badge: {e}");
        }
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = if total > 0 {
            format!("{APP_NAME} ({total} unread)")
        } else {
            APP_NAME.to_string()
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }
    refresh_account_menus(app);
}

fn run_effects(app: &AppHandle, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::CreateAccountWebview { id } => {
                let Some(window) = app.get_window("main") else {
                    continue;
                };
                match add_account_webview(app, &window, &id) {
                    Ok(()) => dispatch(app, Event::AccountLoadBegan { id }),
                    Err(e) => eprintln!("Could not create account webview {id}: {e}"),
                }
            }
            Effect::CloseAccountWebview { id } => {
                if let Some(webview) = app.get_webview(&account_label(&id)) {
                    if let Err(e) = webview.close() {
                        eprintln!("Could not close account webview {id}: {e}");
                    }
                }
            }
            Effect::DeleteAccountData { id } => {
                app.state::<Mutex<Avatars>>()
                    .lock()
                    .expect("avatars lock")
                    .remove(&id);
                emit_avatars(app);
                delete_account_data(app, &id)
            }
            Effect::ShowAccount { id } => show_account(app, &id),
            Effect::ApplyTheme { theme, css } => {
                let scheme = match theme.as_str() {
                    "dark" => Some(Theme::Dark),
                    "light" => Some(Theme::Light),
                    _ => None,
                };
                if let Some(window) = app.get_window("main") {
                    let _ = window.set_theme(scheme);
                }
                for (label, webview) in app.webviews() {
                    if label.starts_with(ACCOUNT_LABEL_PREFIX) {
                        inject_css(&webview, &css);
                    }
                }
            }
            Effect::ApplyZoom { factor } => {
                for (label, webview) in app.webviews() {
                    if label.starts_with(ACCOUNT_LABEL_PREFIX) {
                        let _ = webview.set_zoom(factor);
                    }
                }
            }
            Effect::SetAutostart { enabled } => {
                let manager = app.autolaunch();
                let result = if enabled {
                    manager.enable()
                } else {
                    manager.disable()
                };
                if let Err(e) = result {
                    report_error(
                        app,
                        AUTOSTART_FAILED_MESSAGE,
                        &format!("Could not set start at login to {enabled}: {e}"),
                    );
                    dispatch(app, Event::AutostartFailed { enabled: !enabled });
                }
            }
            // Handled in `begin_quit`: it must hold the exit until the clear is done.
            Effect::ClearAccountCaches { .. } => {}
            Effect::RenderSidebar { accounts, active } => {
                refresh_account_menus(app);
                let accounts: Vec<_> = accounts
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "id": a.id,
                            "name": a.name,
                            "initials": initials(&a.name),
                            "hue": avatar_hue(&a.id),
                            "muted": a.muted,
                        })
                    })
                    .collect();
                let payload = serde_json::json!({ "accounts": accounts, "active": active });
                if let Err(e) = app.emit_to(SIDEBAR_LABEL, "accounts", payload) {
                    eprintln!("Could not update sidebar: {e}");
                }
            }
            Effect::ShowNotification {
                account_id,
                title,
                body,
                sound,
            } => notify::show(app.clone(), account_id, title, body, sound),
            Effect::SaveSettings { json } => {
                let Some(path) = settings_path() else {
                    report_error(app, SAVE_FAILED_MESSAGE, "Could not find config directory");
                    continue;
                };
                let result = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, json));
                if let Err(e) = result {
                    report_error(
                        app,
                        SAVE_FAILED_MESSAGE,
                        &format!("Could not save settings to {}: {e}", path.display()),
                    );
                }
            }
            Effect::ReportError { message } => {
                report_error(app, "Something went wrong with your settings.", &message)
            }
            Effect::ShowWindow => show_main_window(app),
            Effect::HideWindow => {
                if let Some(window) = app.get_window("main") {
                    let _ = window.hide();
                }
            }
            Effect::Quit => begin_quit(app, None),
            Effect::OpenChat { id, url } => {
                show_main_window(app);
                let Some(webview) = app.get_webview(&account_label(&id)) else {
                    continue;
                };
                match url.parse() {
                    Ok(url) => {
                        if let Err(e) = webview.navigate(url) {
                            eprintln!("Could not open chat in account {id}: {e}");
                        }
                    }
                    Err(e) => eprintln!("Could not parse chat URL: {e}"),
                }
            }
            Effect::ReloadAccount { id } => {
                // Loads the start page again. Cookies, local storage and IndexedDB stay.
                if let Some(webview) = app.get_webview(&account_label(&id)) {
                    if let Err(e) = webview.navigate(ACCOUNT_URL.parse().expect("valid URL")) {
                        eprintln!("Could not reload account {id}: {e}");
                    }
                }
            }
            Effect::ScheduleLoadCheck { id, attempt, secs } => {
                dispatch_later(app, secs, Event::LoadCheckDue { id, attempt })
            }
            Effect::ScheduleRetry { id, attempt, secs } => {
                dispatch_later(app, secs, Event::RetryDue { id, attempt })
            }
            Effect::ShowLoadError { id } => show_load_error(app, &id),
            Effect::HideLoadError { id } => {
                if let Some(webview) = app.get_webview(&error_label(&id)) {
                    if let Err(e) = webview.close() {
                        eprintln!("Could not close the error panel for account {id}: {e}");
                    }
                }
            }
            Effect::ApplySidebar => apply_sidebar(app),
            Effect::ShowUnread { counts, total } => show_unread(app, counts, total),
        }
    }
}

pub(crate) fn dispatch(app: &AppHandle, event: Event) {
    let effects = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .handle(event);
    run_effects(app, effects);
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// These commands are allowed for the local sidebar webview only: see `build.rs` and
// `capabilities/default.json`. The remote web.whatsapp.com webviews get no permission.
#[tauri::command]
fn sidebar_ready(app: AppHandle) {
    dispatch(&app, Event::SidebarReady);
    emit_sidebar_state(&app);
}

#[tauri::command]
fn toggle_sidebar(app: AppHandle) {
    dispatch(&app, Event::SidebarToggled);
}

/// The mouse entered or left the auto-hidden sidebar: show it over the page, or hide it.
#[tauri::command]
fn reveal_sidebar(app: AppHandle, shown: bool) {
    let auto_hide = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .sidebar_auto_hide();
    let Some(window) = app.get_window("main") else {
        return;
    };
    if !auto_hide {
        return;
    }
    SIDEBAR_SHOWN.store(shown, Ordering::Relaxed);
    // Moving the sidebar into the window again puts it in front of the account pages.
    if let (true, Some(sidebar)) = (shown, app.get_webview(SIDEBAR_LABEL)) {
        if let Err(e) = sidebar.reparent(&window) {
            eprintln!("Could not bring the sidebar to the front: {e}");
        }
    }
    if let Ok(size) = window.inner_size() {
        layout(&app, &window, size);
    }
}

#[tauri::command]
fn open_settings_window(app: AppHandle) {
    open_settings(&app);
}

#[tauri::command]
fn add_account(app: AppHandle) {
    dispatch(&app, Event::AddAccount { fresh_id: new_id() });
}

#[tauri::command]
fn switch_account(app: AppHandle, id: String) {
    dispatch(&app, Event::SwitchAccount { id });
}

/// Asks the user in a native dialog, then removes the account. `window.confirm` cannot be
/// used: WKWebView returns false for it without a UI delegate.
fn confirm_remove_account(app: &AppHandle, id: String) {
    let Some(name) = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .account_name(&id)
        .map(str::to_string)
    else {
        return;
    };
    let handle = app.clone();
    app.dialog()
        .message(format!(
            "Remove \"{name}\" and delete its login and data? This cannot be undone."
        ))
        .title("Remove account")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Remove".into(),
            "Cancel".into(),
        ))
        .show(move |confirmed| {
            if confirmed {
                dispatch(
                    &handle,
                    Event::RemoveAccount {
                        id,
                        fresh_id: new_id(),
                    },
                );
            }
        });
}

#[tauri::command]
fn remove_account(app: AppHandle, id: String) {
    confirm_remove_account(&app, id);
}

/// Opens the native menu for one account at the pointer.
#[tauri::command]
fn show_account_menu(app: AppHandle, id: String) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let mute_label = if app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .is_muted(&id)
    {
        "Unmute"
    } else {
        "Mute"
    };
    let result = (|| -> tauri::Result<()> {
        let menu = Menu::with_items(
            &app,
            &[
                &MenuItem::with_id(
                    &app,
                    format!("{MENU_CTX_RENAME}{id}"),
                    "Rename...",
                    true,
                    None::<&str>,
                )?,
                &MenuItem::with_id(
                    &app,
                    format!("{MENU_CTX_RELOAD}{id}"),
                    "Reload",
                    true,
                    None::<&str>,
                )?,
                &MenuItem::with_id(
                    &app,
                    format!("{MENU_CTX_MUTE}{id}"),
                    mute_label,
                    true,
                    None::<&str>,
                )?,
                &MenuItem::with_id(
                    &app,
                    format!("{MENU_CTX_REMOVE}{id}"),
                    "Remove...",
                    true,
                    None::<&str>,
                )?,
            ],
        )?;
        menu.popup(window)
    })();
    if let Err(e) = result {
        eprintln!("Could not show the account menu: {e}");
    }
}

/// Asks for a new account name in a small local window.
fn open_rename(app: &AppHandle, id: &str) {
    if let Some(w) = app.get_webview_window(RENAME_LABEL) {
        let _ = w.set_focus();
        return;
    }
    let Some(name) = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .account_name(id)
        .map(str::to_string)
    else {
        return;
    };
    let mut url = url::Url::parse("http://localhost/rename.html").expect("valid URL");
    url.query_pairs_mut()
        .append_pair("id", id)
        .append_pair("name", &name);
    let path = format!("rename.html?{}", url.query().unwrap_or_default());
    let result = WebviewWindowBuilder::new(app, RENAME_LABEL, WebviewUrl::App(path.into()))
        .title("Rename account")
        .inner_size(380.0, 150.0)
        .resizable(false)
        .build();
    if let Err(e) = result {
        eprintln!("Could not open rename window: {e}");
    }
}

fn close_rename(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(RENAME_LABEL) {
        let _ = w.close();
    }
}

// Allowed for the local rename window only: see `capabilities/rename.json`.
#[tauri::command]
fn rename_account(app: AppHandle, id: String, name: String) {
    dispatch(&app, Event::RenameAccount { id, name });
    close_rename(&app);
}

/// The Rename button of the settings window: opens the same rename window as the account menu.
#[tauri::command]
fn open_rename_account(app: AppHandle, id: String) {
    open_rename(&app, &id);
}

#[tauri::command]
fn cancel_rename(app: AppHandle) {
    close_rename(&app);
}

/// The Retry button of the error panel. The account comes from the calling webview's label.
#[tauri::command]
fn retry_account(app: AppHandle, webview: tauri::Webview) {
    if let Some(id) = webview.label().strip_prefix(ERROR_LABEL_PREFIX) {
        dispatch(&app, Event::ReloadRequested { id: id.to_string() });
    }
}

fn toggle_window(app: &AppHandle) {
    let window_visible = app
        .get_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    dispatch(app, Event::ToggleWindow { window_visible });
}

/// Current time in Unix seconds. A clock before 1970 counts as 0.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn snooze(app: &AppHandle, minutes: u64) {
    dispatch(
        app,
        Event::SnoozeSet {
            minutes,
            now: unix_now(),
        },
    );
}

/// The account lines of the tray and Dock menus, as the core decides them.
fn menu_accounts(app: &AppHandle) -> Vec<MenuAccount> {
    app.state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .menu_accounts()
}

fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
        app,
        TRAY_TOGGLE,
        "Show/Hide",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let accounts = menu_accounts(app);
    for a in &accounts {
        menu.append(&MenuItem::with_id(
            app,
            format!("{TRAY_ACCOUNT_PREFIX}{}", a.id),
            &a.label,
            true,
            None::<&str>,
        )?)?;
    }
    if !accounts.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&MenuItem::with_id(
        app,
        TRAY_SNOOZE_1H,
        "Snooze 1 hour",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        TRAY_SNOOZE_8H,
        "Snooze 8 hours",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        TRAY_END_SNOOZE,
        "End Snooze",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        TRAY_QUIT,
        "Quit",
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

/// Rebuilds the tray menu and the Dock list after accounts or unread counts changed.
fn refresh_account_menus(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        match tray_menu(app) {
            Ok(menu) => {
                if let Err(e) = tray.set_menu(Some(menu)) {
                    eprintln!("Could not update the tray menu: {e}");
                }
            }
            Err(e) => eprintln!("Could not build the tray menu: {e}"),
        }
    }
    if let Err(e) = refresh_switch_items(app) {
        eprintln!("Could not update the Account menu: {e}");
    }
    #[cfg(target_os = "macos")]
    dock::set_entries(menu_accounts(app));
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = tray_menu(app)?;
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_TOGGLE => toggle_window(app),
            TRAY_SNOOZE_1H => snooze(app, 60),
            TRAY_SNOOZE_8H => snooze(app, 8 * 60),
            TRAY_END_SNOOZE => dispatch(app, Event::SnoozeCleared),
            TRAY_QUIT => begin_quit(app, None),
            other => {
                // Same effect as a notification click: show the window, switch account.
                if let Some(id) = other.strip_prefix(TRAY_ACCOUNT_PREFIX) {
                    dispatch(
                        app,
                        Event::NotificationClicked {
                            account_id: id.to_string(),
                        },
                    );
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = w.set_focus();
        return;
    }
    let result =
        WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App("settings.html".into()))
            .title("General")
            .inner_size(560.0, 500.0)
            .build();
    if let Err(e) = result {
        eprintln!("Could not open settings window: {e}");
    }
}

/// The only command the remote page may call: see `capabilities/remote.json`.
/// The account comes from the calling webview's label, never from the page's data.
#[tauri::command]
fn notify(app: AppHandle, webview: tauri::Webview, title: String, body: String) {
    let Some(id) = webview.label().strip_prefix(ACCOUNT_LABEL_PREFIX) else {
        return;
    };
    dispatch(
        &app,
        Event::NotificationRequested {
            account_id: id.to_string(),
            title,
            body,
            now: unix_now(),
        },
    );
}

/// The second and last command the remote page may call. The page script sends its own
/// avatar. The account comes from the webview label; the data URL is checked in `avatar.rs`.
/// Calls from any other webview, or for an unknown account, are ignored.
#[tauri::command]
fn profile_picture(app: AppHandle, webview: tauri::Webview, data_url: String) {
    let Some(id) = webview.label().strip_prefix(ACCOUNT_LABEL_PREFIX) else {
        return;
    };
    let known = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .account_name(id)
        .is_some();
    if !known {
        return;
    }
    let result = app
        .state::<Mutex<Avatars>>()
        .lock()
        .expect("avatars lock")
        .submit(id, &data_url, unix_now());
    match result {
        Ok(true) => emit_avatars(&app),
        Ok(false) => {}
        Err(e) => log::warn!("Profile picture of {id} refused: {e}"),
    }
}

/// Local webviews only. Id to data URL; accounts without a picture are absent.
#[tauri::command]
fn get_avatars(avatars: tauri::State<Mutex<Avatars>>) -> serde_json::Value {
    serde_json::json!(avatars.lock().expect("avatars lock").all())
}

fn emit_avatars(app: &AppHandle) {
    let payload = serde_json::json!(app
        .state::<Mutex<Avatars>>()
        .lock()
        .expect("avatars lock")
        .all());
    for label in [SIDEBAR_LABEL, SETTINGS_LABEL] {
        // The settings window may be closed: that is not an error.
        let _ = app.emit_to(label, "avatars", payload.clone());
    }
}

// Allowed for the local settings webview only: see `build.rs` and `capabilities/settings.json`.
// The system answers on its own thread, so these run on the async pool, not the main thread.
#[tauri::command]
async fn notification_permission() -> &'static str {
    notify::permission_status()
}

#[tauri::command]
async fn request_notification_permission() -> &'static str {
    notify::request_permission()
}

#[tauri::command]
fn open_notification_settings() {
    const PANE: &str = "x-apple.systempreferences:com.apple.Notifications-Settings.extension";
    if let Err(e) = tauri_plugin_opener::open_url(PANE, None::<&str>) {
        eprintln!("Could not open System Settings: {e}");
    }
}

/// Sends one test message notification. Returns "sent", or the reason no alert will come:
/// "off", "snoozed", "muted", "denied" or "no_bundle".
#[tauri::command]
async fn send_test_notification(app: AppHandle) -> &'static str {
    let now = unix_now();
    let block = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .notification_block(now);
    match block {
        Some(NotificationBlock::Off) => return "off",
        Some(NotificationBlock::Snoozed) => return "snoozed",
        Some(NotificationBlock::Muted) => return "muted",
        None => {}
    }
    // "not_asked" still sends: the system asks the user when the alert is shown.
    let permission = notify::permission_status();
    if permission == "denied" || permission == "no_bundle" {
        return permission;
    }
    let target = app.clone();
    let _ =
        app.run_on_main_thread(move || dispatch(&target, Event::TestNotificationRequested { now }));
    "sent"
}

/// Snoozes message notifications. Returns the end time in Unix seconds, like the tray items.
#[tauri::command]
fn snooze_notifications(app: AppHandle, minutes: u64) -> Option<u64> {
    snooze(&app, minutes);
    app.state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .active_snooze_end(unix_now())
}

#[tauri::command]
fn end_snooze(app: AppHandle) {
    dispatch(&app, Event::SnoozeCleared);
}

/// Opens the system file picker for a sound file. Returns the path, or nothing if cancelled.
/// Blocks until the user answers, so it runs on the async pool.
#[tauri::command]
async fn choose_sound_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Sound files", &["aiff", "wav", "mp3", "m4a", "caf"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn get_about() -> serde_json::Value {
    serde_json::json!({
        "name": APP_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "description": "An unofficial desktop client for WhatsApp Web.",
        "license": env!("CARGO_PKG_LICENSE"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "settings_path": settings_path().map(|p| p.display().to_string()),
    })
}

/// The only link the About pane opens. The page cannot choose the address.
#[tauri::command]
fn open_repository() {
    if let Err(e) = tauri_plugin_opener::open_url(env!("CARGO_PKG_REPOSITORY"), None::<&str>) {
        eprintln!("Could not open the repository page: {e}");
    }
}

#[tauri::command]
fn get_settings(core: tauri::State<Mutex<Core>>) -> serde_json::Value {
    let core = core.lock().expect("core lock");
    let n = core.notification_settings();
    serde_json::json!({
        "theme": core.theme(),
        "zoom_factor": core.zoom_factor(),
        "close_to_tray": core.close_to_tray(),
        "sidebar_auto_hide": core.sidebar_auto_hide(),
        "start_minimized": core.start_minimized(),
        "notifications": n.show_message_notifications,
        "show_sender": n.show_sender,
        "sound_enabled": n.sound_enabled,
        "sound_path": n.sound_path,
        "autostart_enabled": core.autostart_enabled(),
        "clear_cache_on_exit": core.clear_cache_on_exit(),
        // Only macOS can clear the cache without removing logins.
        "clear_cache_supported": cfg!(target_os = "macos"),
        // Set when the settings file was unusable: changes are not saved then.
        "load_error": core.load_error(),
        "snooze_until": core.active_snooze_end(unix_now()),
        "accounts": core.accounts(),
    })
}

/// The user dragged the edge of the chat list in an account page. `None` is WhatsApp's own
/// width. Saves the width and applies it to every account page.
#[tauri::command]
fn set_chat_list_width(app: AppHandle, webview: tauri::Webview, width: Option<f64>) {
    if !webview.label().starts_with(ACCOUNT_LABEL_PREFIX) {
        return;
    }
    dispatch(&app, Event::ChatListResized { width });
    let js = format!(
        "window.__pairlineSetChatListWidth && window.__pairlineSetChatListWidth({})",
        serde_json::json!(chat_list_width(&app))
    );
    for (label, account) in app.webviews() {
        if label.starts_with(ACCOUNT_LABEL_PREFIX) {
            if let Err(e) = account.eval(&js) {
                eprintln!("Could not resize the chat list in {label}: {e}");
            }
        }
    }
}

fn chat_list_width(app: &AppHandle) -> Option<f64> {
    app.state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .chat_list_width()
}

#[tauri::command]
fn set_sidebar_auto_hide(app: AppHandle, enabled: bool) {
    dispatch(&app, Event::SidebarAutoHideChanged { enabled });
}

#[tauri::command]
fn set_close_to_tray(app: AppHandle, enabled: bool) {
    dispatch(&app, Event::CloseToTrayChanged { enabled });
}

#[tauri::command]
fn set_start_minimized(app: AppHandle, enabled: bool) {
    dispatch(&app, Event::StartMinimizedChanged { enabled });
}

#[tauri::command]
fn set_theme(app: AppHandle, theme: String) {
    dispatch(&app, Event::ThemeChanged { theme });
}

#[tauri::command]
fn set_notifications(
    app: AppHandle,
    messages: bool,
    show_sender: bool,
    sound_enabled: bool,
    sound_path: String,
) {
    dispatch(
        &app,
        Event::NotificationSettingsChanged {
            messages,
            show_sender,
            sound_enabled,
            sound_path,
        },
    );
}

#[tauri::command]
fn set_zoom(app: AppHandle, factor: f64) {
    dispatch(&app, Event::ZoomChanged { factor });
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) {
    dispatch(&app, Event::AutostartChanged { enabled });
}

#[tauri::command]
fn set_clear_cache_on_exit(app: AppHandle, enabled: bool) {
    dispatch(&app, Event::ClearCacheOnExitChanged { enabled });
}

const TRAY_ID: &str = "main";
const APP_NAME: &str = "Pairline";
const ABOUT_COMMENTS: &str = "An unofficial desktop client for WhatsApp Web.\nNot affiliated with or endorsed by WhatsApp or Meta.";
const TRAY_TOGGLE: &str = "tray-toggle";
const TRAY_ACCOUNT_PREFIX: &str = "tray-account-";
const TRAY_QUIT: &str = "tray-quit";
const TRAY_SNOOZE_1H: &str = "tray-snooze-1h";
const TRAY_SNOOZE_8H: &str = "tray-snooze-8h";
const TRAY_END_SNOOZE: &str = "tray-end-snooze";
const MENU_QUIT: &str = "app-quit";
const MENU_SETTINGS: &str = "settings";
const MENU_TOGGLE_SIDEBAR: &str = "sidebar-toggle";
const MENU_ZOOM_IN: &str = "zoom-in";
const MENU_ZOOM_OUT: &str = "zoom-out";
const MENU_ZOOM_RESET: &str = "zoom-reset";
const MENU_NOTIFICATIONS: &str = "notifications-toggle";
const MENU_ADD: &str = "account-add";
const MENU_REMOVE: &str = "account-remove";
const MENU_NEXT: &str = "account-next";
const MENU_PREVIOUS: &str = "account-previous";
const MENU_RELOAD: &str = "account-reload";
const MENU_CTX_RENAME: &str = "ctx-rename:";
const MENU_CTX_RELOAD: &str = "ctx-reload:";
const MENU_CTX_MUTE: &str = "ctx-mute:";
const MENU_CTX_REMOVE: &str = "ctx-remove:";
const MENU_SWITCH_PREFIX: &str = "account-switch-";
const MENU_ACCOUNT: &str = "account-menu";
/// Place of the first account item in the Account menu: after New, Remove, Reload, the line,
/// Next and Previous.
const MENU_SWITCH_POSITION: usize = 6;

/// Puts one item per account (the first nine) in the Account menu, each with its own
/// Cmd+number shortcut, so the menu shows the same accounts as the sidebar and settings.
fn refresh_switch_items(app: &AppHandle) -> tauri::Result<()> {
    let Some(menu) = app.menu() else {
        return Ok(());
    };
    let Some(submenu) = menu.get(MENU_ACCOUNT).and_then(|m| m.as_submenu().cloned()) else {
        return Ok(());
    };
    for n in 1..=MAX_SWITCH_SHORTCUTS {
        if let Some(item) = submenu.get(&format!("{MENU_SWITCH_PREFIX}{n}")) {
            submenu.remove(&item)?;
        }
    }
    for (i, account) in menu_accounts(app)
        .iter()
        .take(MAX_SWITCH_SHORTCUTS)
        .enumerate()
    {
        let n = i + 1;
        submenu.insert(
            &MenuItem::with_id(
                app,
                format!("{MENU_SWITCH_PREFIX}{n}"),
                &account.label,
                true,
                Some(format!("CmdOrCtrl+{n}").as_str()),
            )?,
            MENU_SWITCH_POSITION + i,
        )?;
    }
    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let app_menu = Submenu::with_items(
        app,
        "Pairline",
        true,
        &[
            &PredefinedMenuItem::about(
                app,
                None,
                Some(AboutMetadata {
                    comments: Some(ABOUT_COMMENTS.to_string()),
                    ..Default::default()
                }),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_SETTINGS, "Settings...", true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            // Not the predefined quit: it exits without `begin_quit`, so Cmd+Q would skip the
            // cache clear. The Dock menu Quit and macOS logout bypass this item too and go
            // through `RunEvent::ExitRequested`, which calls `begin_quit` as well.
            &MenuItem::with_id(app, MENU_QUIT, "Quit Pairline", true, Some("CmdOrCtrl+Q"))?,
        ],
    )?;
    // Without an Edit menu, Cmd+C, Cmd+V and friends do nothing in the webviews on macOS.
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let account_menu = Submenu::with_id_and_items(
        app,
        MENU_ACCOUNT,
        "Account",
        true,
        &[
            &MenuItem::with_id(app, MENU_ADD, "New Account", true, Some("CmdOrCtrl+T"))?,
            &MenuItem::with_id(
                app,
                MENU_REMOVE,
                "Remove Account",
                true,
                Some("CmdOrCtrl+W"),
            )?,
            &MenuItem::with_id(
                app,
                MENU_RELOAD,
                "Reload Account",
                true,
                Some("CmdOrCtrl+R"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_NEXT, "Next Account", true, Some("Ctrl+Tab"))?,
            &MenuItem::with_id(
                app,
                MENU_PREVIOUS,
                "Previous Account",
                true,
                Some("Ctrl+Shift+Tab"),
            )?,
        ],
    )?;
    account_menu.append(&PredefinedMenuItem::separator(app)?)?;
    account_menu.append(&MenuItem::with_id(
        app,
        MENU_NOTIFICATIONS,
        "Toggle Notifications",
        true,
        Some("CmdOrCtrl+Shift+N"),
    )?)?;
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(
                app,
                MENU_TOGGLE_SIDEBAR,
                "Toggle Sidebar",
                true,
                Some("Ctrl+Cmd+S"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_ZOOM_IN, "Zoom In", true, Some("CmdOrCtrl+="))?,
            &MenuItem::with_id(app, MENU_ZOOM_OUT, "Zoom Out", true, Some("CmdOrCtrl+-"))?,
            &MenuItem::with_id(
                app,
                MENU_ZOOM_RESET,
                "Actual Size",
                true,
                Some("CmdOrCtrl+0"),
            )?,
        ],
    )?;
    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[&PredefinedMenuItem::minimize(app, None)?],
    )?;
    Menu::with_items(
        app,
        &[
            &app_menu,
            &edit_menu,
            &view_menu,
            &account_menu,
            &window_menu,
        ],
    )
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        MENU_ADD => dispatch(app, Event::AddAccount { fresh_id: new_id() }),
        MENU_QUIT => begin_quit(app, None),
        MENU_REMOVE => {
            let active = app
                .state::<Mutex<Core>>()
                .lock()
                .expect("core lock")
                .active_account_id()
                .map(str::to_string);
            if let Some(id) = active {
                confirm_remove_account(app, id);
            }
        }
        MENU_RELOAD => {
            let active = app
                .state::<Mutex<Core>>()
                .lock()
                .expect("core lock")
                .active_account_id()
                .map(str::to_string);
            if let Some(id) = active {
                dispatch(app, Event::ReloadRequested { id });
            }
        }
        MENU_SETTINGS => open_settings(app),
        MENU_TOGGLE_SIDEBAR => dispatch(app, Event::SidebarToggled),
        MENU_NOTIFICATIONS => dispatch(app, Event::NotificationsToggled),
        MENU_ZOOM_IN => dispatch(app, Event::ZoomStepped { delta: ZOOM_STEP }),
        MENU_ZOOM_OUT => dispatch(app, Event::ZoomStepped { delta: -ZOOM_STEP }),
        MENU_ZOOM_RESET => dispatch(app, Event::ZoomChanged { factor: 1.0 }),
        MENU_NEXT => dispatch(app, Event::SwitchRelative(1)),
        MENU_PREVIOUS => dispatch(app, Event::SwitchRelative(-1)),
        _ if id.starts_with(MENU_CTX_RENAME) => open_rename(app, &id[MENU_CTX_RENAME.len()..]),
        _ if id.starts_with(MENU_CTX_RELOAD) => dispatch(
            app,
            Event::ReloadRequested {
                id: id[MENU_CTX_RELOAD.len()..].to_string(),
            },
        ),
        _ if id.starts_with(MENU_CTX_MUTE) => dispatch(
            app,
            Event::MuteToggled {
                id: id[MENU_CTX_MUTE.len()..].to_string(),
            },
        ),
        _ if id.starts_with(MENU_CTX_REMOVE) => {
            confirm_remove_account(app, id[MENU_CTX_REMOVE.len()..].to_string())
        }
        _ => {
            if let Some(n) = id
                .strip_prefix(MENU_SWITCH_PREFIX)
                .and_then(|n| n.parse::<usize>().ok())
            {
                dispatch(app, Event::SwitchToIndex(n - 1));
            }
        }
    }
}

fn dispatch_deep_links(app: &AppHandle, urls: impl IntoIterator<Item = String>) {
    for url in urls {
        dispatch(app, Event::DeepLinkOpened { url });
    }
}

fn main() {
    env_logger::init();
    tauri::Builder::default()
        // First, so a second copy hands its link to the running one and exits.
        // The `deep-link` feature turns the second copy's argv into an on_open_urls call.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // The second copy exits; show this one so the launch has a visible result.
            show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            sidebar_ready,
            toggle_sidebar,
            reveal_sidebar,
            open_settings_window,
            add_account,
            switch_account,
            remove_account,
            show_account_menu,
            rename_account,
            cancel_rename,
            open_rename_account,
            retry_account,
            get_settings,
            notification_permission,
            request_notification_permission,
            open_notification_settings,
            send_test_notification,
            snooze_notifications,
            end_snooze,
            choose_sound_file,
            get_about,
            open_repository,
            set_theme,
            set_zoom,
            set_autostart,
            set_clear_cache_on_exit,
            set_close_to_tray,
            set_sidebar_auto_hide,
            set_chat_list_width,
            set_start_minimized,
            set_notifications,
            notify,
            profile_picture,
            get_avatars
        ])
        // SIZE and POSITION only: restoring VISIBLE would defeat start-minimized.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION,
                )
                .with_denylist(&[SETTINGS_LABEL, RENAME_LABEL])
                .build(),
        )
        .menu(build_menu)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .on_page_load(|webview, payload| {
            let Some(id) = webview.label().strip_prefix(ACCOUNT_LABEL_PREFIX) else {
                return;
            };
            if payload.event() != PageLoadEvent::Finished {
                return;
            }
            dispatch(
                webview.app_handle(),
                Event::PageLoaded { id: id.to_string() },
            );
            let (css, zoom) = {
                let state = webview.app_handle().state::<Mutex<Core>>();
                let core = state.lock().expect("core lock");
                (theme_css(core.theme()), core.zoom_factor())
            };
            inject_css(webview, &css);
            let _ = webview.set_zoom(zoom);
        })
        .setup(|app| {
            let builder = WindowBuilder::new(app, "main")
                .title("Pairline")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .visible(false);
            // The sidebar page is transparent: the window draws the macOS sidebar material.
            // The overlay title bar lets that material run under the traffic lights, so the
            // window has no separate title bar strip.
            #[cfg(target_os = "macos")]
            let builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true)
                .transparent(true)
                .effects(
                    tauri::window::EffectsBuilder::new()
                        .effect(tauri::window::Effect::Sidebar)
                        .state(tauri::window::EffectState::FollowsWindowActiveState)
                        .build(),
                );
            let window = builder.build()?;
            let sidebar = WebviewBuilder::new(SIDEBAR_LABEL, WebviewUrl::App("index.html".into()));
            #[cfg(target_os = "macos")]
            let sidebar = sidebar.transparent(true);
            window.add_child(
                sidebar,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_NARROW, WINDOW_HEIGHT),
            )?;
            // Transparent, so the strip shows the same sidebar material as the avatar column.
            #[cfg(target_os = "macos")]
            window.add_child(
                WebviewBuilder::new(TITLE_BAR_LABEL, WebviewUrl::App("titlebar.html".into()))
                    .transparent(true),
                LogicalPosition::new(SIDEBAR_NARROW, 0.0),
                LogicalSize::new(WINDOW_WIDTH - SIDEBAR_NARROW, TITLE_BAR_HEIGHT),
            )?;

            let raw = match settings_path().map(|p| (std::fs::read_to_string(&p), p)) {
                None => Ok(None),
                Some((Ok(text), _)) => Ok(Some(text)),
                Some((Err(e), _)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Some((Err(e), p)) => Err(format!("Could not read {}: {e}", p.display())),
            };
            let handle = app.handle().clone();
            notify::init(handle.clone());
            app.manage(Mutex::new(Core::default()));
            app.manage(Mutex::new(Avatars::load(
                app.path().app_data_dir()?.join("avatars"),
            )));
            dispatch(
                &handle,
                Event::SettingsLoaded {
                    raw,
                    fresh_id: new_id(),
                },
            );
            let appearance = handle
                .state::<Mutex<Core>>()
                .lock()
                .expect("core lock")
                .appearance_effects();
            run_effects(&handle, appearance);
            apply_sidebar(&handle);

            build_tray(&handle)?;
            #[cfg(target_os = "macos")]
            {
                dock::init(handle.clone());
                dock::set_entries(menu_accounts(&handle));
            }
            let start = handle
                .state::<Mutex<Core>>()
                .lock()
                .expect("core lock")
                .start_window_effects();
            run_effects(&handle, start);
            // Link that launched the app (argv on Linux and Windows, Apple Event on macOS),
            // then links that arrive while it runs.
            let links = app.deep_link();
            if let Some(urls) = links.get_current()? {
                dispatch_deep_links(&handle, urls.iter().map(|u| u.to_string()));
            }
            let link_handle = handle.clone();
            links.on_open_url(move |event| {
                dispatch_deep_links(&link_handle, event.urls().iter().map(|u| u.to_string()));
            });

            let w = window.clone();
            window.on_window_event(move |event| match event {
                WindowEvent::Resized(size) => layout(&handle, &w, *size),
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    dispatch(&handle, Event::CloseRequested);
                }
                _ => {}
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building WhatsApp-QT")
        .run(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => begin_quit(app, Some(&api)),
                // Dock click after close-to-tray: the window is hidden, show it again.
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { .. } => show_main_window(app),
                _ => {}
            }
        });
}
