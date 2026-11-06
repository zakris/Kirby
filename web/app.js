import { API_BASE } from "./config.js";

const today = new Date().toLocaleDateString("en-CA"); // YYYY-MM-DD in local time
const list = document.getElementById("tasks");
const errorEl = document.getElementById("error");

async function api(path, opts = {}) {
  const res = await fetch(API_BASE + path, {
    headers: { "Content-Type": "application/json" },
    ...opts,
  });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.status === 204 ? null : res.json();
}

async function refresh() {
  try {
    const tasks = await api(`/tasks?date=${today}`);
    errorEl.hidden = true;
    list.replaceChildren(...tasks.map(render));
  } catch (e) {
    errorEl.textContent = `Can't reach backend: ${e.message}`;
    errorEl.hidden = false;
  }
}

function render(t) {
  const li = document.createElement("li");
  li.className = t.done ? "done" : "";
  const box = document.createElement("input");
  box.type = "checkbox";
  box.checked = t.done;
  box.onchange = () => api(`/tasks/${t.id}`, { method: "PATCH", body: JSON.stringify({ done: box.checked }) }).then(refresh);
  const label = document.createElement("span");
  label.textContent = t.title;
  const del = document.createElement("button");
  del.textContent = "✕";
  del.onclick = () => api(`/tasks/${t.id}`, { method: "DELETE" }).then(refresh);
  li.append(box, label, del);
  return li;
}

document.getElementById("add").onsubmit = async (e) => {
  e.preventDefault();
  const input = document.getElementById("title");
  await api("/tasks", { method: "POST", body: JSON.stringify({ title: input.value, date: today }) });
  input.value = "";
  refresh();
};

refresh();
