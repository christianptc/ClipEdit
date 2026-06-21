const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const currentEl = document.getElementById("current");
const filterEl = document.getElementById("filter");
const historyEl = document.getElementById("history");
const countEl = document.getElementById("count");

let items = [];

function preview(text) {
  return text.replace(/\s+/g, " ").trim();
}

function render() {
  const query = filterEl.value.toLowerCase();
  historyEl.innerHTML = "";
  let shown = 0;

  for (const item of items) {
    if (query && !item.text.toLowerCase().includes(query)) continue;
    shown++;

    const li = document.createElement("li");
    li.className = "item";

    const span = document.createElement("span");
    span.className = "text";
    span.textContent = preview(item.text);
    span.title = item.text;
    li.appendChild(span);

    const del = document.createElement("button");
    del.className = "del";
    del.textContent = "×";
    del.title = "Remove from history";
    del.addEventListener("click", async (e) => {
      e.stopPropagation();
      await invoke("delete_history_item", { id: item.id });
    });
    li.appendChild(del);

    li.addEventListener("click", async () => {
      await invoke("select_history", { id: item.id });
      await hide();
    });

    historyEl.appendChild(li);
  }

  if (shown === 0) {
    const empty = document.createElement("div");
    empty.id = "empty";
    empty.textContent = query ? "No matches" : "No clipboard history yet";
    historyEl.appendChild(empty);
  }

  countEl.textContent = items.length
    ? `${items.length} item${items.length > 1 ? "s" : ""}`
    : "";
}

async function loadHistory() {
  items = (await invoke("get_history")) || [];
  render();
}

async function refresh() {
  const [text, history] = await Promise.all([
    invoke("get_clipboard_text"),
    invoke("get_history"),
  ]);
  currentEl.value = text || "";
  items = history || [];
  filterEl.value = "";
  render();
  currentEl.focus();
  currentEl.select();
}

async function commit() {
  await invoke("commit_edit", { newText: currentEl.value });
  await hide();
}

async function hide() {
  await invoke("hide_popup");
}

currentEl.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    commit();
  } else if (e.key === "Escape") {
    e.preventDefault();
    hide();
  }
});

filterEl.addEventListener("input", render);
filterEl.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    hide();
  }
});

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    hide();
  }
});

listen("clip:refresh", refresh);
listen("clip:changed", loadHistory);

refresh();
