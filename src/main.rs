mod core;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::webview::WebviewBuilder;
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowBuilder, WindowEvent};

use crate::core::{Core, Effect, Event};

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
    let builder = WebviewBuilder::new(account_label(id), WebviewUrl::External(ACCOUNT_URL.parse()?))
        .user_agent(USER_AGENT);
    #[cfg(target_os = "macos")]
    let builder = builder.data_store_identifier(*uuid.as_bytes());
    #[cfg(not(target_os = "macos"))]
    let builder = builder.data_directory(app.path().app_data_dir()?.join("accounts").join(uuid.to_string()));
    #[cfg(target_os = "macos")]
    let _ = app;

    let size = window.inner_size()?.to_logical::<f64>(window.scale_factor()?);
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
            let _ = if label == active { webview.show() } else { webview.hide() };
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
            let _ = webview.set_size(LogicalSize::new((size.width - SIDEBAR_WIDTH).max(1.0), size.height));
        }
    }
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

fn main() {
    tauri::Builder::default()
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

            let raw = settings_path().and_then(|p| std::fs::read_to_string(p).ok());
            let core = Mutex::new(Core::default());
            let effects = core.lock().expect("core lock").handle(Event::SettingsLoaded {
                raw,
                fresh_id: uuid::Uuid::new_v4().to_string(),
            });
            let handle = app.handle().clone();
            run_effects(&handle, &window, effects);
            app.manage(core);

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
