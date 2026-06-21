const { invoke } = window.__TAURI__.core;

const els = {
  autostart: document.getElementById("autostart"),
  hotkey: document.getElementById("hotkey"),
  historySize: document.getElementById("historySize"),
  persist: document.getElementById("persist"),
  save: document.getElementById("save"),
  clear: document.getElementById("clear"),
  quit: document.getElementById("quit"),
};

async function load() {
  const s = await invoke("get_settings");
  els.autostart.checked = !!s.autostart;
  els.hotkey.value = s.hotkey || "CmdOrCtrl+B";
  els.historySize.value = s.history_size || 100;
  els.persist.checked = s.persist_history !== false;
}

function flash(button, text) {
  const original = button.dataset.label || button.textContent;
  button.dataset.label = original;
  button.textContent = text;
  setTimeout(() => (button.textContent = original), 1200);
}

els.save.addEventListener("click", async () => {
  const newSettings = {
    autostart: els.autostart.checked,
    hotkey: (els.hotkey.value || "").trim() || "CmdOrCtrl+B",
    history_size: Math.min(1000, Math.max(1, parseInt(els.historySize.value, 10) || 100)),
    persist_history: els.persist.checked,
    launched_before: true,
  };
  await invoke("set_settings", { newSettings });
  flash(els.save, "Saved ✓");
});

els.clear.addEventListener("click", async () => {
  await invoke("clear_history");
  flash(els.clear, "Cleared ✓");
});

els.quit.addEventListener("click", async () => {
  await invoke("quit_app");
});

load();
