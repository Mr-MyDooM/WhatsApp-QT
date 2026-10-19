import type { SettingChange, SettingsView } from "./settings-view";

/** Shape of the `get_settings` reply, generated from Rust. */
export type SettingsData = SettingsView;

/** Names the control a save error shows under. */
export type Field = SettingChange["field"] | "accounts";

export type PaneProps = {
  s: SettingsData;
  /** Marks `field` as the last change and clears old errors. */
  change: (field: Field) => void;
  /** Applies a local edit right away. The backend reverts it with an error if the save fails. */
  patch: (p: Partial<SettingsData>) => void;
  errorFor: (field: Field) => string | undefined;
};
