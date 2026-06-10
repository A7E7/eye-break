use crate::settings::Settings;
use crate::state::{Phase, Snapshot};
use crate::timer::SharedState;
use tauri::{AppHandle, Manager};

// ---- Commands invoked from the frontend ---------------------------------

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    app.state::<SharedState>().lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> Snapshot {
    app.state::<SharedState>().lock().unwrap().snapshot()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.save(&app)?;
    apply_autostart(&app, settings.start_on_login);

    {
        let state = app.state::<SharedState>();
        let mut s = state.lock().unwrap();
        let work_changed = s.settings.work_minutes != settings.work_minutes;
        s.settings = settings.clone();
        // Re-arm the current countdown if the interval changed mid-work.
        if work_changed && s.phase == Phase::Working {
            s.remaining = settings.work_secs();
        }
    }

    crate::tray::refresh(&app);
    Ok(())
}

#[tauri::command]
pub fn confirm_looking(app: AppHandle) {
    do_confirm(&app);
}

// ---- Shared actions (also called from the tray and toast callback) ------

/// Confirm the user is looking away → start the break countdown.
pub fn do_confirm(app: &AppHandle) {
    {
        let state = app.state::<SharedState>();
        let mut s = state.lock().unwrap();
        if s.phase != Phase::AwaitingConfirm {
            return;
        }
        s.phase = Phase::Break;
        s.remaining = s.settings.break_seconds as i64;
    }
    crate::tray::refresh(app);
}

/// Pause reminders for a number of minutes, or indefinitely (`None`).
pub fn do_pause(app: &AppHandle, minutes: Option<u32>) {
    {
        let state = app.state::<SharedState>();
        let mut s = state.lock().unwrap();
        s.phase = Phase::Paused;
        match minutes {
            Some(m) => {
                s.pause_indefinite = false;
                s.remaining = m as i64 * 60;
            }
            None => {
                s.pause_indefinite = true;
                s.remaining = 0;
            }
        }
    }
    crate::tray::refresh(app);
}

/// Resume from a paused state with a fresh work interval.
pub fn do_resume(app: &AppHandle) {
    app.state::<SharedState>().lock().unwrap().start_working();
    crate::tray::refresh(app);
}

/// Postpone the next break by the configured snooze amount.
pub fn do_snooze(app: &AppHandle) {
    {
        let state = app.state::<SharedState>();
        let mut s = state.lock().unwrap();
        let add = s.settings.snooze_minutes as i64 * 60;
        match s.phase {
            // Mid-reminder or mid-break: postpone a whole snooze window.
            Phase::AwaitingConfirm | Phase::Break => {
                s.start_working();
                s.remaining = add;
            }
            // Otherwise extend the current work countdown.
            _ => {
                s.phase = Phase::Working;
                s.idle_paused = false;
                s.pause_indefinite = false;
                s.remaining = s.remaining.max(0) + add;
            }
        }
    }
    crate::tray::refresh(app);
}

/// Show and focus the settings window (it starts hidden).
pub fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

// ---- helpers ------------------------------------------------------------

fn apply_autostart(app: &AppHandle, enable: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    let _ = if enable { mgr.enable() } else { mgr.disable() };
}
