fn main() {
    // With an app manifest, each command needs a permission. The capability file grants
    // them to the local sidebar webview only, so remote pages cannot call them.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "sidebar_ready",
            "add_account",
            "switch_account",
            "remove_account",
        ]),
    ))
    .expect("failed to run tauri-build");
}
