export type SettingsAccount = { id: string; name: string };

/** Shape of the `get_settings` reply. */
export type SettingsData = {
  theme: string;
  zoom_factor: number;
  notifications: boolean;
  show_sender: boolean;
  sound_enabled: boolean;
  sound_path: string;
  autostart_enabled: boolean;
  clear_cache_on_exit: boolean;
  clear_cache_supported: boolean;
  close_to_tray: boolean;
  start_minimized: boolean;
  snooze_until: number | null;
  accounts: SettingsAccount[];
  load_error: string | null;
};

/** Names the control a save error shows under. */
export type Field =
  | "theme"
  | "zoom"
  | "autostart"
  | "start-minimized"
  | "close-to-tray"
  | "clear-cache"
  | "notifications"
  | "show-sender"
  | "sound-enabled"
  | "sound-path"
  | "accounts";

export type PaneProps = {
  s: SettingsData;
  /** Marks `field` as the last change and clears old errors. */
  change: (field: Field) => void;
  /** Applies a local edit right away. The backend reverts it with an error if the save fails. */
  patch: (p: Partial<SettingsData>) => void;
  errorFor: (field: Field) => string | undefined;
};
