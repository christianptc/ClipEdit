use crate::history::{HistoryItem, HistoryState};
use crate::settings::{Settings, SettingsState};
use crate::{autostart, clipboard, shortcut, window};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_clipboard_text() -> String {
    clipboard::read_text().unwrap_or_default()
}

#[tauri::command]
pub fn get_history(history: State<HistoryState>) -> Vec<HistoryItem> {
    history.snapshot()
}

#[tauri::command]
pub fn commit_edit(app: AppHandle, new_text: String) {
    clipboard::commit_edit(&app, new_text);
}

#[tauri::command]
pub fn select_history(app: AppHandle, id: u64) {
    let history = app.state::<HistoryState>();
    if let Some(text) = history.select(id) {
        clipboard::write_text(&app, &text);
        let _ = app.emit("clip:changed", ());
    }
}

#[tauri::command]
pub fn delete_history_item(app: AppHandle, id: u64) {
    app.state::<HistoryState>().delete(id);
    let _ = app.emit("clip:changed", ());
}

#[tauri::command]
pub fn clear_history(app: AppHandle) {
    app.state::<HistoryState>().clear();
    let _ = app.emit("clip:changed", ());
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    if let Some(win) = app.get_webview_window(window::POPUP) {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) {
    window::open_settings(&app);
}

#[tauri::command]
pub fn get_settings(settings: State<SettingsState>) -> Settings {
    settings.get()
}

#[tauri::command]
pub fn set_settings(app: AppHandle, mut new_settings: Settings) {
    autostart::set(&app, new_settings.autostart);
    let history = app.state::<HistoryState>();
    history.set_cap(new_settings.history_size);
    history.set_persist(new_settings.persist_history);
    shortcut::reregister(&app, &new_settings.hotkey);
    new_settings.launched_before = true;
    app.state::<SettingsState>().save(new_settings);
    let _ = app.emit("clip:changed", ());
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
