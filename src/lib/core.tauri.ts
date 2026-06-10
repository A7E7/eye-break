import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Core } from "./core";
import type { Settings, Snapshot } from "./types";

/// Desktop adapter — delegates to the Rust backend (the source of truth).
export function makeTauriCore(): Core {
  return {
    platform: "desktop",
    getSettings: () => invoke<Settings>("get_settings"),
    getState: () => invoke<Snapshot>("get_state"),
    saveSettings: (s) => invoke("save_settings", { settings: s }).then(() => {}),
    confirmLooking: () => invoke("confirm_looking").then(() => {}),
    lookAwayNow: () => invoke("look_away_now").then(() => {}),
    subscribe: (cb) => listen<Snapshot>("state-changed", (e) => cb(e.payload)),
    dismiss: () => getCurrentWindow().hide(),
  };
}
