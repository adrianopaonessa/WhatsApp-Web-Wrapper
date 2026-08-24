mod state;
mod tray;
mod window;

use tauri::WebviewWindowBuilder;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tray::setup(app)?;

            let app_state = state::load(app.handle());

            // Build the main window programmatically with initialization_script
            let win = WebviewWindowBuilder::from_config(app.handle(), &app.config().app.windows[0])?
                .initialization_script(window::NOTIFICATION_SCRIPT)
                .build()?;

            // Grant hardware and notification permissions
            window::setup_webview_permissions(&win);

            if app_state.maximized {
                let _ = win.maximize();
            } else {
                let _ = win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width: app_state.width,
                    height: app_state.height,
                }));
            }

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            let should_hide = (is_autostart && app_state.start_in_background_on_boot)
                || (!is_autostart && app_state.start_in_background_on_manual_launch);

            if should_hide {
                window::hide_to_tray(&win);
            }

            Ok(())
        })
        .on_window_event(window::handle_window_event)
        .run(tauri::generate_context!())
        .expect("error while running application");
}
