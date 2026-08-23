//! Register/unregister the app to start on login, per platform.
//!
//! Each backend writes (or deletes) one file/registry value and nothing else, so
//! `set_enabled(x)` is idempotent: enabling twice is the same as enabling once.
//! Errors are returned, never logged-and-swallowed, so the caller can surface
//! them through `AppController::save_failed`.

/// Bundle / app id, matching packaging/macos/Info.plist and the Flatpak metainfo.
#[cfg(target_os = "macos")]
const BUNDLE_ID: &str = "io.github.mrdoom.WhatsAppQT";

/// Enable or disable start-on-login for the current executable.
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate own executable: {e}"))?
        .to_string_lossy()
        .to_string();
    platform_set(enabled, &exe)
}

// ── macOS: LaunchAgent plist ────────────────────────────────────────────────
#[cfg(target_os = "macos")]
fn platform_set(enabled: bool, exe: &str) -> Result<(), String> {
    use std::process::Command;

    let plist = dirs::home_dir()
        .ok_or("No home directory")?
        .join("Library/LaunchAgents")
        .join(format!("{BUNDLE_ID}.plist"));

    if !enabled {
        // Unload first: removing the file alone leaves the agent loaded until logout.
        let _ = Command::new("launchctl").arg("unload").arg(&plist).status();
        return match std::fs::remove_file(&plist) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("Cannot remove {}: {e}", plist.display())),
        };
    }

    let contents = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{BUNDLE_ID}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#
    );

    if let Some(parent) = plist.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(&plist, contents).map_err(|e| format!("Cannot write {}: {e}", plist.display()))?;
    // Best-effort: makes the toggle take effect now rather than at next login.
    // A failure here is not fatal, the plist on disk is what login reads.
    let _ = Command::new("launchctl").arg("load").arg(&plist).status();
    Ok(())
}

// ── Linux: XDG autostart .desktop ──────────────────────────────────────────
#[cfg(target_os = "linux")]
fn platform_set(enabled: bool, exe: &str) -> Result<(), String> {
    let entry = dirs::config_dir()
        .ok_or("No config directory")?
        .join("autostart/whatsapp-qt.desktop");

    if !enabled {
        return match std::fs::remove_file(&entry) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("Cannot remove {}: {e}", entry.display())),
        };
    }

    // Fields mirror packaging/WhatsApp-QT.desktop, except Exec, which points at the
    // running binary so a dev build or a non-PATH install autostarts correctly.
    let contents = format!(
        "[Desktop Entry]\n\
         Name=WhatsApp QT\n\
         Comment=WhatsApp desktop client for Linux\n\
         Exec={exe} %u\n\
         Icon=io.github.mrdoom.WhatsAppQT\n\
         Type=Application\n\
         Categories=Network;InstantMessaging;\n\
         StartupNotify=true\n\
         StartupWMClass=WhatsApp-QT\n\
         X-GNOME-Autostart-enabled=true\n"
    );

    if let Some(parent) = entry.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(&entry, contents).map_err(|e| format!("Cannot write {}: {e}", entry.display()))
}

// ── Windows: HKCU Run key ──────────────────────────────────────────────────
// ponytail: shelling out to `reg` keeps the `winreg` dependency out for one
// write and one delete. Swap in winreg if this ever grows real registry needs.
#[cfg(target_os = "windows")]
fn platform_set(enabled: bool, exe: &str) -> Result<(), String> {
    use std::process::Command;

    const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const NAME: &str = "WhatsApp-QT";

    let output = if enabled {
        Command::new("reg")
            .args(["add", KEY, "/v", NAME, "/t", "REG_SZ", "/d", exe, "/f"])
            .output()
    } else {
        Command::new("reg")
            .args(["delete", KEY, "/v", NAME, "/f"])
            .output()
    };

    match output {
        Err(e) => Err(format!("Cannot run reg.exe: {e}")),
        // `reg delete` on a missing value exits non-zero; that is already the
        // desired state, so only report failures when enabling.
        Ok(o) if !o.status.success() && enabled => Err(format!(
            "reg.exe failed: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        )),
        Ok(_) => Ok(()),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_set(_enabled: bool, _exe: &str) -> Result<(), String> {
    Err("Autostart is not supported on this platform".to_string())
}
