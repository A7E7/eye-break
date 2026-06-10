use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";
const KEY: &str = "settings";

/// User-configurable settings, persisted via `tauri-plugin-store`.
/// Field names are camelCase to match the frontend directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Work interval before a break is suggested. Fractional values are
    /// allowed so the cycle can be exercised quickly during testing.
    pub work_minutes: f64,
    /// How long the "look away" break lasts.
    pub break_seconds: u32,
    /// Launch the app automatically on login.
    pub start_on_login: bool,
    /// Pause the work countdown while the machine is idle.
    pub skip_on_idle: bool,
    /// Idle threshold (seconds) that counts as "away".
    pub idle_threshold_seconds: u32,
    /// Play a sound with each reminder.
    pub sound_on_reminder: bool,
    /// Minutes added when snoozing.
    pub snooze_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 20.0,
            break_seconds: 20,
            start_on_login: false,
            skip_on_idle: true,
            idle_threshold_seconds: 120,
            sound_on_reminder: true,
            snooze_minutes: 5,
        }
    }
}

impl Settings {
    /// Work interval expressed in whole seconds (always at least 1).
    pub fn work_secs(&self) -> i64 {
        ((self.work_minutes * 60.0).round() as i64).max(1)
    }

    pub fn load<R: Runtime>(app: &AppHandle<R>) -> Self {
        match app.store(STORE_PATH) {
            Ok(store) => store
                .get(KEY)
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    }

    pub fn save<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set(KEY, value);
        store.save().map_err(|e| e.to_string())?;
        Ok(())
    }
}
