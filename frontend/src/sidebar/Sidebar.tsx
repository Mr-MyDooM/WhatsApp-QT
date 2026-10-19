import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BellOff, PanelLeftClose, PanelLeftOpen, Plus, Settings, type LucideIcon } from "lucide-react";
import { useEffect, useState, type KeyboardEvent } from "react";
import { useAvatars } from "../lib/useAvatars";
import { cn } from "../lib/utils";

// Cmd+1 to Cmd+9 switch accounts; keep equal to MAX_SWITCH_SHORTCUTS in main.rs.
const MAX_SHORTCUT = 9;
const UNREAD_CAP = 99;

type Account = { id: string; name: string; initials: string; hue: number; muted: boolean };
type AccountsPayload = { accounts: Account[]; active: string };

const rowClass =
  "relative mx-1.5 flex w-[calc(100%-12px)] items-center gap-2.5 rounded-lg px-[11px] py-[3px] text-left";

// Same look for the New account, Settings and Toggle buttons: icon circle, label when wide.
function ActionRow({ icon: Icon, label, wide, onClick }: {
  icon: LucideIcon;
  label: string;
  wide: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      className={cn(rowClass, "group")}
    >
      <span className="flex size-[38px] shrink-0 items-center justify-center rounded-full bg-hover text-label transition-colors group-hover:bg-segment">
        <Icon size={18} strokeWidth={2} aria-hidden="true" />
      </span>
      {wide && <span className="truncate text-label">{label}</span>}
    </button>
  );
}

export function Sidebar() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [active, setActive] = useState("");
  // Unread count per account UUID; accounts with none are absent.
  const [unread, setUnread] = useState<Record<string, number>>({});
  const [wide, setWide] = useState(false);
  const avatars = useAvatars();

  useEffect(() => {
    let stopped = false;
    const stops = Promise.all([
      listen<AccountsPayload>("accounts", (e) => {
        setAccounts(e.payload.accounts);
        setActive(e.payload.active);
      }),
      listen<Record<string, number>>("unread", (e) => setUnread(e.payload)),
      listen<boolean>("sidebar-wide", (e) => setWide(e.payload)),
    ]).then((fns) => {
      // Ask for the first state only when both listeners are in place.
      if (stopped) fns.forEach((f) => f());
      else invoke("sidebar_ready");
      return fns;
    });
    return () => {
      stopped = true;
      stops.then((fns) => fns.forEach((f) => f()));
    };
  }, []);

  function moveFocus(e: KeyboardEvent<HTMLDivElement>) {
    const step = { ArrowDown: 1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    const items = [...e.currentTarget.querySelectorAll<HTMLElement>("[data-account]")];
    items[items.indexOf(document.activeElement as HTMLElement) + step]?.focus();
  }

  return (
    <nav className="flex h-full flex-col">
      {/* Top strip: the traffic lights sit here, so they never cover an account. Drags the window. */}
      <div data-tauri-drag-region className="h-[38px] shrink-0" />
      <div
        role="group"
        aria-label="Accounts"
        onKeyDown={moveFocus}
        className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pb-2"
      >
        {accounts.map((a, i) => {
          const count = unread[a.id];
          const isActive = a.id === active;
          const badge = count ? (count > UNREAD_CAP ? `${UNREAD_CAP}+` : String(count)) : null;
          return (
            <button
              key={a.id}
              type="button"
              data-account
              title={i < MAX_SHORTCUT ? `${a.name}, Cmd+${i + 1}` : a.name}
              aria-label={[a.name, a.muted && "muted", count && `${count} unread`].filter(Boolean).join(", ")}
              aria-current={isActive ? "true" : undefined}
              className={cn(
                rowClass,
                "motion-safe:transition-transform motion-safe:active:scale-[0.98]",
                wide && isActive && "bg-hover",
              )}
              onClick={() => invoke("switch_account", { id: a.id })}
              onContextMenu={(e) => {
                e.preventDefault();
                invoke("show_account_menu", { id: a.id });
              }}
            >
              {isActive && (
                <span className="absolute -left-1.5 top-2.5 bottom-2.5 w-[3px] rounded-r-full bg-accent" aria-hidden="true" />
              )}
              <span
                aria-hidden="true"
                style={{ backgroundColor: `hsl(${a.hue} 55% 28%)` }}
                className={cn(
                  "relative flex size-[38px] shrink-0 items-center justify-center rounded-full text-[14px] font-semibold text-white shadow-thumb",
                  isActive && "shadow-[0_0_0_2px_var(--accent)]",
                )}
              >
                {avatars[a.id] ? (
                  <img src={avatars[a.id]} alt="" className="size-full rounded-full object-cover" />
                ) : (
                  a.initials
                )}
                {badge && !wide && (
                  <span className="absolute -right-1.5 -top-1.5 h-[18px] min-w-[18px] rounded-full bg-badge px-1 text-center text-[11px] leading-[18px] font-semibold text-white shadow-thumb">
                    {badge}
                  </span>
                )}
                {a.muted && !wide && (
                  <span className="absolute -right-1 -bottom-1 flex size-4 items-center justify-center rounded-full bg-mute text-white shadow-thumb">
                    <BellOff size={9} strokeWidth={2.5} />
                  </span>
                )}
              </span>
              {wide && (
                <>
                  <span aria-hidden="true" className="min-w-0 flex-1 truncate text-label">
                    {a.name}
                  </span>
                  {a.muted && <BellOff aria-hidden="true" size={13} className="shrink-0 text-secondary" />}
                  {badge && (
                    <span
                      aria-hidden="true"
                      className="h-[18px] min-w-[18px] shrink-0 rounded-full bg-badge px-1 text-center text-[11px] leading-[18px] font-semibold text-white"
                    >
                      {badge}
                    </span>
                  )}
                </>
              )}
            </button>
          );
        })}
        <ActionRow icon={Plus} label="New account" wide={wide} onClick={() => invoke("add_account")} />
      </div>
      <div className="flex shrink-0 flex-col gap-1 pb-3 pt-1">
        <ActionRow icon={Settings} label="Settings" wide={wide} onClick={() => invoke("open_settings_window")} />
        <ActionRow
          icon={wide ? PanelLeftClose : PanelLeftOpen}
          label={wide ? "Hide account names" : "Show account names"}
          wide={wide}
          onClick={() => invoke("toggle_sidebar")}
        />
      </div>
    </nav>
  );
}
