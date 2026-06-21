mod autostart;
mod clipboard;
mod commands;
mod history;
mod sensitive;
mod settings;
mod shortcut;
mod tray;
mod window;

use clipboard::LastClipboard;
use history::HistoryState;
use settings::SettingsState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_clipboard_text,
            commands::get_history,
            commands::commit_edit,
            commands::select_history,
            commands::delete_history_item,
            commands::clear_history,
            commands::hide_popup,
            commands::open_settings_window,
            commands::get_settings,
            commands::set_settings,
            commands::quit_app,
        ])
        .setup(|app| {
            // macOS: run as a menu-bar accessory (no Dock icon). The bundled app
            // also sets LSUIElement via Info.plist; this covers `tauri dev`.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();

            let data_dir = app.path().app_data_dir().expect("resolve app data dir");
            let _ = std::fs::create_dir_all(&data_dir);
            let config_dir = app.path().app_config_dir().expect("resolve app config dir");
            let _ = std::fs::create_dir_all(&config_dir);

            let settings_state = SettingsState::load(config_dir.join("settings.json"));
            let current = settings_state.get();

            let history_state = HistoryState::load(
                data_dir.join("history.json"),
                current.history_size,
                current.persist_history,
            );

            app.manage(LastClipboard(Mutex::new(String::new())));
            app.manage(history_state);
            app.manage(settings_state);

            // Reconcile OS autostart with the saved setting.
            autostart::set(&handle, current.autostart);

            tray::build_tray(&handle)?;
            shortcut::register(&handle, &current.hotkey);
            clipboard::start_monitor(handle.clone());

            // First launch: persist the flag so onboarding shows only once, then
            // open settings as a small onboarding step.
            if !current.launched_before {
                let settings = handle.state::<SettingsState>();
                let mut first = settings.get();
                first.launched_before = true;
                settings.save(first);
                window::open_settings(&handle);
            }

            Ok(())
        })
        .on_window_event(|win, event| {
            // Auto-dismiss the popup when it loses focus (click-away to close).
            if win.label() == window::POPUP {
                if let tauri::WindowEvent::Focused(focused) = event {
                    if !*focused {
                        let _ = win.hide();
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building ClipEdit")
        .run(|_app, event| {
            // Tray app: don't quit when the last window closes; only the tray
            // "Quit" action (app.exit) ends the process.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
