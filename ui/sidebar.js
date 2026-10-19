const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// Cmd+1 to Cmd+9 switch accounts; keep equal to MAX_SWITCH_SHORTCUTS in main.rs.
const MAX_SHORTCUT = 9;

let accounts = [];
let active = "";
// Unread count per account UUID; accounts with none are absent.
let unread = {};

function render() {
  const list = document.getElementById("accounts");
  const focusedId = document.activeElement?.dataset?.id;
  list.replaceChildren(
    ...accounts.map((a, i) => {
      const el = document.createElement("button");
      el.className = "account" + (a.id === active ? " active" : "");
      el.dataset.id = a.id;
      el.style.setProperty("--hue", a.hue);
      el.textContent = a.initials;
      const count = unread[a.id];
      el.title = i < MAX_SHORTCUT ? `${a.name}, Cmd+${i + 1}` : a.name;
      el.setAttribute("aria-label", count ? `${a.name}, ${count} unread` : a.name);
      if (a.id === active) el.setAttribute("aria-current", "true");
      if (count) {
        const badge = document.createElement("span");
        badge.className = "unread";
        badge.setAttribute("aria-hidden", "true");
        badge.textContent = count > 99 ? "99+" : String(count);
        el.appendChild(badge);
      }
      el.addEventListener("click", () => invoke("switch_account", { id: a.id }));
      el.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        invoke("show_account_menu", { id: a.id });
      });
      return el;
    }),
  );
  if (focusedId) list.querySelector(`[data-id="${focusedId}"]`)?.focus();
}

document.getElementById("accounts").addEventListener("keydown", (e) => {
  const step = { ArrowDown: 1, ArrowUp: -1 }[e.key];
  if (!step) return;
  e.preventDefault();
  const items = [...document.querySelectorAll(".account")];
  items[items.indexOf(document.activeElement) + step]?.focus();
});

document.getElementById("add").addEventListener("click", () => invoke("add_account"));
listen("accounts", (e) => {
  accounts = e.payload.accounts;
  active = e.payload.active;
  render();
});
listen("unread", (e) => {
  unread = e.payload;
  render();
});
invoke("sidebar_ready");
