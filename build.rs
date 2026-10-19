fn main() {
    // With an app manifest, each command needs a permission. Capability files grant
    // them to local webviews only, so remote pages cannot call them.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "sidebar_ready",
            "toggle_sidebar",
            "reveal_sidebar",
            "open_settings_window",
            "add_account",
            "switch_account",
            "remove_account",
            "show_account_menu",
            "rename_account",
            "cancel_rename",
            "open_rename_account",
            "retry_account",
            "get_settings",
            "update_setting",
            "notification_permission",
            "request_notification_permission",
            "open_notification_settings",
            "send_test_notification",
            "snooze_notifications",
            "end_snooze",
            "choose_sound_file",
            "choose_download_folder",
            "get_about",
            "open_repository",
            "notify",
            "profile_picture",
            "set_chat_list_width",
            "get_avatars",
        ]),
    ))
    .expect("failed to run tauri-build");
}
