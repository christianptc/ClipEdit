use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_CAP: usize = 100;

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: u64,
    pub text: String,
    pub created_at: u64,
}

/// In-memory clipboard history with bounded size, dedup, and atomic JSON persistence.
pub struct HistoryState {
    items: Mutex<VecDeque<HistoryItem>>,
    path: PathBuf,
    cap: AtomicUsize,
    persist: AtomicBool,
    next_id: AtomicU64,
}

impl HistoryState {
    pub fn load(path: PathBuf, cap: usize, persist: bool) -> Self {
        let items: VecDeque<HistoryItem> = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<HistoryItem>>(&s).ok())
            .map(VecDeque::from)
            .unwrap_or_default();
        let max_id = items.iter().map(|i| i.id).max().unwrap_or(0);
        Self {
            items: Mutex::new(items),
            path,
            cap: AtomicUsize::new(cap.max(1)),
            persist: AtomicBool::new(persist),
            next_id: AtomicU64::new(max_id + 1),
        }
    }

    pub fn snapshot(&self) -> Vec<HistoryItem> {
        self.items.lock().unwrap().iter().cloned().collect()
    }

    /// Add text to the front. If it already exists it is moved to the front
    /// instead of duplicated. Returns true if the history changed.
    pub fn add(&self, text: String) -> bool {
        if text.is_empty() {
            return false;
        }
        {
            let mut items = self.items.lock().unwrap();
            if let Some(pos) = items.iter().position(|i| i.text == text) {
                if pos == 0 {
                    return false; // already the most recent entry
                }
                items.remove(pos);
            }
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            items.push_front(HistoryItem {
                id,
                text,
                created_at: now_millis(),
            });
            let cap = self.cap.load(Ordering::Relaxed);
            while items.len() > cap {
                items.pop_back();
            }
        }
        self.save();
        true
    }

    /// Move an existing entry to the front and return its text.
    pub fn select(&self, id: u64) -> Option<String> {
        let text = {
            let mut items = self.items.lock().unwrap();
            let pos = items.iter().position(|i| i.id == id)?;
            let item = items.remove(pos)?;
            let text = item.text.clone();
            items.push_front(item);
            text
        };
        self.save();
        Some(text)
    }

    pub fn delete(&self, id: u64) {
        {
            let mut items = self.items.lock().unwrap();
            items.retain(|i| i.id != id);
        }
        self.save();
    }

    pub fn clear(&self) {
        self.items.lock().unwrap().clear();
        self.save();
    }

    pub fn set_cap(&self, cap: usize) {
        let cap = cap.max(1);
        self.cap.store(cap, Ordering::Relaxed);
        {
            let mut items = self.items.lock().unwrap();
            while items.len() > cap {
                items.pop_back();
            }
        }
        self.save();
    }

    pub fn set_persist(&self, persist: bool) {
        self.persist.store(persist, Ordering::Relaxed);
        if persist {
            self.save();
        } else {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn save(&self) {
        if !self.persist.load(Ordering::Relaxed) {
            return;
        }
        let items: Vec<HistoryItem> = self.items.lock().unwrap().iter().cloned().collect();
        if let Ok(json) = serde_json::to_string(&items) {
            // Atomic write: temp file + rename, so a crash can't corrupt history.
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, json).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
