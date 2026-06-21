use crate::history::HistoryState;
use crate::sensitive;
use arboard::Clipboard;
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::io;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

/// The last clipboard text ClipEdit itself read or wrote. Used to ignore the
/// monitor's echo of our own writes and to de-dupe repeated reads.
pub struct LastClipboard(pub Mutex<String>);

pub fn read_text() -> Option<String> {
    Clipboard::new().ok()?.get_text().ok()
}

pub fn write_text(app: &AppHandle, text: &str) {
    if let Some(last) = app.try_state::<LastClipboard>() {
        *last.0.lock().unwrap() = text.to_string();
    }
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }
}

/// Save the previous clipboard value and the new (edited) value to history, set
/// the clipboard to the new value, and notify the UI.
pub fn commit_edit(app: &AppHandle, new_text: String) {
    let history = app.state::<HistoryState>();
    if let Some(prev) = read_text() {
        if !prev.is_empty() && prev != new_text {
            history.add(prev);
        }
    }
    write_text(app, &new_text);
    history.add(new_text);
    let _ = app.emit("clip:changed", ());
}

struct Handler {
    app: AppHandle,
}

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        if sensitive::is_sensitive() {
            return CallbackResult::Next;
        }
        if let Some(text) = read_text() {
            if text.is_empty() {
                return CallbackResult::Next;
            }
            {
                let last = self.app.state::<LastClipboard>();
                let mut guard = last.0.lock().unwrap();
                if *guard == text {
                    return CallbackResult::Next; // our own write or a repeat
                }
                *guard = text.clone();
            }
            if self.app.state::<HistoryState>().add(text) {
                let _ = self.app.emit("clip:changed", ());
            }
        }
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, _error: io::Error) -> CallbackResult {
        CallbackResult::Next
    }
}

/// Spawn the OS-native clipboard change monitor on a background thread.
pub fn start_monitor(app: AppHandle) {
    std::thread::spawn(move || {
        if let Ok(mut master) = Master::new(Handler { app }) {
            let _ = master.run();
        }
    });
}
