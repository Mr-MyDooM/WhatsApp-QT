mod core;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::webview::{PageLoadEvent, WebviewBuilder};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Theme, WebviewUrl,
    WebviewWindowBuilder, WindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::core::{theme_css, Core, Effect, Event};

const SIDEBAR_WIDTH: f64 = 60.0;
const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;
const ACCOUNT_URL: &str = "https://web.whatsapp.com";
const ACCOUNT_LABEL_PREFIX: &str = "account-";
const SIDEBAR_LABEL: &str = "sidebar";
const MAX_SWITCH_SHORTCUTS: usize = 9;
const SETTINGS_LABEL: &str = "settings";
const ZOOM_STEP: f64 = 0.1;
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

#[cfg(not(target_os = "macos"))]
fn account_data_dir(app: &AppHandle, uuid: &uuid::Uuid) -> tauri::Result<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()?
        .join("accounts")
        .join(uuid.to_string()))
}

/// Removes only the disk and memory cache of one account. Cookies, local storage and
/// IndexedDB stay, so the login survives. Sends `()` once when the clear is done or failed.
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
fn on_exit_requested(app: &AppHandle, api: &tauri::ExitRequestApi) {
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
        return;
    }
    api.prevent_exit();
    let app = app.clone();
    std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + CACHE_CLEAR_TIMEOUT;
        for _ in 0..pending {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if rx.recv_timeout(left).is_err() {
                eprintln!("Cache clear did not finish in time; quitting anyway");
                break;
            }
        }
        app.exit(0);
    });
}

