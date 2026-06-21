use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const POPUP: &str = "popup";
pub const SETTINGS: &str = "settings";

/// Show (creating on first use) and focus the Spotlight-style popup, then ask
/// the UI to refresh. Window creation is marshalled to the main thread, which
/// macOS requires.
pub fn show_popup(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let win = get_or_build_popup(&handle);
        let _ = win.center();
        let _ = win.show();
        let _ = win.set_focus();
        let _ = handle.emit("clip:refresh", ());
    });
}

pub fn toggle_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP) {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            return;
        }
    }
    show_popup(app);
}

fn get_or_build_popup(app: &AppHandle) -> WebviewWindow {
    if let Some(win) = app.get_webview_window(POPUP) {
        return win;
    }
    let win = WebviewWindowBuilder::new(app, POPUP, WebviewUrl::App("popup.html".into()))
        .title("ClipEdit")
        .inner_size(680.0, 460.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(true)
        .center()
        .build()
        .expect("failed to build popup window");
    apply_glass(&win);
    win
}

/// Apply the native "liquid glass" backdrop so the desktop blurs through the
/// popup (macOS vibrancy / Windows acrylic). Rounded to match the CSS corners.
#[allow(unused_variables)]
fn apply_glass(win: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
        let _ = apply_vibrancy(
            win,
            NSVisualEffectMaterial::HudWindow,
            Some(NSVisualEffectState::Active),
            Some(12.0),
        );
    }
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::apply_acrylic;
        let _ = apply_acrylic(win, Some((18, 18, 20, 160)));
    }
}

/// Open (creating on first use) the settings window.
pub fn open_settings(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let win = handle.get_webview_window(SETTINGS).unwrap_or_else(|| {
            WebviewWindowBuilder::new(&handle, SETTINGS, WebviewUrl::App("settings.html".into()))
                .title("ClipEdit Settings")
                .inner_size(460.0, 600.0)
                .resizable(false)
                .visible(false)
                .center()
                .build()
                .expect("failed to build settings window")
        });
        let _ = win.show();
        let _ = win.set_focus();
        let _ = handle.emit("settings:refresh", ());
    });
}
