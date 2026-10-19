mod core;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem, MenuItemKind, PredefinedMenuItem};
use tauri::webview::{PageLoadEvent, WebviewBuilder};
use tauri::{
    Emitter, LogicalPosition, LogicalSize, Manager, Theme, WebviewUrl, WebviewWindowBuilder,
    WindowBuilder, WindowEvent,
};

use crate::core::{theme_css, Core, Effect, Event};

const SETTINGS_LABEL: &str = "settings";
const ZOOM_STEP: f64 = 0.1;
const SAVE_FAILED_MESSAGE: &str =
    "Could not save your settings. The change is active until you quit the app.";

const SIDEBAR_WIDTH: f64 = 60.0;
const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;
const ACCOUNT_URL: &str = "https://web.whatsapp.com";
const ACCOUNT_LABEL_PREFIX: &str = "account-";
/// WhatsApp Web rejects the default WKWebView user agent ("works with Safari 15+").
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("WhatsAppDesktop").join("settings.json"))
}

fn account_label(id: &str) -> String {
    format!("{ACCOUNT_LABEL_PREFIX}{id}")
}

fn add_account_webview(
    app: &tauri::AppHandle,
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
    let builder = builder.data_directory(
        app.path()
            .app_data_dir()?
            .join("accounts")
            .join(uuid.to_string()),
    );
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

fn show_account(app: &tauri::AppHandle, id: &str) {
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

fn layout(app: &tauri::AppHandle, window: &tauri::Window, size: tauri::PhysicalSize<u32>) {
    let size = size.to_logical::<f64>(window.scale_factor().unwrap_or(1.0));
    for (label, webview) in app.webviews() {
        if label == "sidebar" {
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
fn report_error(app: &tauri::AppHandle, friendly: &str, detail: &str) {
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

/// Runs core effects for a change that comes from a menu item or a settings command.
fn dispatch(app: &tauri::AppHandle, event: Event) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let effects = app
        .state::<Mutex<Core>>()
        .lock()
        .expect("core lock")
        .handle(event);
    run_effects(app, &window, effects);
}

fn open_settings(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = w.set_focus();
        return;
    }
    let result =
        WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App("settings.html".into()))
            .title("Settings")
            .inner_size(420.0, 300.0)
            .build();
    if let Err(e) = result {
        eprintln!("Could not open settings window: {e}");
    }
}

#[tauri::command]
fn get_settings(core: tauri::State<Mutex<Core>>) -> serde_json::Value {
    let core = core.lock().expect("core lock");
    serde_json::json!({ "theme": core.theme(), "zoom_factor": core.zoom_factor() })
}

#[tauri::command]
fn set_theme(app: tauri::AppHandle, theme: String) {
    dispatch(&app, Event::ThemeChanged { theme });
}

#[tauri::command]
fn set_zoom(app: tauri::AppHandle, factor: f64) {
    dispatch(&app, Event::ZoomChanged { factor });
}

/// Default menu plus Settings (Cmd+,) in the app menu and zoom items in View.
fn build_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::default(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, Some("CmdOrCtrl+,"))?;
    let items = menu.items()?;
    if let Some(MenuItemKind::Submenu(app_menu)) = items.first() {
        app_menu.insert(&settings, 1)?;
        app_menu.insert(&PredefinedMenuItem::separator(app)?, 2)?;
    }
    let view = items
        .iter()
        .filter_map(MenuItemKind::as_submenu)
        .find(|m| m.text().is_ok_and(|t| t == "View"));
    if let Some(view) = view {
        for (id, label, key) in [
            ("zoom-in", "Zoom In", "CmdOrCtrl+="),
            ("zoom-out", "Zoom Out", "CmdOrCtrl+-"),
            ("zoom-reset", "Actual Size", "CmdOrCtrl+0"),
        ] {
            view.append(&MenuItem::with_id(app, id, label, true, Some(key))?)?;
        }
    }
    Ok(menu)
}

fn run_effects(app: &tauri::AppHandle, window: &tauri::Window, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::CreateAccountWebview { id } => {
                if let Err(e) = add_account_webview(app, window, &id) {
                    eprintln!("Could not create account webview {id}: {e}");
                }
            }
            Effect::ShowAccount { id } => show_account(app, &id),
            Effect::ApplyTheme { theme, css } => {
                let scheme = match theme.as_str() {
                    "dark" => Some(Theme::Dark),
                    "light" => Some(Theme::Light),
                    _ => None,
                };
                let _ = window.set_theme(scheme);
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings, set_theme, set_zoom])
        .menu(build_menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => open_settings(app),
            "zoom-in" => dispatch(app, Event::ZoomStepped { delta: ZOOM_STEP }),
            "zoom-out" => dispatch(app, Event::ZoomStepped { delta: -ZOOM_STEP }),
            "zoom-reset" => dispatch(app, Event::ZoomChanged { factor: 1.0 }),
            _ => {}
        })
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
                WebviewBuilder::new("sidebar", WebviewUrl::App("index.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_WIDTH, WINDOW_HEIGHT),
            )?;

            let raw = match settings_path().map(|p| (std::fs::read_to_string(&p), p)) {
                None => Ok(None),
                Some((Ok(text), _)) => Ok(Some(text)),
                Some((Err(e), _)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Some((Err(e), p)) => Err(format!("Could not read {}: {e}", p.display())),
            };
            let core = Mutex::new(Core::default());
            let effects = core
                .lock()
                .expect("core lock")
                .handle(Event::SettingsLoaded {
                    raw,
                    fresh_id: uuid::Uuid::new_v4().to_string(),
                });
            let handle = app.handle().clone();
            run_effects(&handle, &window, effects);
            app.manage(core);
            let appearance = app
                .state::<Mutex<Core>>()
                .lock()
                .expect("core lock")
                .appearance_effects();
            run_effects(&handle, &window, appearance);

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
