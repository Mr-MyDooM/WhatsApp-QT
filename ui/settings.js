const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const theme = document.getElementById("theme");
const zoom = document.getElementById("zoom");
const zoomValue = document.getElementById("zoom-value");
const autostart = document.getElementById("autostart");
const clearCache = document.getElementById("clear-cache");
const clearCacheRow = document.getElementById("clear-cache-row");
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
autostart.addEventListener("change", () =>
  invoke("set_autostart", { enabled: autostart.checked })
);
clearCache.addEventListener("change", () =>
  invoke("set_clear_cache_on_exit", { enabled: clearCache.checked })
);

function load() {
  return invoke("get_settings").then((s) => {
    theme.value = s.theme;
    showZoom(s.zoom_factor);
    autostart.checked = s.autostart_enabled;
    clearCache.checked = s.clear_cache_on_exit;
    clearCacheRow.hidden = !s.clear_cache_supported;
    closeToTray.checked = s.close_to_tray;
    startMinimized.checked = s.start_minimized;
  });
}

closeToTray.addEventListener("change", () =>
  invoke("set_close_to_tray", { enabled: closeToTray.checked }));
startMinimized.addEventListener("change", () =>
  invoke("set_start_minimized", { enabled: startMinimized.checked }));
listen("app-error", (e) => {
  error.textContent = e.payload;
  error.hidden = false;
  // A refused change is reverted in the backend: show the real state again.
  load();
});

load();
