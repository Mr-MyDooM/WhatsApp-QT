const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const theme = document.getElementById("theme");
const zoom = document.getElementById("zoom");
const zoomValue = document.getElementById("zoom-value");
const notifications = document.getElementById("notifications");
const showSender = document.getElementById("show-sender");
const soundEnabled = document.getElementById("sound-enabled");
const soundPath = document.getElementById("sound-path");
const autostart = document.getElementById("autostart");
const clearCache = document.getElementById("clear-cache");
const clearCacheRow = document.getElementById("clear-cache-row");
const closeToTray = document.getElementById("close-to-tray");
const startMinimized = document.getElementById("start-minimized");
const loadError = document.getElementById("load-error");
const snoozeStatus = document.getElementById("snooze-status");
const accountList = document.getElementById("accounts");
const tabs = Array.from(document.querySelectorAll("#tabs button"));
const PANE_KEY = "settings-pane";

// The field of the last change. A save error shows next to it.
let lastField = null;

function fieldOf(el) {
  return el.closest(".field") || el.closest("section");
}

function clearErrors() {
  for (const p of document.querySelectorAll(".error")) p.hidden = true;
}

function showPane(name) {
  const tab = tabs.find((t) => t.dataset.pane === name) || tabs[0];
  for (const t of tabs) t.setAttribute("aria-selected", String(t === tab));
  for (const s of document.querySelectorAll("section")) {
    s.hidden = s.id !== "pane-" + tab.dataset.pane;
  }
  const title = document.getElementById("pane-" + tab.dataset.pane).dataset.title;
  document.title = title;
  window.__TAURI__.window.getCurrentWindow().setTitle(title);
  try {
    localStorage.setItem(PANE_KEY, tab.dataset.pane);
  } catch (_) {
    // Storage can be off: the window then opens on General.
  }
}

for (const t of tabs) t.addEventListener("click", () => showPane(t.dataset.pane));

function changed(el) {
  clearErrors();
  lastField = fieldOf(el);
}

function showZoom(factor) {
  zoom.value = factor;
  zoomValue.textContent = Math.round(factor * 100) + "%";
}

function saveZoom(factor) {
  showZoom(factor);
  invoke("set_zoom", { factor });
}

theme.addEventListener("change", () => {
  changed(theme);
  invoke("set_theme", { theme: theme.value });
});
zoom.addEventListener("change", () => {
  changed(zoom);
  saveZoom(Number(zoom.value));
});
for (const b of document.querySelectorAll("[data-zoom]")) {
  b.addEventListener("click", () => {
    changed(b);
    saveZoom(Number(b.dataset.zoom));
  });
}
function saveNotifications() {
  invoke("set_notifications", {
    messages: notifications.checked,
    showSender: showSender.checked,
    soundEnabled: soundEnabled.checked,
    soundPath: soundPath.value.trim(),
  });
}
for (const el of [notifications, showSender, soundEnabled, soundPath]) {
  el.addEventListener("change", () => {
    changed(el);
    saveNotifications();
  });
}
autostart.addEventListener("change", () => {
  changed(autostart);
  invoke("set_autostart", { enabled: autostart.checked });
});
clearCache.addEventListener("change", () => {
  changed(clearCache);
  invoke("set_clear_cache_on_exit", { enabled: clearCache.checked });
});

function load() {
  return invoke("get_settings").then((s) => {
    theme.value = s.theme;
    showZoom(s.zoom_factor);
    notifications.checked = s.notifications;
    showSender.checked = s.show_sender;
    soundEnabled.checked = s.sound_enabled;
    soundPath.value = s.sound_path;
    autostart.checked = s.autostart_enabled;
    clearCache.checked = s.clear_cache_on_exit;
    clearCacheRow.hidden = !s.clear_cache_supported;
    closeToTray.checked = s.close_to_tray;
    startMinimized.checked = s.start_minimized;
    snoozeStatus.textContent = s.snooze_until
      ? "On until " + new Date(s.snooze_until * 1000).toLocaleString()
      : "Off";
    showAccounts(s.accounts);
    if (s.load_error) {
      loadError.textContent = s.load_error + " Changes are not saved.";
      loadError.hidden = false;
    }
  });
}

function showAccounts(accounts) {
  accountList.replaceChildren(
    ...accounts.map((a) => {
      const li = document.createElement("li");
      const name = document.createElement("span");
      name.className = "name";
      name.textContent = a.name;
      const rename = document.createElement("button");
      rename.type = "button";
      rename.textContent = "Rename...";
      rename.addEventListener("click", () => {
        changed(accountList);
        invoke("open_rename_account", { id: a.id });
      });
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "Remove...";
      remove.addEventListener("click", () => {
        changed(accountList);
        invoke("remove_account", { id: a.id });
      });
      li.append(name, rename, remove);
      return li;
    })
  );
}

closeToTray.addEventListener("change", () => {
  changed(closeToTray);
  invoke("set_close_to_tray", { enabled: closeToTray.checked });
});
startMinimized.addEventListener("change", () => {
  changed(startMinimized);
  invoke("set_start_minimized", { enabled: startMinimized.checked });
});
listen("app-error", (e) => {
  const slot = lastField && lastField.querySelector(".error");
  if (slot) {
    slot.textContent = e.payload;
    slot.hidden = false;
  } else {
    loadError.textContent = e.payload;
    loadError.hidden = false;
  }
  // A refused change is reverted in the backend: show the real state again.
  load();
});

const permissionStatus = document.getElementById("permission-status");
const permissionButton = document.getElementById("permission-button");
const PERMISSION_LABELS = {
  allowed: "Allowed",
  denied: "Not allowed",
  not_asked: "Not asked yet",
  no_bundle: "Not available outside the app bundle",
  unsupported: "Not available on this system",
};
const PERMISSION_BUTTONS = {
  not_asked: ["Allow notifications", "request_notification_permission"],
  denied: ["Open System Settings", "open_notification_settings"],
};
let permissionCommand = null;

function showPermission(status) {
  permissionStatus.textContent = PERMISSION_LABELS[status] || PERMISSION_LABELS.unsupported;
  const button = PERMISSION_BUTTONS[status];
  permissionButton.hidden = !button;
  permissionCommand = button ? button[1] : null;
  if (button) permissionButton.textContent = button[0];
}

function refreshPermission() {
  return invoke("notification_permission").then(showPermission);
}

permissionButton.addEventListener("click", () => {
  if (!permissionCommand) return;
  invoke(permissionCommand).then((status) => {
    // Opening System Settings returns nothing: the focus event refreshes the status.
    if (typeof status === "string") showPermission(status);
  });
});
// The rename window, the remove dialog and the tray can change state while this window is away.
window.addEventListener("focus", () => {
  refreshPermission();
  load();
});

const { getName, getVersion } = window.__TAURI__.app;
getName().then((n) => (document.getElementById("app-name").textContent = n));
getVersion().then((v) => (document.getElementById("app-version").textContent = v));

let startPane = "general";
try {
  startPane = localStorage.getItem(PANE_KEY) || startPane;
} catch (_) {
  // Storage can be off: open on General.
}
showPane(startPane);
refreshPermission();
load();
