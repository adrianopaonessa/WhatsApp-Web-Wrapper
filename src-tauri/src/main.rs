#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::glib;
        glib::set_application_name("WhatsApp");
        glib::set_prgname(Some("WhatsApp"));

        unsafe {
            // Essential for Tauri WebKitGTK on Wayland to prevent white screen freezing
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    whatsapp_web_wrapper_lib::run();
}
