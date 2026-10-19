const { invoke } = window.__TAURI__.core;

const params = new URLSearchParams(location.search);
const input = document.getElementById("name");
input.value = params.get("name") ?? "";
input.select();

document.getElementById("form").addEventListener("submit", (e) => {
  e.preventDefault();
  if (!input.value.trim()) return;
  invoke("rename_account", { id: params.get("id"), name: input.value });
});
document.getElementById("cancel").addEventListener("click", () => invoke("cancel_rename"));
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") invoke("cancel_rename");
});
