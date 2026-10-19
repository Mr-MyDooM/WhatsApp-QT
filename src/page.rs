//! The account page: everything the app sends to a WhatsApp Web page. No Tauri types.
//! One `PageConfig` carries every page setting. It reaches the page as the init script at
//! load and as the live update on each change.

use serde::Serialize;

/// The JS side of the contract. These names appear here and in `page_shim.js` only.
const CONFIG_GLOBAL: &str = "__PAIRLINE_CONFIG__";
const APPLY_ENTRY: &str = "__pairlineApply";
const BOOTSTRAP_SCRIPT: &str = include_str!("page_shim.js");

/// Every setting that changes the account page.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageConfig {
    /// Empty removes any injected theme.
    pub theme_css: String,
    pub zoom: f64,
    pub spellcheck: bool,
    /// `None` is WhatsApp's own width.
    pub chat_list_width: Option<f64>,
    pub chat_list_min: f64,
    pub chat_list_max: f64,
}

/// Script that runs before the page: sets the config and defines the apply entry.
pub fn init_script(config: &PageConfig) -> String {
    let json = serde_json::to_string(config).expect("page config serializes");
    format!("window.{CONFIG_GLOBAL} = {json};\n{BOOTSTRAP_SCRIPT}")
}

/// Script that gives a loaded page a new config.
pub fn apply_script(config: &PageConfig) -> String {
    let json = serde_json::to_string(config).expect("page config serializes");
    format!("window.{APPLY_ENTRY} && window.{APPLY_ENTRY}({json});")
}

const DARK_BASE_CSS: &str = r#"
    /* Dark theme base styles */
    body, #app, [data-testid="conversation-panel-body"] {
        background-color: #1e1e1e !important;
        color: #ffffff !important;
        font-family: 'Fira Code', 'JetBrains Mono', 'monospace' !important;
    }
"#;
const DARK_CHAT_CSS: &str = r#"
    /* Dark theme chat styles */
    [data-testid="chat-list"] {
        background-color: #2d2d2d !important;
    }
    [data-testid="chat"] {
        background-color: #3d3d3d !important;
        border-bottom: 1px solid #555 !important;
    }
"#;
const LIGHT_BASE_CSS: &str = r#"
    /* Light theme base styles */
    body, #app, [data-testid="conversation-panel-body"] {
        background-color: #ffffff !important;
        color: #000000 !important;
        font-family: 'Fira Code', 'JetBrains Mono', 'monospace' !important;
    }
"#;
const LIGHT_CHAT_CSS: &str = r#"
    /* Light theme chat styles */
    [data-testid="chat-list"] {
        background-color: #f8f9fa !important;
    }
    [data-testid="chat"] {
        background-color: #ffffff !important;
        border-bottom: 1px solid #e9ecef !important;
    }
"#;

/// CSS to inject into account webviews. "system" and unknown names inject nothing.
pub fn theme_css(theme: &str) -> String {
    let mut css = String::new();
    if theme == "dark" {
        css.push_str(DARK_BASE_CSS);
        css.push_str(DARK_CHAT_CSS);
        css.push_str(
            r#"
                /* Additional dark theme styles */
                ._3OtEr, .app, #app, .app-wrapper-web, [data-testid="app-wrapper"] {
                    background-color: #1e1e1e !important;
                }
                ._3j7s9 {
                    background-color: #2d2d2d !important;
                }
                /* WhatsApp specific dark theme */
                [data-testid="chatlist-header"], [data-testid="chat-header"] {
                    background-color: #2d2d2d !important;
                }
            "#,
        );
    } else if theme == "light" {
        css.push_str(LIGHT_BASE_CSS);
        css.push_str(LIGHT_CHAT_CSS);
        css.push_str(
            r#"
                /* Additional light theme styles */
                ._3OtEr, .app, #app, .app-wrapper-web, [data-testid="app-wrapper"] {
                    background-color: #ffffff !important;
                }
                ._3j7s9 {
                    background-color: #f8f9fa !important;
                }
            "#,
        );
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> PageConfig {
        PageConfig {
            theme_css: "body{color:red}".into(),
            zoom: 1.5,
            spellcheck: false,
            chat_list_width: Some(320.0),
            chat_list_min: 200.0,
            chat_list_max: 900.0,
        }
    }

    const CONFIG_JSON: &str = r#"{"theme_css":"body{color:red}","zoom":1.5,"spellcheck":false,"chat_list_width":320.0,"chat_list_min":200.0,"chat_list_max":900.0}"#;

    #[test]
    fn init_script_holds_the_config_as_json() {
        let js = init_script(&config());
        assert!(js.starts_with(&format!("window.__PAIRLINE_CONFIG__ = {CONFIG_JSON};")));
        assert!(js.contains("window.__pairlineApply = "));
    }

    #[test]
    fn apply_script_holds_the_config_as_json() {
        assert_eq!(
            apply_script(&config()),
            format!("window.__pairlineApply && window.__pairlineApply({CONFIG_JSON});")
        );
    }

    #[test]
    fn unset_chat_list_width_is_null() {
        let mut c = config();
        c.chat_list_width = None;
        assert!(apply_script(&c).contains(r#""chat_list_width":null"#));
    }

    #[test]
    fn theme_css_covers_each_theme() {
        let dark = theme_css("dark");
        assert!(dark.contains("#1e1e1e"), "dark theme must set its base bg");
        assert!(
            dark.contains("chatlist-header"),
            "dark adds WhatsApp overrides"
        );

        let light = theme_css("light");
        assert!(
            light.contains("#ffffff"),
            "light theme must set its base bg"
        );
        assert!(
            !light.contains("#1e1e1e"),
            "light must not leak dark colors"
        );

        // "system" (and anything unrecognised) means "inject nothing".
        assert!(theme_css("system").is_empty());
        assert!(theme_css("").is_empty());
    }
}
