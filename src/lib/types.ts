export type Phase = "working" | "awaitingConfirm" | "break" | "paused";

export interface Settings {
  workMinutes: number;
  breakSeconds: number;
  startOnLogin: boolean;
  skipOnIdle: boolean;
  idleThresholdSeconds: number;
  soundOnReminder: boolean;
  snoozeMinutes: number;
}

export interface Snapshot {
  phase: Phase;
  remaining: number;
  idlePaused: boolean;
  settings: Settings;
}

export const DEFAULT_SETTINGS: Settings = {
  workMinutes: 20,
  breakSeconds: 20,
  startOnLogin: false,
  skipOnIdle: true,
  idleThresholdSeconds: 120,
  soundOnReminder: true,
  snoozeMinutes: 5,
};
