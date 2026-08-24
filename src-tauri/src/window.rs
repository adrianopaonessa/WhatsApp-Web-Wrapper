use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, WebviewWindow, Window, WindowEvent};

use crate::state;

pub const MAIN_WINDOW_LABEL: &str = "main";

static HIDDEN_IN_TRAY: AtomicBool = AtomicBool::new(false);
static WAS_MAXIMIZED: AtomicBool = AtomicBool::new(false);

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if HIDDEN_IN_TRAY.load(Ordering::SeqCst) {
            restore_from_tray(&window);
        } else {
            hide_to_tray(&window);
        }
    }
}

pub fn save_window_state(window: &WebviewWindow) {
    if let Ok(size) = window.inner_size() {
        state::update_geometry(
            window.app_handle(),
            size.width,
            size.height,
            window.is_maximized().unwrap_or(false),
        );
    }
}

pub fn hide_to_tray(window: &WebviewWindow) {
    save_window_state(window);
    WAS_MAXIMIZED.store(window.is_maximized().unwrap_or(false), Ordering::SeqCst);
    let _ = window.hide();
    HIDDEN_IN_TRAY.store(true, Ordering::SeqCst);
}

pub fn restore_from_tray(window: &WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    HIDDEN_IN_TRAY.store(false, Ordering::SeqCst);

    // KWin Wayland: force recalculation of decoration input regions
    #[cfg(target_os = "linux")]
    {
        let w = window.clone();
        let was_maximized = WAS_MAXIMIZED.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            if was_maximized {
                let _ = w.unmaximize();
                std::thread::sleep(std::time::Duration::from_millis(16));
                let _ = w.maximize();
            } else {
                let _ = w.maximize();
                std::thread::sleep(std::time::Duration::from_millis(16));
                let _ = w.unmaximize();
            }
        });
    }
}

/// Configure hardware and feature permissions (microphone, camera, location, notifications)
pub fn setup_webview_permissions(window: &WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::{
            NotificationExt, PermissionRequestExt, SettingsExt, UserContentInjectedFrames,
            UserContentManagerExt, UserScript, UserScriptInjectionTime, WebViewExt,
        };

        let w = window.clone();
        let _ = window.with_webview(move |webview| {
            let wv = webview.inner();

            if let Some(settings) = wv.settings() {
                settings.set_enable_media_stream(true);
                settings.set_enable_mediasource(true);
                settings.set_media_playback_requires_user_gesture(false);
            }

            // Inject notification bridge so WhatsApp auto-grants permissions and hooks click events
            if let Some(ucm) = wv.user_content_manager() {
                let notification_bridge = r#"
                    (function() {
                        if (typeof Notification === 'undefined') return;

                        window.__activeNotifications = {};
                        let notifCount = 0;

                        const OrigNotification = window.Notification;
                        function CustomNotification(title, options) {
                            options = options || {};
                            const id = ++notifCount;

                            this.title = title;
                            this.body = options.body || '';
                            this.icon = options.icon || '';
                            this.tag = options.tag || '';
                            this.onclick = null;
                            this.onclose = null;
                            this.onerror = null;
                            this.onshow = null;

                            const listeners = {};
                            this.addEventListener = function(type, listener) {
                                if (!listeners[type]) listeners[type] = [];
                                listeners[type].push(listener);
                            };
                            this.removeEventListener = function(type, listener) {
                                if (listeners[type]) {
                                    listeners[type] = listeners[type].filter(l => l !== listener);
                                }
                            };
                            this.dispatchEvent = function(event) {
                                if (listeners[event.type]) {
                                    listeners[event.type].forEach(cb => cb.call(this, event));
                                }
                                return true;
                            };

                            window.__activeNotifications[id] = this;

                            this.close = function() {
                                delete window.__activeNotifications[id];
                            };

                            try {
                                if (OrigNotification) {
                                    new OrigNotification(title, options);
                                }
                            } catch (e) {}
                        }

                        CustomNotification.permission = 'granted';
                        CustomNotification.requestPermission = function(callback) {
                            if (typeof callback === 'function') callback('granted');
                            return Promise.resolve('granted');
                        };

                        try {
                            Object.defineProperty(window, 'Notification', {
                                value: CustomNotification,
                                writable: true,
                                configurable: true
                            });
                        } catch (e) {
                            window.Notification = CustomNotification;
                        }
                    })();
                "#;

                let script = UserScript::new(
                    notification_bridge,
                    UserContentInjectedFrames::AllFrames,
                    UserScriptInjectionTime::Start,
                    &[],
                    &[],
                );
                ucm.add_script(&script);
            }

            // Automatically allow camera, mic, geolocation and notification requests
            wv.connect_permission_request(|_, request| {
                request.allow();
                true
            });

            // Handle native system notification with explicit "WhatsApp" branding and click-to-open
            let win_target = w.clone();
            wv.connect_show_notification(move |_wv, notification| {
                let title = notification.title().unwrap_or_default().to_string();
                let body = notification.body().unwrap_or_default().to_string();
                let notif_id = notification.id();
                let win = win_target.clone();

                std::thread::spawn(move || {
                    let _ = notify_rust::Notification::new()
                        .appname("WhatsApp")
                        .summary(&title)
                        .body(&body)
                        .icon("whatsapp-web-wrapper")
                        .action("default", "Open")
                        .show()
                        .map(|handle| {
                            handle.wait_for_action(|action| {
                                if action == "default" {
                                    restore_from_tray(&win);
                                    let js = format!(
                                        r#"
                                        (function() {{
                                            const notif = window.__activeNotifications && (window.__activeNotifications[{}] || Object.values(window.__activeNotifications).pop());
                                            if (notif) {{
                                                if (typeof notif.onclick === 'function') notif.onclick(new Event('click'));
                                                notif.dispatchEvent(new Event('click'));
                                            }}
                                        }})();
                                        "#,
                                        notif_id
                                    );
                                    let _ = win.eval(&js);
                                }
                            });
                        });
                });

                true
            });
        });
    }
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        if let Some(webview_window) = window.app_handle().get_webview_window(MAIN_WINDOW_LABEL) {
            hide_to_tray(&webview_window);
        }
    }
}
