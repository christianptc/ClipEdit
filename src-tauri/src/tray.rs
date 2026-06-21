use crate::{autostart, history::HistoryState, settings::SettingsState, window};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", "Open ClipEdit", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let autostart_on = app.state::<SettingsState>().get().autostart;
    let autostart_i = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start on login",
        true,
        autostart_on,
        None::<&str>,
    )?;
    let clear_i = MenuItem::with_id(app, "clear", "Clear history", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit ClipEdit", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_i,
            &settings_i,
            &autostart_i,
            &sep1,
            &clear_i,
            &sep2,
            &quit_i,
        ],
    )?;

    let autostart_handle = autostart_i.clone();
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default window icon");

    let _tray = TrayIconBuilder::with_id("clipedit-tray")
        .icon(icon)
        .tooltip("ClipEdit")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => window::show_popup(app),
            "settings" => window::open_settings(app),
            "clear" => {
                app.state::<HistoryState>().clear();
                let _ = app.emit("clip:changed", ());
            }
            "autostart" => {
                let enabled = !autostart_handle.is_checked().unwrap_or(false);
                let _ = autostart_handle.set_checked(enabled);
                autostart::set(app, enabled);
                let settings = app.state::<SettingsState>();
                let mut current = settings.get();
                current.autostart = enabled;
                settings.save(current);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click brings up the popup; right-click shows the menu.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::show_popup(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
