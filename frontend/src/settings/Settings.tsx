import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Bell, Info, Settings as Gear, Users, type LucideIcon } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from "react";
import { cn } from "../lib/utils";
import { AboutPane, AccountsPane, GeneralPane, NotificationsPane } from "./panes";
import type { Field, PaneProps, SettingsData } from "./types";

const PANE_KEY = "settings-pane";

const PANES: { id: string; title: string; icon: LucideIcon }[] = [
  { id: "general", title: "General", icon: Gear },
  { id: "notifications", title: "Notifications", icon: Bell },
  { id: "accounts", title: "Accounts", icon: Users },
  { id: "about", title: "About", icon: Info },
];

function savedPane() {
  try {
    const id = localStorage.getItem(PANE_KEY);
    if (PANES.some((p) => p.id === id)) return id!;
  } catch {
    // Storage can be off: the window then opens on General.
  }
  return PANES[0].id;
}

export function Settings() {
  const [pane, setPane] = useState(savedPane);
  const [s, setS] = useState<SettingsData | null>(null);
  const [permission, setPermission] = useState("unsupported");
  const [loadError, setLoadError] = useState("");
  const [fieldError, setFieldError] = useState<{ field: Field; message: string } | null>(null);
  // The field of the last change. A save error shows next to it.
  const lastField = useRef<Field | null>(null);

  const load = useCallback(
    () =>
      invoke<SettingsData>("get_settings").then((data) => {
        setS(data);
        if (data.load_error) setLoadError(data.load_error + " Changes are not saved.");
      }),
    [],
  );
  const refreshPermission = useCallback(() => invoke<string>("notification_permission").then(setPermission), []);

  // The title of the window is the name of the pane.
  useEffect(() => {
    const title = PANES.find((p) => p.id === pane)!.title;
    document.title = title;
    getCurrentWindow().setTitle(title);
    try {
      localStorage.setItem(PANE_KEY, pane);
    } catch {
      // Storage can be off: the window then opens on General.
    }
  }, [pane]);

  useEffect(() => {
    load();
    refreshPermission();
    // The rename window, the remove dialog and the tray can change state while this window is away.
    const onFocus = () => {
      refreshPermission();
      load();
    };
    window.addEventListener("focus", onFocus);
    const stop = listen<string>("app-error", (e) => {
      const field = lastField.current;
      if (field) setFieldError({ field, message: e.payload });
      else setLoadError(e.payload);
      // A refused change is reverted in the backend: show the real state again.
      load();
    });
    return () => {
      window.removeEventListener("focus", onFocus);
      stop.then((f) => f());
    };
  }, [load, refreshPermission]);

  function onTabKey(e: KeyboardEvent) {
    const step = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    const next = PANES[(PANES.findIndex((p) => p.id === pane) + step + PANES.length) % PANES.length];
    setPane(next.id);
    document.getElementById(`tab-${next.id}`)?.focus();
  }

  const props: PaneProps | null = s && {
    s,
    change: (field) => {
      setFieldError(null);
      lastField.current = field;
    },
    patch: (p) => setS({ ...s, ...p }),
    errorFor: (field) => (fieldError?.field === field ? fieldError.message : undefined),
  };

  return (
    <div className="flex h-full flex-col">
      <nav
        role="tablist"
        aria-label="Settings panes"
        onKeyDown={onTabKey}
        className="flex shrink-0 justify-center gap-1 border-b border-separator bg-toolbar px-3 pt-2 pb-1.5"
      >
        {PANES.map(({ id, title, icon: Icon }) => (
          <button
            key={id}
            id={`tab-${id}`}
            type="button"
            role="tab"
            aria-selected={pane === id}
            aria-controls="pane"
            tabIndex={pane === id ? 0 : -1}
            onClick={() => setPane(id)}
            className={cn(
              "flex w-[72px] flex-col items-center gap-0.5 rounded-lg px-1 pt-1 pb-0.5 text-[11px] text-secondary transition-colors",
              "hover:bg-hover aria-selected:bg-hover aria-selected:text-label aria-selected:[&>svg]:text-accent",
            )}
          >
            <Icon size={22} strokeWidth={1.75} aria-hidden="true" />
            {title}
          </button>
        ))}
      </nav>
      <main id="pane" role="tabpanel" aria-labelledby={`tab-${pane}`} className="min-h-0 flex-1 overflow-y-auto p-5">
        {loadError && (
          <p role="alert" className="mb-4 rounded-lg bg-group px-3 py-2 text-danger shadow-group">
            {loadError}
          </p>
        )}
        {props && pane === "general" && <GeneralPane {...props} />}
        {props && pane === "notifications" && (
          <NotificationsPane {...props} permission={permission} onPermission={setPermission} />
        )}
        {props && pane === "accounts" && <AccountsPane {...props} />}
        {pane === "about" && <AboutPane />}
      </main>
    </div>
  );
}
