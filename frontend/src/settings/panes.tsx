import { invoke } from "@tauri-apps/api/core";
import { Monitor, Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Group, Row } from "../components/ui/group";
import { Input } from "../components/ui/input";
import { Segmented } from "../components/ui/segmented";
import { Slider } from "../components/ui/slider";
import { Switch } from "../components/ui/switch";
import { useAvatars } from "../lib/useAvatars";
import type { PaneProps } from "./types";

const ZOOM_MIN = 0.25;
const ZOOM_MAX = 3;
const ZOOM_STEP = 0.05;
const ZOOM_PRESETS = [0.9, 1, 1.1, 1.25];
const percent = (factor: number) => `${Math.round(factor * 100)}%`;

const THEMES = [
  { value: "system", label: "System", icon: <Monitor size={13} /> },
  { value: "light", label: "Light", icon: <Sun size={13} /> },
  { value: "dark", label: "Dark", icon: <Moon size={13} /> },
];

export function GeneralPane({ s, change, patch, errorFor }: PaneProps) {
  // The slider moves this value live. The save runs when the user lets go.
  const [zoom, setZoom] = useState(s.zoom_factor);
  useEffect(() => setZoom(s.zoom_factor), [s.zoom_factor]);

  function saveZoom(factor: number) {
    change("zoom");
    setZoom(factor);
    patch({ zoom_factor: factor });
    invoke("set_zoom", { factor });
  }

  return (
    <>
      <Group>
        <Row label="Theme" error={errorFor("theme")}>
          <Segmented
            label="Theme"
            value={s.theme}
            options={THEMES}
            onValueChange={(theme) => {
              change("theme");
              patch({ theme });
              invoke("set_theme", { theme });
            }}
          />
        </Row>
        <Row label="Zoom" htmlFor="zoom" error={errorFor("zoom")}>
          <Slider
            id="zoom"
            className="w-44"
            aria-label="Zoom"
            min={ZOOM_MIN}
            max={ZOOM_MAX}
            step={ZOOM_STEP}
            value={[zoom]}
            onValueChange={([v]) => setZoom(v)}
            onValueCommit={([v]) => saveZoom(v)}
          />
          <output className="w-10 text-right tabular-nums text-secondary">{percent(zoom)}</output>
        </Row>
        <div className="flex justify-end px-3 py-2">
          <Segmented
            label="Zoom presets"
            value={String(ZOOM_PRESETS.find((p) => Math.abs(zoom - p) < 0.005) ?? "")}
            options={ZOOM_PRESETS.map((p) => ({ value: String(p), label: percent(p) }))}
            onValueChange={(v) => saveZoom(Number(v))}
          />
        </div>
      </Group>
      <Group>
        <SwitchRow
          id="autostart"
          label="Start at login"
          checked={s.autostart_enabled}
          error={errorFor("autostart")}
          onChange={(enabled) => {
            change("autostart");
            patch({ autostart_enabled: enabled });
            invoke("set_autostart", { enabled });
          }}
        />
        <SwitchRow
          id="start-minimized"
          label="Start minimized"
          checked={s.start_minimized}
          error={errorFor("start-minimized")}
          onChange={(enabled) => {
            change("start-minimized");
            patch({ start_minimized: enabled });
            invoke("set_start_minimized", { enabled });
          }}
        />
        <SwitchRow
          id="close-to-tray"
          label="Close to tray"
          checked={s.close_to_tray}
          error={errorFor("close-to-tray")}
          onChange={(enabled) => {
            change("close-to-tray");
            patch({ close_to_tray: enabled });
            invoke("set_close_to_tray", { enabled });
          }}
        />
        <SwitchRow
          id="sidebar-auto-hide"
          label="Auto-hide sidebar"
          checked={s.sidebar_auto_hide}
          error={errorFor("sidebar-auto-hide")}
          onChange={(enabled) => {
            change("sidebar-auto-hide");
            patch({ sidebar_auto_hide: enabled });
            invoke("set_sidebar_auto_hide", { enabled });
          }}
        />
        {s.clear_cache_supported && (
          <SwitchRow
            id="clear-cache"
            label="Clear cache on exit"
            checked={s.clear_cache_on_exit}
            error={errorFor("clear-cache")}
            onChange={(enabled) => {
              change("clear-cache");
              patch({ clear_cache_on_exit: enabled });
              invoke("set_clear_cache_on_exit", { enabled });
            }}
          />
        )}
      </Group>
    </>
  );
}

