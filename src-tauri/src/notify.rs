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

/// One-time setup so Windows toasts are branded as "Eye Break" rather than
/// attributed to a generic host. No-op on other platforms.
pub fn init_branding(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    windows::register(app);
    #[cfg(not(target_os = "windows"))]
    let _ = app;
}

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
    use std::path::PathBuf;
    use tauri::Manager;
    use winrt_toast_reborn::content::audio::{Audio, LoopingSound, Sound};
    use winrt_toast_reborn::content::image::ImagePlacement;
    use winrt_toast_reborn::{Action, ActivatedAction, Image, Scenario, Toast, ToastManager};

    /// Our Application User Model ID — the identity Windows uses to brand toasts.
    /// Windows caches the toast icon per-AUMID, so this is versioned: bump the
    /// suffix if the icon ever needs to change and won't refresh.
    const AUMID: &str = "EyeBreak.App.1";
    const DISPLAY_NAME: &str = "Eye Break";

    /// Make Windows treat us as a branded toast source: register our display
    /// name + icon, tag the process with the AUMID, and ensure a Start-Menu
    /// shortcut carrying it exists (also a proper launch entry). Idempotent.
    pub fn register(app: &AppHandle) {
        // Remove the earlier experimental registration, if present.
        let _ = winrt_toast_reborn::unregister("com.eyebreak.app");

        set_process_aumid();

        // Register display name + icon (writes HKCU\Software\Classes\
        // AppUserModelId\{AUMID} with DisplayName + IconUri).
        let icon = logo_png(app);
        let _ = winrt_toast_reborn::register(AUMID, DISPLAY_NAME, icon.as_deref());

        if let Err(e) = ensure_shortcut(app) {
            eprintln!("[eye-break] shortcut registration failed: {e:?}");
        }
    }

    fn set_process_aumid() {
        use ::windows::core::HSTRING;
        use ::windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
        unsafe {
            let _ = SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(AUMID));
        }
    }

    fn shortcut_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            PathBuf::from(appdata)
                .join(r"Microsoft\Windows\Start Menu\Programs")
                .join(format!("{DISPLAY_NAME}.lnk"))
        })
    }

    /// Drop the app icon (.ico) on disk so the shortcut can reference a real file.
    fn icon_file(app: &AppHandle) -> Option<PathBuf> {
        let dir = app.path().app_local_data_dir().ok()?;
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("eye-break.ico");
        std::fs::write(&path, include_bytes!("../icons/icon.ico"))
            .ok()
            .map(|_| path)
    }

    /// Drop a PNG logo on disk for the toast's app-logo image (the icon shown
    /// on the notification itself).
    fn logo_png(app: &AppHandle) -> Option<PathBuf> {
        let dir = app.path().app_local_data_dir().ok()?;
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("eye-break-logo.png");
        if !path.exists() {
            std::fs::write(&path, include_bytes!("../icons/128x128.png")).ok()?;
        }
        Some(path)
    }

    /// Create the Start-Menu shortcut with the AUMID property if it's missing.
    fn ensure_shortcut(app: &AppHandle) -> ::windows::core::Result<()> {
        use ::windows::core::{Interface, GUID, HSTRING};
        use ::windows::Win32::Foundation::PROPERTYKEY;
        use ::windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
        use ::windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use ::windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
        use ::windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

        let Some(lnk) = shortcut_path() else {
            return Ok(());
        };
        let Ok(exe) = std::env::current_exe() else {
            return Ok(());
        };

        // PKEY_AppUserModel_ID = {9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}, pid 5.
        const PKEY_APP_USER_MODEL_ID: PROPERTYKEY = PROPERTYKEY {
            fmtid: GUID::from_u128(0x9F4C2855_9F79_4B39_A8D0_E1D42DE1D5F3),
            pid: 5,
        };

        unsafe {
            // Tauri's main thread is already an STA; ignore "already initialized".
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            link.SetPath(&HSTRING::from(exe.to_string_lossy().as_ref()))?;
            if let Some(ico) = icon_file(app) {
                let _ = link.SetIconLocation(&HSTRING::from(ico.to_string_lossy().as_ref()), 0);
            }

            let store: IPropertyStore = link.cast()?;
            store.SetValue(&PKEY_APP_USER_MODEL_ID, &PROPVARIANT::from(AUMID))?;
            store.Commit()?;

            let persist: IPersistFile = link.cast()?;
            persist.Save(&HSTRING::from(lnk.to_string_lossy().as_ref()), true)?;
        }
        Ok(())
    }

    pub fn show(app: &AppHandle, title: &str, body: &str, with_button: bool, sound: bool) {
        // Toasts are attributed to our registered AUMID (see `register`).
        let manager = ToastManager::new(AUMID);

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
        // Show the eyes icon on the toast itself (don't rely on AUMID lookup).
        if let Some(logo) = logo_png(app) {
            if let Ok(image) = Image::new_local(logo) {
                toast.image(1, image.with_placement(ImagePlacement::AppLogoOverride));
            }
        }
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
