// SPDX-License-Identifier: Apache-2.0

//! The optional tray icon (off by default; Settings › General).
//!
//! When it is on: DeeCure starts at login with only the tray icon, closing the window
//! keeps DeeCure in the tray, and the tray menu opens the window or quits.
//!
//! Worst cases handled:
//! - A desktop without a tray area (GNOME without the AppIndicator extension) would hide
//!   the window with no way back, so the tray is only used when a tray host
//!   (`org.kde.StatusNotifierWatcher`) is running. Otherwise the window stays as usual.
//! - Opening DeeCure from the app menu while it sits in the tray shows the running
//!   window instead of starting a second copy (a small socket, [`instance`]).
//! - The login entry is a file DeeCure writes and marks; DeeCure removes only its own.

use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

/// True while the tray icon is shown, so closing the window only hides it.
pub static ACTIVE: AtomicBool = AtomicBool::new(false);
const TRAY_ID: &str = "main";
/// Marks the login entry as DeeCure's own.
const MARK: &str = "X-DeeCure-Tray=true";

/// The desktop app's own settings, in `$XDG_CONFIG_HOME/deecure/desktop.json`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopSettings {
    /// Show the tray icon.
    pub tray: bool,
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|h| !h.is_empty())
                .map(|h| PathBuf::from(h).join(".config"))
        })
}

fn settings_path() -> Option<PathBuf> {
    Some(config_dir()?.join("deecure").join("desktop.json"))
}

/// The settings, or the defaults when the file is missing or unreadable.
pub fn load() -> DesktopSettings {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save(settings: &DesktopSettings) -> Result<(), String> {
    let path = settings_path().ok_or("HOME is not set")?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    write_private(&path, &text)
}

/// Writes through a temporary file in a private folder, so a crash never leaves half a file.
fn write_private(path: &Path, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no folder")?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    let tmp = dir.join(format!(".{}.tmp", std::process::id()));
    let result = std::fs::write(&tmp, text).and_then(|()| std::fs::rename(&tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result.map_err(|e| format!("{}: {e}", path.display()))
}

/// True when the desktop has a tray area. Asking never starts anything.
pub async fn host_available() -> bool {
    let Ok(conn) = zbus::Connection::session().await else {
        return false;
    };
    let reply = conn
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &("org.kde.StatusNotifierWatcher",),
        )
        .await;
    reply
        .ok()
        .and_then(|r| r.body().deserialize::<bool>().ok())
        .unwrap_or(false)
}

/// Shows and focuses the main window.
pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Puts the tray icon up.
pub fn create(app: &AppHandle) -> Result<(), String> {
    if app.tray_by_id(TRAY_ID).is_some() {
        ACTIVE.store(true, Ordering::SeqCst);
        return Ok(());
    }
    let open = MenuItem::with_id(app, "open", "Open DeeCure", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit DeeCure", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&open, &quit]).map_err(|e| e.to_string())?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("DeeCure")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app).map_err(|e| e.to_string())?;
    ACTIVE.store(true, Ordering::SeqCst);
    Ok(())
}

/// Takes the tray icon down.
pub fn remove(app: &AppHandle) {
    ACTIVE.store(false, Ordering::SeqCst);
    let _ = app.remove_tray_by_id(TRAY_ID);
}

fn autostart_path() -> Option<PathBuf> {
    Some(
        config_dir()?
            .join("autostart")
            .join(format!("{}.desktop", deecure_core::APP_ID)),
    )
}

/// Starts DeeCure in the tray at login (on), or stops that (off). A login entry with the
/// same name that DeeCure did not write is left alone.
fn set_autostart(on: bool) -> Result<(), String> {
    let path = autostart_path().ok_or("HOME is not set")?;
    let ours = std::fs::read_to_string(&path).map(|t| t.contains(MARK));
    match (on, ours) {
        (true, Ok(false)) | (false, Ok(false)) => Ok(()), // someone else's file
        (true, _) => {
            let text = format!(
                "[Desktop Entry]\nType=Application\nName=DeeCure\nComment=Shows DeeCure in the tray\nExec=deecure-desktop --hidden\nIcon={}\nX-GNOME-Autostart-enabled=true\nNoDisplay=true\n{MARK}\n",
                deecure_core::APP_ID
            );
            write_private(&path, &text)?;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
            Ok(())
        }
        (false, Ok(true)) => std::fs::remove_file(&path).map_err(|e| e.to_string()),
        (false, Err(_)) => Ok(()),
    }
}

/// What Settings shows.
#[derive(Debug, Serialize)]
pub struct TrayState {
    /// The setting.
    pub enabled: bool,
    /// The icon is shown right now.
    pub active: bool,
    /// This desktop has a tray area.
    pub available: bool,
}

/// The tray setting and whether it can work here.
#[tauri::command]
pub async fn tray_state() -> TrayState {
    TrayState {
        enabled: load().tray,
        active: ACTIVE.load(Ordering::SeqCst),
        available: host_available().await,
    }
}

/// Turns the tray on or off. Turning it on is refused when this desktop has no tray area.
#[tauri::command]
pub async fn set_tray(app: AppHandle, on: bool) -> Result<TrayState, String> {
    if on && !host_available().await {
        return Err("this desktop has no tray area (on GNOME it needs the AppIndicator extension, which Ubuntu turns on by default)".into());
    }
    if on {
        create(&app)?;
    } else {
        remove(&app);
    }
    let mut settings = load();
    settings.tray = on;
    save(&settings)?;
    if let Err(e) = set_autostart(on) {
        eprintln!("deecure-desktop: could not change the login entry: {e}");
    }
    Ok(tray_state().await)
}

/// One running copy of the app: a second start asks the first to show its window.
pub mod instance {
    use super::*;
    use std::os::unix::net::{UnixListener, UnixStream};

    fn socket_path() -> Option<PathBuf> {
        let dir = std::env::var_os("XDG_RUNTIME_DIR").filter(|d| !d.is_empty())?;
        Some(PathBuf::from(dir).join("deecure").join("desktop.sock"))
    }

    /// True when another copy runs (it was asked to show its window).
    pub fn signal_running() -> bool {
        let Some(path) = socket_path() else {
            return false;
        };
        match UnixStream::connect(&path) {
            Ok(mut stream) => stream.write_all(b"show\n").is_ok(),
            Err(_) => false,
        }
    }

    /// Listens for later starts and shows the window when one asks.
    pub fn listen(app: AppHandle) {
        let Some(path) = socket_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir);
        }
        // Nobody answered, so a socket file left here is from a copy that crashed.
        let _ = std::fs::remove_file(&path);
        let Ok(listener) = UnixListener::bind(&path) else {
            return;
        };
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut word = [0u8; 5];
                let mut stream = stream;
                // A client that says nothing must not block later starts.
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
                if stream.read_exact(&mut word).is_ok() && &word == b"show\n" {
                    let handle = app.clone();
                    let _ = app.run_on_main_thread(move || show(&handle));
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_off_and_read_partial_files() {
        assert!(!DesktopSettings::default().tray);
        let s: DesktopSettings = serde_json::from_str("{}").unwrap_or_default();
        assert!(!s.tray);
        let s: DesktopSettings =
            serde_json::from_str(r#"{"tray": true, "later": 1}"#).unwrap_or_default();
        assert!(s.tray);
    }
}
