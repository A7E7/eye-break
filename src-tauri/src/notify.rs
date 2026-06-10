//! Reminder notifications.
//!
//! On Windows we use `winrt-toast-reborn` so the "look away" toast can carry a
//! real **"I'm looking"** action button whose in-process callback confirms the
//! break (the app is always running in the tray, so the callback fires
//! directly). On other platforms `tauri-plugin-notification` shows a plain
//! toast and confirmation happens via the tray menu.

use tauri::AppHandle;

const REMINDER_TITLE: &str = "Time for an eye break 👀";
const REMINDER_BODY: &str = "Look ~20 ft (6 m) into the distance for a moment.";
const DONE_TITLE: &str = "Break complete ✅";
const DONE_BODY: &str = "You can look back at your screen.";

/// The "look away now" reminder (carries the confirm button on Windows).
pub fn reminder(app: &AppHandle, sound: bool) {
    #[cfg(target_os = "windows")]
    windows::show(app, REMINDER_TITLE, REMINDER_BODY, true, sound);
    #[cfg(not(target_os = "windows"))]
    plugin::show(app, REMINDER_TITLE, REMINDER_BODY, sound);
}

/// The "break is over" notification.
pub fn break_over(app: &AppHandle, sound: bool) {
    #[cfg(target_os = "windows")]
    windows::show(app, DONE_TITLE, DONE_BODY, false, sound);
    #[cfg(not(target_os = "windows"))]
    plugin::show(app, DONE_TITLE, DONE_BODY, sound);
}

#[cfg(not(target_os = "windows"))]
mod plugin {
    use super::*;
    use tauri_plugin_notification::NotificationExt;

    pub fn show(app: &AppHandle, title: &str, body: &str, sound: bool) {
        let mut builder = app.notification().builder().title(title).body(body);
        if sound {
            builder = builder.sound("default");
        }
        let _ = builder.show();
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use winrt_toast_reborn::content::audio::{Audio, LoopingSound, Sound};
    use winrt_toast_reborn::{Action, ActivatedAction, Scenario, Toast, ToastManager};

    pub fn show(app: &AppHandle, title: &str, body: &str, with_button: bool, sound: bool) {
        // Using the well-known PowerShell AUMID guarantees the toast is shown
        // for an unpackaged/dev build. When packaged with a Start-Menu
        // shortcut, swap this for the app's own AUMID for proper branding.
        let manager = ToastManager::new(ToastManager::POWERSHELL_AUM_ID);

        let manager = if with_button {
            let app = app.clone();
            // Fires for either the button or a click on the toast body.
            manager.on_activated(None, move |_action: Option<ActivatedAction>| {
                crate::commands::do_confirm(&app);
            })
        } else {
            manager
        };

        let mut toast = Toast::new();
        toast.text1(title).text2(body);
        if with_button {
            // Reminder scenario keeps the toast on screen (pre-expanded) until
            // the user acts, so the "look away" prompt can't be missed.
            toast.scenario(Scenario::Reminder);
            toast.action(Action::new("I'm looking", "confirm", "confirm"));
        }
        if !sound {
            // Silence by attaching a muted audio element.
            toast.audio(Audio::new(Sound::Looping(LoopingSound::Alarm5)).with_silent());
        }

        let _ = manager.show(&toast);
    }
}
