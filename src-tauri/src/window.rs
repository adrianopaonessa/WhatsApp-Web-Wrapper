use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, WebviewWindow, Window, WindowEvent};

use crate::state;

pub const MAIN_WINDOW_LABEL: &str = "main";

static HIDDEN_IN_TRAY: AtomicBool = AtomicBool::new(false);

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
    let _ = window.set_skip_taskbar(true);
    let _ = window.minimize();
    HIDDEN_IN_TRAY.store(true, Ordering::SeqCst);
}

pub fn restore_from_tray(window: &WebviewWindow) {
    let _ = window.set_skip_taskbar(false);
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    HIDDEN_IN_TRAY.store(false, Ordering::SeqCst);

    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::WidgetExt;
        let _ = window.with_webview(|webview| {
            let wv = webview.inner();
            wv.queue_draw();
        });
    }
}

#[derive(serde::Deserialize)]
struct NotificationPayload {
    id: u64,
    title: String,
    body: String,
}

pub fn setup_webview_permissions(window: &WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::{
            JavascriptResult, PermissionRequestExt, SettingsExt, UserContentInjectedFrames,
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

            if let Some(ucm) = wv.user_content_manager() {
                let script_content = r#"
                    (function() {
                        try {
                            Object.defineProperty(document, 'hidden', {
                                get: () => false,
                                configurable: true
                            });
                            Object.defineProperty(document, 'visibilityState', {
                                get: () => 'visible',
                                configurable: true
                            });
                            Object.defineProperty(document, 'webkitVisibilityState', {
                                get: () => 'visible',
                                configurable: true
                            });
                        } catch (e) {}

                        window.__activeNotifications = {};
                        let notifCount = 0;

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
                                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.notify) {
                                    window.webkit.messageHandlers.notify.postMessage(JSON.stringify({
                                        id: id,
                                        title: title || 'WhatsApp',
                                        body: options.body || ''
                                    }));
                                }
                            } catch (e) {
                                console.error('Failed to post notification message to host:', e);
                            }
                        }

                        CustomNotification.permission = 'granted';
                        CustomNotification.requestPermission = function(callback) {
                            if (typeof callback === 'function') callback('granted');
                            return Promise.resolve('granted');
                        };

                        window.Notification = CustomNotification;
                        try {
                            Object.defineProperty(window, 'Notification', {
                                value: CustomNotification,
                                writable: true,
                                configurable: true
                            });
                        } catch (e) {}

                        window.__triggerNotificationClick = function(id) {
                            const notif = window.__activeNotifications[id] || Object.values(window.__activeNotifications).pop();
                            if (notif) {
                                if (typeof notif.onclick === 'function') notif.onclick(new Event('click'));
                                notif.dispatchEvent(new Event('click'));
                            }
                        };
                    })();
                "#;

                let script = UserScript::new(
                    script_content,
                    UserContentInjectedFrames::AllFrames,
                    UserScriptInjectionTime::Start,
                    &[],
                    &[],
                );
                ucm.add_script(&script);

                ucm.register_script_message_handler("notify");
                let win_target = w.clone();
                ucm.connect_script_message_received(
                    Some("notify"),
                    move |_ucm, js_result: &JavascriptResult| {
                        if let Some(js_val) = js_result.js_value() {
                            let json_str = js_val.to_string();
                            if let Ok(payload) =
                                serde_json::from_str::<NotificationPayload>(&json_str)
                            {
                                let win = win_target.clone();
                                let notif_id = payload.id;
                                std::thread::spawn(move || {
                                    let _ = notify_rust::Notification::new()
                                        .appname("WhatsApp")
                                        .summary(&payload.title)
                                        .body(&payload.body)
                                        .icon("whatsapp-web-wrapper")
                                        .action("default", "Open")
                                        .show()
                                        .map(|handle| {
                                            handle.wait_for_action(|action| {
                                                if action == "default" {
                                                    restore_from_tray(&win);
                                                    let js = format!(
                                                        "if (window.__triggerNotificationClick) window.__triggerNotificationClick({});",
                                                        notif_id
                                                    );
                                                    let _ = win.eval(&js);
                                                }
                                            });
                                        });
                                });
                            }
                        }
                    },
                );
            }

            wv.connect_permission_request(|_, request| {
                request.allow();
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
