import { browser } from "$app/environment";
import { base } from "$app/paths";
import type { Core } from "./core";
import { DEFAULT_SETTINGS, type Phase, type Settings, type Snapshot } from "./types";

const LS_KEY = "eye-break-settings";

/// Web adapter — runs the 20-20-20 state machine in the browser tab, using the
/// Notification API for reminders and localStorage for persistence. Mirrors the
/// Rust state machine (Working → AwaitingConfirm → Break → Working).
export function makeWebCore(): Core {
  let settings: Settings = browser ? loadSettings() : { ...DEFAULT_SETTINGS };
  let phase: Phase = "working";
  let remaining = workSecs(settings);
  const subscribers = new Set<(s: Snapshot) => void>();

  const snapshot = (): Snapshot => ({
    phase,
    remaining,
    idlePaused: false,
    settings: { ...settings },
  });
  const emit = () => {
    const s = snapshot();
    subscribers.forEach((cb) => cb(s));
  };
  const startWorking = () => {
    phase = "working";
    remaining = workSecs(settings);
  };

  const notify = (title: string, body: string) => {
    if (
      typeof Notification !== "undefined" &&
      Notification.permission === "granted"
    ) {
      try {
        const n = new Notification(title, { body, icon: `${base}/icon.png` });
        n.onclick = () => window.focus();
      } catch {
        /* ignore */
      }
    }
  };

  if (browser) {
    setInterval(() => {
      if (phase === "working") {
        remaining -= 1;
        if (remaining <= 0) {
          phase = "awaitingConfirm";
          notify(
            "Time for an eye break 👀",
            "Look ~20 ft (6 m) into the distance, then confirm.",
          );
        }
      } else if (phase === "break") {
        remaining -= 1;
        if (remaining <= 0) {
          notify("Break complete ✅", "You can look back at your screen.");
          startWorking();
        }
      }
      // awaitingConfirm: hold until the user confirms.
      emit();
    }, 1000);
  }

  return {
    platform: "web",
    getSettings: async () => ({ ...settings }),
    getState: async () => snapshot(),
    saveSettings: async (s) => {
      const workChanged = s.workMinutes !== settings.workMinutes;
      settings = { ...s };
      if (browser) persist(settings);
      if (workChanged && phase === "working") remaining = workSecs(settings);
      emit();
    },
    confirmLooking: async () => {
      if (phase === "awaitingConfirm") {
        phase = "break";
        remaining = settings.breakSeconds;
        emit();
      }
    },
    lookAwayNow: async () => {
      phase = "break";
      remaining = settings.breakSeconds;
      emit();
    },
    subscribe: async (cb) => {
      subscribers.add(cb);
      cb(snapshot());
      return () => subscribers.delete(cb);
    },
    dismiss: async () => {},
  };
}

function workSecs(s: Settings): number {
  return Math.max(1, Math.round(s.workMinutes * 60));
}

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (raw) return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_SETTINGS };
}

function persist(s: Settings) {
  try {
    localStorage.setItem(LS_KEY, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}