function SwitchRow(props: {
  id: string;
  label: string;
  checked: boolean;
  error?: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <Row label={props.label} htmlFor={props.id} error={props.error}>
      <Switch id={props.id} checked={props.checked} onCheckedChange={props.onChange} />
    </Row>
  );
}

const PERMISSION_LABELS: Record<string, string> = {
  allowed: "Allowed",
  denied: "Not allowed",
  not_asked: "Not asked yet",
  no_bundle: "Not available outside the app bundle",
  unsupported: "Not available on this system",
};
const PERMISSION_BUTTONS: Record<string, [label: string, command: string]> = {
  not_asked: ["Allow notifications", "request_notification_permission"],
  denied: ["Open System Settings", "open_notification_settings"],
};

const TEST_RESULTS: Record<string, string> = {
  sent: "Test notification sent.",
  off: "No alert came. Message notifications are off.",
  snoozed: "No alert came. Snooze is on.",
  muted: "No alert came. The active account is muted.",
  denied: "No alert came. Notifications are not allowed for this app.",
  no_bundle: "No alert came. Alerts work only in the installed app.",
};

export function NotificationsPane({
  s,
  change,
  patch,
  errorFor,
  permission,
  onPermission,
}: PaneProps & { permission: string; onPermission: (status: string) => void }) {
  const [soundPath, setSoundPath] = useState(s.sound_path);
  useEffect(() => setSoundPath(s.sound_path), [s.sound_path]);

  function save(next: { notifications?: boolean; show_sender?: boolean; sound_enabled?: boolean; path?: string }) {
    invoke("set_notifications", {
      messages: next.notifications ?? s.notifications,
      showSender: next.show_sender ?? s.show_sender,
      soundEnabled: next.sound_enabled ?? s.sound_enabled,
      soundPath: (next.path ?? s.sound_path).trim(),
    });
  }
  const button = PERMISSION_BUTTONS[permission];
  const snooze = s.snooze_until ? "On until " + new Date(s.snooze_until * 1000).toLocaleString() : "Off";
  const [testResult, setTestResult] = useState("");

  // Used by the Choose and Default buttons. An empty path means the default sound.
  function saveSound(path: string) {
    change("sound-path");
    setSoundPath(path);
    patch({ sound_path: path });
    save({ path });
  }
  function startSnooze(minutes: number) {
    invoke<number | null>("snooze_notifications", { minutes }).then((end) => patch({ snooze_until: end }));
  }

  return (
    <>
      <Group>
        <SwitchRow
          id="notifications"
          label="Message notifications"
          checked={s.notifications}
          error={errorFor("notifications")}
          onChange={(notifications) => {
            change("notifications");
            patch({ notifications });
            save({ notifications });
          }}
        />
        <Row label="macOS notifications" hint={PERMISSION_LABELS[permission] ?? PERMISSION_LABELS.unsupported}>
          {button && (
            <Button
              onClick={() =>
                invoke<string | null>(button[1]).then((status) => {
                  // Opening System Settings returns nothing: the focus event refreshes the status.
                  if (typeof status === "string") onPermission(status);
                })
              }
            >
              {button[0]}
            </Button>
          )}
        </Row>
        <SwitchRow
          id="show-sender"
          label="Show sender"
          checked={s.show_sender}
          error={errorFor("show-sender")}
          onChange={(show_sender) => {
            change("show-sender");
            patch({ show_sender });
            save({ show_sender });
          }}
        />
        <SwitchRow
          id="sound-enabled"
          label="Sound"
          checked={s.sound_enabled}
          error={errorFor("sound-enabled")}
          onChange={(sound_enabled) => {
            change("sound-enabled");
            patch({ sound_enabled });
            save({ sound_enabled });
          }}
        />
        <Row
          label="Sound file"
          htmlFor="sound-path"
          hint="Leave empty for the default sound."
          error={errorFor("sound-path")}
        >
          <Button onClick={() => invoke<string | null>("choose_sound_file").then((path) => path && saveSound(path))}>
            Choose...
          </Button>
          <Button disabled={!s.sound_path} onClick={() => saveSound("")}>
            Default
          </Button>
          <Input
            id="sound-path"
            className="w-56"
            value={soundPath}
            onChange={(e) => setSoundPath(e.target.value)}
            onBlur={() => {
              if (soundPath.trim() === s.sound_path) return;
              change("sound-path");
              patch({ sound_path: soundPath.trim() });
              save({ path: soundPath });
            }}
            onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          />
        </Row>
      </Group>
      <Group>
        <Row label="Test" hint={testResult}>
          <Button
            onClick={() =>
              invoke<string>("send_test_notification").then((r) => setTestResult(TEST_RESULTS[r] ?? TEST_RESULTS.sent))
            }
          >
            Send Test Notification
          </Button>
        </Row>
      </Group>
      <Group>
        <Row label="Snooze" hint={snooze}>
          <Button onClick={() => startSnooze(60)}>Snooze 1 hour</Button>
          <Button onClick={() => startSnooze(8 * 60)}>Snooze 8 hours</Button>
          <Button
            disabled={!s.snooze_until}
            onClick={() => {
              invoke("end_snooze");
              patch({ snooze_until: null });
            }}
          >
            End Snooze
          </Button>
        </Row>
      </Group>
    </>
  );
}

