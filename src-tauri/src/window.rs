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

pub fn mark_hidden_in_tray() {
    HIDDEN_IN_TRAY.store(true, Ordering::SeqCst);
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

#[derive(serde::Deserialize, Debug)]
struct NotificationPayload {
    id: u64,
    title: String,
    body: String,
}

pub const NOTIFICATION_SCRIPT: &str = r#"
(function() {
    let notifCount = 0;
    window.__activeNotifications = window.__activeNotifications || {};

    class CustomNotification extends EventTarget {
        constructor(title, options = {}) {
            super();
            this.id = ++notifCount;
            this.title = String(title || 'WhatsApp');
            this.body = String(options.body || '');
            this.icon = String(options.icon || '');
            this.tag = String(options.tag || '');
            this.data = options.data || null;
            this.silent = !!options.silent;
            this.timestamp = options.timestamp || Date.now();
            this.onclick = null;
            this.onclose = null;
            this.onerror = null;
            this.onshow = null;

            window.__activeNotifications[this.id] = this;

            try {
                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.notify) {
                    window.webkit.messageHandlers.notify.postMessage(JSON.stringify({
                        id: this.id,
                        title: this.title,
                        body: this.body
                    }));
                }
            } catch (e) {
                console.error('Failed to post notification to host:', e);
            }

            setTimeout(() => {
                if (typeof this.onshow === 'function') this.onshow(new Event('show'));
                this.dispatchEvent(new Event('show'));
            }, 10);
        }

        close() {
            delete window.__activeNotifications[this.id];
            if (typeof this.onclose === 'function') this.onclose(new Event('close'));
            this.dispatchEvent(new Event('close'));
        }

        static get permission() {
            return 'granted';
        }

        static requestPermission(callback) {
            if (typeof callback === 'function') callback('granted');
            return Promise.resolve('granted');
        }

        static get maxActions() {
            return 2;
        }
    }

    window.Notification = CustomNotification;

    if (typeof ServiceWorkerRegistration !== 'undefined') {
        ServiceWorkerRegistration.prototype.showNotification = function(title, options) {
            try {
                new CustomNotification(title, options);
            } catch (e) {
                console.error('SW showNotification error:', e);
            }
            return Promise.resolve();
        };
        ServiceWorkerRegistration.prototype.getNotifications = function() {
            return Promise.resolve(Object.values(window.__activeNotifications || {}));
        };
    }

    if (navigator.permissions && navigator.permissions.query) {
        const originalQuery = navigator.permissions.query.bind(navigator.permissions);
        navigator.permissions.query = function(queryObj) {
            if (queryObj && queryObj.name === 'notifications') {
                return Promise.resolve({
                    name: 'notifications',
                    state: 'granted',
                    status: 'granted',
                    onchange: null,
                    addEventListener: function() {},
                    removeEventListener: function() {},
                    dispatchEvent: function() { return false; }
                });
            }
            return originalQuery(queryObj);
        };
    }

    window.__triggerNotificationClick = function(id) {
        const notif = window.__activeNotifications[id] || Object.values(window.__activeNotifications).pop();
        if (notif) {
            if (typeof notif.onclick === 'function') notif.onclick(new Event('click'));
            notif.dispatchEvent(new Event('click'));
        }
    };
})();
"#;

pub fn setup_webview_permissions(window: &WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::{
            JavascriptResult, PermissionRequestExt, UserContentInjectedFrames,
            UserContentManagerExt, UserScript, UserScriptInjectionTime, WebViewExt,
        };

        let w = window.clone();
        let _ = window.with_webview(move |webview| {
            let wv = webview.inner();

            if let Some(ucm) = wv.user_content_manager() {
                let script = UserScript::new(
                    NOTIFICATION_SCRIPT,
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
                            let parsed: Result<NotificationPayload, _> =
                                serde_json::from_str(&json_str).or_else(|_| {
                                    serde_json::from_str::<String>(&json_str).and_then(
                                        |unquoted| {
                                            serde_json::from_str::<NotificationPayload>(&unquoted)
                                        },
                                    )
                                });

                            if let Ok(payload) = parsed {
                                let win = win_target.clone();
                                let notif_id = payload.id;
                                std::thread::spawn(move || {
                                    let mut notif = notify_rust::Notification::new();
                                    notif.appname("WhatsApp")
                                        .summary(&payload.title)
                                        .body(&payload.body)
                                        .icon("whatsapp-web-wrapper")
                                        .action("default", "Open");

                                    if let Ok(handle) = notif.show() {
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
                                    }
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