/// Deletes the account's data store for good. The account webview is already closed.
fn delete_account_data(app: &AppHandle, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::parse_str(id)?;
    #[cfg(target_os = "macos")]
    {
        let app = app.clone();
        let bytes = *uuid.as_bytes();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = app.remove_data_store(bytes).await {
                eprintln!("Could not remove data store: {e}");
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    match std::fs::remove_dir_all(account_data_dir(app, &uuid)?) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    Ok(())
}

fn add_account_webview(
    app: &AppHandle,
    window: &tauri::Window,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::parse_str(id)?;
    let builder = WebviewBuilder::new(
        account_label(id),
        WebviewUrl::External(ACCOUNT_URL.parse()?),
    )
    .user_agent(USER_AGENT);
    #[cfg(target_os = "macos")]
    let builder = builder.data_store_identifier(*uuid.as_bytes());
    #[cfg(not(target_os = "macos"))]
    let builder = builder.data_directory(account_data_dir(app, &uuid)?);
    #[cfg(target_os = "macos")]
    let _ = app;

    let size = window
        .inner_size()?
        .to_logical::<f64>(window.scale_factor()?);
    window.add_child(
        builder,
        LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
        LogicalSize::new((size.width - SIDEBAR_WIDTH).max(1.0), size.height),
    )?;
    Ok(())
}

fn show_account(app: &AppHandle, id: &str) {
    let active = account_label(id);
    for (label, webview) in app.webviews() {
        if label.starts_with(ACCOUNT_LABEL_PREFIX) {
            let _ = if label == active {
                webview.show()
            } else {
                webview.hide()
            };
        }
    }
}

fn layout(app: &AppHandle, window: &tauri::Window, size: tauri::PhysicalSize<u32>) {
    let size = size.to_logical::<f64>(window.scale_factor().unwrap_or(1.0));
    for (label, webview) in app.webviews() {
        if label == SIDEBAR_LABEL {
            let _ = webview.set_size(LogicalSize::new(SIDEBAR_WIDTH, size.height));
        } else if label.starts_with(ACCOUNT_LABEL_PREFIX) {
            let _ = webview.set_position(LogicalPosition::new(SIDEBAR_WIDTH, 0.0));
            let _ = webview.set_size(LogicalSize::new(
                (size.width - SIDEBAR_WIDTH).max(1.0),
                size.height,
            ));
        }
    }
}

/// Logs the detail, and shows the friendly message on the settings page if it is open.
fn report_error(app: &AppHandle, friendly: &str, detail: &str) {
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

fn run_effects(app: &AppHandle, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::CreateAccountWebview { id } => {
                let Some(window) = app.get_window("main") else {
                    continue;
                };
                if let Err(e) = add_account_webview(app, &window, &id) {
                    eprintln!("Could not create account webview {id}: {e}");
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
                if let Err(e) = delete_account_data(app, &id) {
                    eprintln!("Could not delete data of account {id}: {e}");
                }
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
            // Handled in `on_exit_requested`: it must hold the exit until the clear is done.
            Effect::ClearAccountCaches { .. } => {}
            Effect::RenderSidebar { accounts, active } => {
                let payload = serde_json::json!({ "accounts": accounts, "active": active });
                if let Err(e) = app.emit_to(SIDEBAR_LABEL, "accounts", payload) {
                    eprintln!("Could not update sidebar: {e}");
                }
            }
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
        }
    }
}

fn dispatch(app: &AppHandle, event: Event) {
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
}

#[tauri::command]
fn add_account(app: AppHandle) {
    dispatch(&app, Event::AddAccount { fresh_id: new_id() });
}

#[tauri::command]
fn switch_account(app: AppHandle, id: String) {
    dispatch(&app, Event::SwitchAccount { id });
}

/// The sidebar asks the user to confirm before it calls this.
#[tauri::command]
fn remove_account(app: AppHandle, id: String) {
    dispatch(
        &app,
        Event::RemoveAccount {
            id,
            fresh_id: new_id(),
        },
    );
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = w.set_focus();
        return;
    }
    let result =
        WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App("settings.html".into()))
            .title("Settings")
            .inner_size(420.0, 380.0)
            .build();
    if let Err(e) = result {
        eprintln!("Could not open settings window: {e}");
    }
}

// Allowed for the local settings webview only: see `build.rs` and `capabilities/settings.json`.
#[tauri::command]
fn get_settings(core: tauri::State<Mutex<Core>>) -> serde_json::Value {
    let core = core.lock().expect("core lock");
    serde_json::json!({
        "theme": core.theme(),
        "zoom_factor": core.zoom_factor(),
        "autostart_enabled": core.autostart_enabled(),
        "clear_cache_on_exit": core.clear_cache_on_exit(),
        // Only macOS can clear the cache without removing logins.
        "clear_cache_supported": cfg!(target_os = "macos"),
    })
}

#[tauri::command]
fn set_theme(app: AppHandle, theme: String) {
    dispatch(&app, Event::ThemeChanged { theme });
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

const MENU_SETTINGS: &str = "settings";
const MENU_ZOOM_IN: &str = "zoom-in";
const MENU_ZOOM_OUT: &str = "zoom-out";
const MENU_ZOOM_RESET: &str = "zoom-reset";
const MENU_ADD: &str = "account-add";
const MENU_REMOVE: &str = "account-remove";
const MENU_NEXT: &str = "account-next";
const MENU_PREVIOUS: &str = "account-previous";
const MENU_SWITCH_PREFIX: &str = "account-switch-";

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let app_menu = Submenu::with_items(
        app,
        "WhatsApp",
        true,
        &[
            &PredefinedMenuItem::about(app, None, None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_SETTINGS, "Settings...", true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
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
    let account_menu = Submenu::with_items(
        app,
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
    for n in 1..=MAX_SWITCH_SHORTCUTS {
        account_menu.append(&MenuItem::with_id(
            app,
            format!("{MENU_SWITCH_PREFIX}{n}"),
            format!("Account {n}"),
            true,
            Some(format!("CmdOrCtrl+{n}").as_str()),
        )?)?;
    }
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
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
        // The sidebar knows the active account and asks for confirmation.
        MENU_REMOVE => {
            if let Err(e) = app.emit_to(SIDEBAR_LABEL, "remove-active", ()) {
                eprintln!("Could not ask the sidebar to remove the account: {e}");
            }
        }
        MENU_SETTINGS => open_settings(app),
        MENU_ZOOM_IN => dispatch(app, Event::ZoomStepped { delta: ZOOM_STEP }),
        MENU_ZOOM_OUT => dispatch(app, Event::ZoomStepped { delta: -ZOOM_STEP }),
        MENU_ZOOM_RESET => dispatch(app, Event::ZoomChanged { factor: 1.0 }),
        MENU_NEXT => dispatch(app, Event::SwitchRelative(1)),
        MENU_PREVIOUS => dispatch(app, Event::SwitchRelative(-1)),
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            sidebar_ready,
            add_account,
            switch_account,
            remove_account,
            get_settings,
            set_theme,
            set_zoom,
            set_autostart,
            set_clear_cache_on_exit
        ])
        .menu(build_menu)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .on_page_load(|webview, payload| {
            if payload.event() != PageLoadEvent::Finished
                || !webview.label().starts_with(ACCOUNT_LABEL_PREFIX)
            {
                return;
            }
            let (css, zoom) = {
                let state = webview.app_handle().state::<Mutex<Core>>();
                let core = state.lock().expect("core lock");
                (theme_css(core.theme()), core.zoom_factor())
            };
            inject_css(webview, &css);
            let _ = webview.set_zoom(zoom);
        })
        .setup(|app| {
            let window = WindowBuilder::new(app, "main")
                .title("WhatsApp")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .build()?;
            window.add_child(
                WebviewBuilder::new(SIDEBAR_LABEL, WebviewUrl::App("index.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_WIDTH, WINDOW_HEIGHT),
            )?;

            let raw = match settings_path().map(|p| (std::fs::read_to_string(&p), p)) {
                None => Ok(None),
                Some((Ok(text), _)) => Ok(Some(text)),
                Some((Err(e), _)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Some((Err(e), p)) => Err(format!("Could not read {}: {e}", p.display())),
            };
            let handle = app.handle().clone();
            app.manage(Mutex::new(Core::default()));
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

            let w = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(size) = event {
                    layout(&handle, &w, *size);
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building WhatsApp-QT")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                on_exit_requested(app, &api);
            }
        });
}
