import type { SettingChange, SettingsView } from "./settings-view";

/** Shape of the `get_settings` reply, generated from Rust. */
export type SettingsData = SettingsView;

export type PaneProps = {
  s: SettingsData;
  /** Marks `field` as the last change and clears old errors. */
  change: (field: SettingChange["field"] | "accounts") => void;
  /** The one command that saves a setting. A refusal or a failed call shows under the field. */
  update: (change: SettingChange) => void;
  /** Applies a local edit right away. The backend reverts it with an error if the save fails. */
  patch: (p: Partial<SettingsData>) => void;
  errorFor: (field: SettingChange["field"] | "accounts") => string | undefined;
};
