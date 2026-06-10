import type { Settings, Snapshot } from "./types";
import { makeTauriCore } from "./core.tauri";
import { makeWebCore } from "./core.web";

/// The single interface the UI talks to, regardless of platform.
export interface Core {
  /** "desktop" when running inside the Tauri app, "web" in a browser. */
  platform: "desktop" | "web";
  getSettings(): Promise<Settings>;
  getState(): Promise<Snapshot>;
  saveSettings(s: Settings): Promise<void>;
  confirmLooking(): Promise<void>;
  lookAwayNow(): Promise<void>;
  /** Push updates ~1/s. Returns an unsubscribe fn. */
  subscribe(cb: (s: Snapshot) => void): Promise<() => void>;
  /** Desktop: hide the window back to the tray. Web: no-op. */
  dismiss(): Promise<void>;
}

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const core: Core = isTauri() ? makeTauriCore() : makeWebCore();
