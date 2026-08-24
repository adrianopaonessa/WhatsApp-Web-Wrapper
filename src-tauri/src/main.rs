#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::glib;
        glib::set_application_name("WhatsApp");
        glib::set_prgname(Some("WhatsApp"));

        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            // SAFETY: Process entry point before any threads are spawned
            unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
        }
    }

    whatsapp_web_wrapper_lib::run();
}
