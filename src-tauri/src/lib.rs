mod commands;
mod idle;
mod notify;
mod settings;
mod state;
mod timer;
mod tray;

use settings::Settings;
use state::AppState;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use timer::SharedState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            // Brand notifications (Windows toasts say "Eye Break").
            notify::init_branding(&handle);

            // Load settings and seed the shared state, then build the tray.
            let settings = Settings::load(&handle);
            app.manage(SharedState::new(AppState::new(settings)));
            tray::build(&handle)?;
            tray::refresh(&handle);

            // Drive the 20-20-20 cycle on a 1 Hz background thread.
            timer::spawn(handle.clone());

            // Closing the settings window hides it instead of quitting; the
            // app keeps running in the tray.
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::get_state,
            commands::save_settings,
            commands::confirm_looking,
            commands::look_away_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
