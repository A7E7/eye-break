use crate::state::{AppState, Phase};
use crate::{idle, notify, tray};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// The shared, mutable app state, registered with Tauri's managed state.
pub type SharedState = Mutex<AppState>;

/// After this many seconds in AwaitingConfirm, re-fire the reminder once.
const GRACE_SECS: i64 = 60;
/// After this many seconds in AwaitingConfirm with no response, skip the break.
const SKIP_AFTER_SECS: i64 = 120;

/// Spawn the 1 Hz background loop that drives the state machine.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        tick(&app);
    });
}

fn tick(app: &AppHandle) {
    let state = app.state::<SharedState>();
    let mut s = state.lock().unwrap();
    let sound = s.settings.sound_on_reminder;
    let mut tray_dirty = false;

    match s.phase {
        Phase::Working => {
            let idle_now = s.settings.skip_on_idle
                && idle::idle_seconds() >= s.settings.idle_threshold_seconds as u64;
            if s.idle_paused != idle_now {
                s.idle_paused = idle_now;
                tray_dirty = true;
            }
            if !idle_now {
                s.remaining -= 1;
                if s.remaining <= 0 {
                    s.phase = Phase::AwaitingConfirm;
                    s.awaiting_elapsed = 0;
                    s.renotified = false;
                    notify::reminder(app, sound);
                    tray_dirty = true;
                }
            }
        }
        Phase::AwaitingConfirm => {
            s.awaiting_elapsed += 1;
            if !s.renotified && s.awaiting_elapsed >= GRACE_SECS {
                s.renotified = true;
                notify::reminder(app, sound);
            } else if s.awaiting_elapsed >= SKIP_AFTER_SECS {
                // Ignored for too long — skip this break and start over.
                s.start_working();
                tray_dirty = true;
            }
        }
        Phase::Break => {
            s.remaining -= 1;
            if s.remaining <= 0 {
                notify::break_over(app, sound);
                s.start_working();
                tray_dirty = true;
            }
        }
        Phase::Paused => {
            if !s.pause_indefinite {
                s.remaining -= 1;
                if s.remaining <= 0 {
                    s.start_working();
                    tray_dirty = true;
                }
            }
        }
    }

    let snap = s.snapshot();
    drop(s);

    let _ = app.emit("state-changed", &snap);
    tray::update_tooltip(app, &snap);
    if tray_dirty {
        tray::refresh(app);
    }
}
