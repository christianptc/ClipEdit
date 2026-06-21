use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Register the global hotkey (replacing any previously registered shortcuts).
/// Pressing it toggles the popup.
pub fn register(app: &AppHandle, accelerator: &str) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    let _ = shortcuts.on_shortcut(accelerator, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            crate::window::toggle_popup(app);
        }
    });
}

pub fn reregister(app: &AppHandle, accelerator: &str) {
    register(app, accelerator);
}
