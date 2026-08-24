use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const STATE_FILE: &str = "window-state.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub maximized: bool,
    #[serde(default = "default_true")]
    pub start_in_background_on_boot: bool,
    #[serde(default)]
    pub start_in_background_on_manual_launch: bool,
    #[serde(default)]
    pub dark_tray_icon: bool,
}

fn default_width() -> u32 {
    1100
}

fn default_height() -> u32 {
    750
}

fn default_true() -> bool {
    true
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            maximized: false,
            start_in_background_on_boot: true,
            start_in_background_on_manual_launch: false,
            dark_tray_icon: false,
        }
    }
}

fn state_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join(STATE_FILE))
}

pub fn load(app: &AppHandle) -> AppState {
    if let Some(path) = state_path(app) {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str(&data) {
                return state;
            }
        }
    }
    AppState::default()
}

pub fn save(app: &AppHandle, state: &AppState) {
    if let Some(path) = state_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(state) {
            let _ = fs::write(path, json);
        }
    }
}

pub fn update_geometry(app: &AppHandle, width: u32, height: u32, maximized: bool) {
    let mut state = load(app);
    state.width = width;
    state.height = height;
    state.maximized = maximized;
    save(app, &state);
}

pub fn set_start_in_background_on_boot(app: &AppHandle, value: bool) {
    let mut state = load(app);
    state.start_in_background_on_boot = value;
    save(app, &state);
}

pub fn set_start_in_background_on_manual_launch(app: &AppHandle, value: bool) {
    let mut state = load(app);
    state.start_in_background_on_manual_launch = value;
    save(app, &state);
}

pub fn set_dark_tray_icon(app: &AppHandle, value: bool) {
    let mut state = load(app);
    state.dark_tray_icon = value;
    save(app, &state);
}
