const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const theme = document.getElementById("theme");
const zoom = document.getElementById("zoom");
const zoomValue = document.getElementById("zoom-value");
const closeToTray = document.getElementById("close-to-tray");
const startMinimized = document.getElementById("start-minimized");
const error = document.getElementById("error");

function showZoom(factor) {
  zoom.value = factor;
  zoomValue.textContent = Math.round(factor * 100) + "%";
}

theme.addEventListener("change", () => invoke("set_theme", { theme: theme.value }));
zoom.addEventListener("change", () => {
  showZoom(Number(zoom.value));
  invoke("set_zoom", { factor: Number(zoom.value) });
});
closeToTray.addEventListener("change", () =>
  invoke("set_close_to_tray", { enabled: closeToTray.checked }));
startMinimized.addEventListener("change", () =>
  invoke("set_start_minimized", { enabled: startMinimized.checked }));
listen("app-error", (e) => {
  error.textContent = e.payload;
  error.hidden = false;
});

invoke("get_settings").then((s) => {
  theme.value = s.theme;
  showZoom(s.zoom_factor);
  closeToTray.checked = s.close_to_tray;
  startMinimized.checked = s.start_minimized;
});
