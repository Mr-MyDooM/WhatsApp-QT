fn main() {
    // Listing the commands makes Tauri enforce capabilities on them, so only
    // webviews named in capabilities/settings.json can call them.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["get_settings", "set_theme", "set_zoom"]),
    ))
    .expect("failed to run tauri-build");
}
