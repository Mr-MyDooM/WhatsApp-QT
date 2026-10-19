const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let accounts = [];
let active = "";

function confirmRemove(id) {
  const account = accounts.find((a) => a.id === id);
  if (!account) return;
  const ok = confirm(`Remove "${account.name}" and delete its login and data? This cannot be undone.`);
  if (ok) invoke("remove_account", { id });
}

function render() {
  const list = document.getElementById("accounts");
  list.replaceChildren(
    ...accounts.map((a) => {
      const el = document.createElement("div");
      el.className = "account" + (a.id === active ? " active" : "");
      el.title = a.name;
      el.textContent = a.name;
      el.addEventListener("click", () => invoke("switch_account", { id: a.id }));
      el.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        confirmRemove(a.id);
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
listen("remove-active", () => confirmRemove(active));
invoke("sidebar_ready");
