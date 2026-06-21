use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub autostart: bool,
    pub hotkey: String,
    pub history_size: usize,
    pub persist_history: bool,
    pub launched_before: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            hotkey: "CmdOrCtrl+B".to_string(),
            history_size: crate::history::DEFAULT_CAP,
            persist_history: true,
            launched_before: false,
        }
    }
}

pub struct SettingsState {
    inner: Mutex<Settings>,
    path: PathBuf,
}

impl SettingsState {
    pub fn load(path: PathBuf) -> Self {
        let inner = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Settings>(&t).ok())
            .unwrap_or_default();
        Self {
            inner: Mutex::new(inner),
            path,
        }
    }

    pub fn get(&self) -> Settings {
        self.inner.lock().unwrap().clone()
    }

    pub fn save(&self, settings: Settings) {
        *self.inner.lock().unwrap() = settings.clone();
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, json).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }
}
