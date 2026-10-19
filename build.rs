fn main() {
    // With an app manifest, each command needs a permission. Capability files grant
    // them to local webviews only, so remote pages cannot call them.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "sidebar_ready",
            "add_account",
            "switch_account",
            "remove_account",
            "show_account_menu",
            "rename_account",
            "cancel_rename",
            "open_rename_account",
            "retry_account",
            "get_settings",
            "notification_permission",
            "request_notification_permission",
            "open_notification_settings",
            "set_theme",
            "set_zoom",
            "set_notifications",
            "notify",
            "set_autostart",
            "set_clear_cache_on_exit",
            "set_close_to_tray",
            "set_start_minimized",
        ]),
    ))
    .expect("failed to run tauri-build");
}
