const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let accounts = [];
let active = "";
// Unread count per account UUID; accounts with none are absent.
let unread = {};

function render() {
  const list = document.getElementById("accounts");
  list.replaceChildren(
    ...accounts.map((a) => {
      const el = document.createElement("div");
      el.className = "account" + (a.id === active ? " active" : "");
      el.title = a.name;
      el.textContent = a.name;
      const count = unread[a.id];
      if (count) {
        const badge = document.createElement("span");
        badge.className = "unread";
        badge.textContent = count > 99 ? "99+" : String(count);
        el.title = `${a.name} (${count} unread)`;
        el.appendChild(badge);
      }
      el.addEventListener("click", () => invoke("switch_account", { id: a.id }));
      el.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        invoke("remove_account", { id: a.id });
      });
      return el;
    }),
  );
}

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
