import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BellOff, Plus } from "lucide-react";
import { useEffect, useState, type KeyboardEvent } from "react";
import { cn } from "../lib/utils";

// Cmd+1 to Cmd+9 switch accounts; keep equal to MAX_SWITCH_SHORTCUTS in main.rs.
const MAX_SHORTCUT = 9;
const UNREAD_CAP = 99;

type Account = { id: string; name: string; initials: string; hue: number; muted: boolean };
type AccountsPayload = { accounts: Account[]; active: string };

export function Sidebar() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [active, setActive] = useState("");
  // Unread count per account UUID; accounts with none are absent.
  const [unread, setUnread] = useState<Record<string, number>>({});

  useEffect(() => {
    let stopped = false;
    const stops = Promise.all([
      listen<AccountsPayload>("accounts", (e) => {
        setAccounts(e.payload.accounts);
        setActive(e.payload.active);
      }),
      listen<Record<string, number>>("unread", (e) => setUnread(e.payload)),
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
    <nav className="flex flex-col items-center gap-2 pt-3">
      <div role="group" aria-label="Accounts" onKeyDown={moveFocus} className="flex flex-col items-center gap-2">
        {accounts.map((a, i) => {
          const count = unread[a.id];
          const isActive = a.id === active;
          return (
            <button
              key={a.id}
              type="button"
              data-account
              title={i < MAX_SHORTCUT ? `${a.name}, Cmd+${i + 1}` : a.name}
              aria-label={[a.name, a.muted && "muted", count && `${count} unread`].filter(Boolean).join(", ")}
              aria-current={isActive ? "true" : undefined}
              style={{ backgroundColor: `hsl(${a.hue} 55% 28%)` }}
              className={cn(
                "relative flex size-[38px] items-center justify-center rounded-full text-[14px] font-semibold text-white shadow-thumb",
                "motion-safe:transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95",
                isActive && "shadow-[0_0_0_2px_var(--accent)]",
              )}
              onClick={() => invoke("switch_account", { id: a.id })}
              onContextMenu={(e) => {
                e.preventDefault();
                invoke("show_account_menu", { id: a.id });
              }}
            >
              {isActive && (
                <span className="absolute -left-[11px] top-2.5 bottom-2.5 w-[3px] rounded-r-full bg-accent" aria-hidden="true" />
              )}
              {a.initials}
              {count ? (
                <span
                  aria-hidden="true"
                  className="absolute -right-1.5 -top-1.5 h-[18px] min-w-[18px] rounded-full bg-badge px-1 text-center text-[11px] leading-[18px] font-semibold text-white shadow-thumb"
                >
                  {count > UNREAD_CAP ? `${UNREAD_CAP}+` : count}
                </span>
              ) : null}
              {a.muted && (
                <span
                  aria-hidden="true"
                  className="absolute -right-1 -bottom-1 flex size-4 items-center justify-center rounded-full bg-mute text-white shadow-thumb"
                >
                  <BellOff size={9} strokeWidth={2.5} />
                </span>
              )}
            </button>
          );
        })}
      </div>
      <button
        type="button"
        title="New account"
        aria-label="New account"
        onClick={() => invoke("add_account")}
        className="flex size-[38px] items-center justify-center rounded-full bg-hover text-label transition-colors hover:bg-segment"
      >
        <Plus size={18} strokeWidth={2} />
      </button>
    </nav>
  );
}
