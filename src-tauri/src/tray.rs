use crate::state::{Phase, Snapshot};
use crate::timer::SharedState;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

/// Handles to the dynamic tray pieces we update as state changes.
pub struct TrayHandles {
    icon: TrayIcon,
    confirm: MenuItem<tauri::Wry>,
    resume: MenuItem<tauri::Wry>,
    snooze: MenuItem<tauri::Wry>,
}

/// Build the tray icon + menu and stash the dynamic handles in managed state.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let confirm = MenuItem::with_id(app, "confirm", "👀  I'm looking", false, None::<&str>)?;
    let pause_30 = MenuItem::with_id(app, "pause_30", "Pause 30 minutes", true, None::<&str>)?;
    let pause_60 = MenuItem::with_id(app, "pause_60", "Pause 1 hour", true, None::<&str>)?;
    let pause_inf = MenuItem::with_id(app, "pause_inf", "Pause until I resume", true, None::<&str>)?;
    let pause = Submenu::with_items(app, "Pause", true, &[&pause_30, &pause_60, &pause_inf])?;
    let resume = MenuItem::with_id(app, "resume", "Resume", false, None::<&str>)?;
    let snooze = MenuItem::with_id(app, "snooze", "Snooze 5 min", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&confirm, &pause, &resume, &snooze, &sep, &settings, &quit],
    )?;

    let icon = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(false)
        .tooltip("20-20-20 — starting…")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_icon_event)
        .build(app)?;

    app.manage(TrayHandles {
        icon,
        confirm,
        resume,
        snooze,
    });
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "confirm" => crate::commands::do_confirm(app),
        "pause_30" => crate::commands::do_pause(app, Some(30)),
        "pause_60" => crate::commands::do_pause(app, Some(60)),
        "pause_inf" => crate::commands::do_pause(app, None),
        "resume" => crate::commands::do_resume(app),
        "snooze" => crate::commands::do_snooze(app),
        "settings" => crate::commands::open_settings(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn on_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        // While a reminder is pending, a left-click confirms; otherwise it
        // opens the settings panel.
        let awaiting = {
            let s = app.state::<SharedState>();
            let guard = s.lock().unwrap();
            guard.phase == Phase::AwaitingConfirm
        };
        if awaiting {
            crate::commands::do_confirm(app);
        } else {
            crate::commands::open_settings(app);
        }
    }
}

/// Enable/disable the dynamic menu items and refresh the snooze label.
pub fn refresh(app: &AppHandle) {
    let handles = app.state::<TrayHandles>();
    let s = app.state::<SharedState>();
    let guard = s.lock().unwrap();
    let _ = handles.confirm.set_enabled(guard.phase == Phase::AwaitingConfirm);
    let _ = handles.resume.set_enabled(guard.phase == Phase::Paused);
    let _ = handles
        .snooze
        .set_text(format!("Snooze {} min", guard.settings.snooze_minutes));
}

/// Update the tray tooltip to reflect the current phase / countdown.
pub fn update_tooltip(app: &AppHandle, snap: &Snapshot) {
    let handles = app.state::<TrayHandles>();
    let text = match snap.phase {
        Phase::Working if snap.idle_paused => "20-20-20 — paused (you're idle)".to_string(),
        Phase::Working => format!("Next break in {}", fmt_mmss(snap.remaining)),
        Phase::AwaitingConfirm => "Look away! Click the icon to confirm 👀".to_string(),
        Phase::Break => format!("Look into the distance — {}s left", snap.remaining.max(0)),
        Phase::Paused => "20-20-20 — paused".to_string(),
    };
    let _ = handles.icon.set_tooltip(Some(text));
}

fn fmt_mmss(secs: i64) -> String {
    let secs = secs.max(0);
    format!("{}:{:02}", secs / 60, secs % 60)
}
