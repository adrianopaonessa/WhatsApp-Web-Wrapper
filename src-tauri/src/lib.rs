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

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            let should_hide = (is_autostart && app_state.start_in_background_on_boot)
                || (!is_autostart && app_state.start_in_background_on_manual_launch);

            // If we should start hidden, set visible=false on the builder
            // to avoid a flash of the window before hide_to_tray() runs.
            let mut builder =
                WebviewWindowBuilder::from_config(app.handle(), &app.config().app.windows[0])?
                    .initialization_script(window::NOTIFICATION_SCRIPT);

            // Apply saved size before building, so Tauri sets it at creation time
            if app_state.maximized {
                builder = builder.maximized(true);
            } else {
                builder = builder
                    .inner_size(app_state.width as f64, app_state.height as f64);
            }

            // If we need to start hidden, tell the OS not to show the window at all
            if should_hide {
                builder = builder.visible(false);
            }

            let win = builder.build()?;

            // Grant hardware and notification permissions
            window::setup_webview_permissions(&win);

            // If starting hidden, mark it in our HIDDEN_IN_TRAY flag
            // and hide from taskbar (skip_taskbar)
            if should_hide {
                window::mark_hidden_in_tray();
            }

            Ok(())
        })
        .on_window_event(window::handle_window_event)
        .run(tauri::generate_context!())
        .expect("error while running application");
}