export function AccountsPane({ s, change, errorFor }: PaneProps) {
  const error = errorFor("accounts");
  const avatars = useAvatars();
  return (
    <Group>
      {s.accounts.map((a) => (
        <Row key={a.id} label={a.name}>
          <span
            aria-hidden="true"
            className="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded-full bg-segment text-[10px] font-semibold text-secondary"
          >
            {avatars[a.id] ? (
              <img src={avatars[a.id]} alt="" className="size-full object-cover" />
            ) : (
              a.name.split(/\s+/).filter(Boolean).slice(0, 2).map((w) => w[0].toUpperCase()).join("")
            )}
          </span>
          <Button
            onClick={() => {
              change("accounts");
              invoke("open_rename_account", { id: a.id });
            }}
          >
            Rename...
          </Button>
          <Button
            onClick={() => {
              change("accounts");
              invoke("remove_account", { id: a.id });
            }}
          >
            Remove...
          </Button>
        </Row>
      ))}
      {error && (
        <p role="alert" className="px-3 py-2 text-xs text-danger">
          {error}
        </p>
      )}
    </Group>
  );
}

type About = {
  name: string;
  version: string;
  description: string;
  license: string;
  repository: string;
  settings_path: string | null;
};

export function AboutPane() {
  const [about, setAbout] = useState<About | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    invoke<About>("get_about").then(setAbout, () => setFailed(true));
  }, []);
  return (
    <div className="flex flex-col items-center gap-1 pt-6 text-center">
      <img src="/icon.png" alt="" width={96} height={96} className="mb-2 drop-shadow-md" />
      <h1 className="text-xl font-semibold">{about?.name ?? "Pairline"}</h1>
      {failed && (
        <p role="alert" className="text-xs text-danger">
          Could not load the app details.
        </p>
      )}
      {about && (
        <>
          <p className="text-secondary">Version {about.version}</p>
          <p className="mt-2 text-label">{about.description}</p>
          <div className="mt-2 flex items-center gap-2">
            <Button onClick={() => invoke("open_repository")}>Repository</Button>
            <span className="text-secondary">{about.license} licence</span>
          </div>
          {about.settings_path && (
            <p className="mt-4 max-w-96 break-all text-xs text-secondary select-text">
              Settings file: {about.settings_path}
            </p>
          )}
        </>
      )}
      <p className="mt-4 max-w-72 text-xs text-secondary">Not affiliated with or endorsed by WhatsApp or Meta.</p>
    </div>
  );
}
