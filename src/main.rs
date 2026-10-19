mod core;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::webview::WebviewBuilder;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowBuilder,
    WindowEvent,
};

use crate::core::{Core, Effect, Event};

const SIDEBAR_WIDTH: f64 = 60.0;
const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;
const ACCOUNT_URL: &str = "https://web.whatsapp.com";
const ACCOUNT_LABEL_PREFIX: &str = "account-";
const SIDEBAR_LABEL: &str = "sidebar";
const MAX_SWITCH_SHORTCUTS: usize = 9;
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
            Effect::RenderSidebar { accounts, active } => {
                let payload = serde_json::json!({ "accounts": accounts, "active": active });
                if let Err(e) = app.emit_to(SIDEBAR_LABEL, "accounts", payload) {
                    eprintln!("Could not update sidebar: {e}");
                }
            }
            Effect::SaveSettings { json } => {
                let Some(path) = settings_path() else {
                    eprintln!("Could not find config directory; settings not saved");
                    continue;
                };
                let result = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, json));
                if let Err(e) = result {
                    eprintln!("Could not save settings to {}: {e}", path.display());
                }
            }
            Effect::ReportError { message } => eprintln!("{message}"),
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
    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[&PredefinedMenuItem::minimize(app, None)?],
    )?;
    Menu::with_items(app, &[&app_menu, &edit_menu, &account_menu, &window_menu])
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
        .invoke_handler(tauri::generate_handler![
            sidebar_ready,
            add_account,
            switch_account,
            remove_account
        ])
        .menu(build_menu)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
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

            let w = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(size) = event {
                    layout(&handle, &w, *size);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WhatsApp-QT");
}
