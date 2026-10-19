const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const theme = document.getElementById("theme");
const zoom = document.getElementById("zoom");
const zoomValue = document.getElementById("zoom-value");
const notifications = document.getElementById("notifications");
const showSender = document.getElementById("show-sender");
const soundEnabled = document.getElementById("sound-enabled");
const soundPath = document.getElementById("sound-path");
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
function saveNotifications() {
  invoke("set_notifications", {
    messages: notifications.checked,
    showSender: showSender.checked,
    soundEnabled: soundEnabled.checked,
    soundPath: soundPath.value.trim(),
  });
}
for (const el of [notifications, showSender, soundEnabled, soundPath]) {
  el.addEventListener("change", saveNotifications);
}
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
  notifications.checked = s.notifications;
  showSender.checked = s.show_sender;
  soundEnabled.checked = s.sound_enabled;
  soundPath.value = s.sound_path;
  closeToTray.checked = s.close_to_tray;
  startMinimized.checked = s.start_minimized;
});
