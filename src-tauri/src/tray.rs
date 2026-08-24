use tauri::{
    image::Image,
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};
use tauri_plugin_autostart::ManagerExt;

use crate::{state, window};

const MENU_ITEM_TOGGLE: &str = "toggle";
const MENU_ITEM_AUTOSTART: &str = "autostart";
const MENU_ITEM_BACKGROUND_ON_BOOT: &str = "background_on_boot";
const MENU_ITEM_BACKGROUND_ON_MANUAL: &str = "background_on_manual";
const MENU_ITEM_DARK_TRAY_ICON: &str = "dark_tray_icon";
const MENU_ITEM_QUIT: &str = "quit";

const TRAY_ICON_WHITE: &[u8] = include_bytes!("../icons/icon-tray-white.png");
const TRAY_ICON_DARK: &[u8] = include_bytes!("../icons/icon-tray-dark.png");

fn load_tray_icon(dark: bool) -> Image<'static> {
    let bytes = if dark {
        TRAY_ICON_DARK
    } else {
        TRAY_ICON_WHITE
    };
    Image::from_bytes(bytes).expect("failed to parse embedded tray icon")
}

pub fn setup(app: &App) -> Result<(), tauri::Error> {
    let app_state = state::load(app.handle());
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let toggle_item = MenuItemBuilder::with_id(MENU_ITEM_TOGGLE, "Show/Hide").build(app)?;
    let autostart_item = CheckMenuItemBuilder::with_id(MENU_ITEM_AUTOSTART, "Start on Boot")
        .checked(autostart_enabled)
        .build(app)?;
    let background_on_boot_item =
        CheckMenuItemBuilder::with_id(MENU_ITEM_BACKGROUND_ON_BOOT, "Start in Background on Boot")
            .checked(app_state.start_in_background_on_boot)
            .build(app)?;
    let background_on_manual_item = CheckMenuItemBuilder::with_id(
        MENU_ITEM_BACKGROUND_ON_MANUAL,
        "Start in Background on Manual Launch",
    )
    .checked(app_state.start_in_background_on_manual_launch)
    .build(app)?;
    let dark_tray_icon_item =
        CheckMenuItemBuilder::with_id(MENU_ITEM_DARK_TRAY_ICON, "Dark Tray Icon")
            .checked(app_state.dark_tray_icon)
            .build(app)?;
    let quit_item = MenuItemBuilder::with_id(MENU_ITEM_QUIT, "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle_item)
        .separator()
        .item(&autostart_item)
        .item(&background_on_boot_item)
        .item(&background_on_manual_item)
        .separator()
        .item(&dark_tray_icon_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let icon = load_tray_icon(app_state.dark_tray_icon);

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("WhatsApp")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_ITEM_TOGGLE => {
                window::toggle_main_window(app);
            }
            MENU_ITEM_AUTOSTART => {
                let autolaunch = app.autolaunch();
                if autolaunch.is_enabled().unwrap_or(false) {
                    let _ = autolaunch.disable();
                } else {
                    let _ = autolaunch.enable();
                }
            }
            MENU_ITEM_BACKGROUND_ON_BOOT => {
                let current_state = state::load(app);
                state::set_start_in_background_on_boot(
                    app,
                    !current_state.start_in_background_on_boot,
                );
            }
            MENU_ITEM_BACKGROUND_ON_MANUAL => {
                let current_state = state::load(app);
                state::set_start_in_background_on_manual_launch(
                    app,
                    !current_state.start_in_background_on_manual_launch,
                );
            }
            MENU_ITEM_DARK_TRAY_ICON => {
                let current_state = state::load(app);
                let new_value = !current_state.dark_tray_icon;
                state::set_dark_tray_icon(app, new_value);

                if let Some(tray) = app.tray_by_id("main-tray") {
                    let _ = tray.set_icon(Some(load_tray_icon(new_value)));
                }
            }
            MENU_ITEM_QUIT => {
                if let Some(w) = app.get_webview_window(window::MAIN_WINDOW_LABEL) {
                    window::save_window_state(&w);
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
