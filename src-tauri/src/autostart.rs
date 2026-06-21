use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Enable or disable launching ClipEdit on login (registry Run key on Windows,
/// LaunchAgent on macOS) via the autostart plugin.
pub fn set(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let _ = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
}
