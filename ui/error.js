const { invoke } = window.__TAURI__.core;

document.getElementById("retry").addEventListener("click", () => invoke("retry_account"));
