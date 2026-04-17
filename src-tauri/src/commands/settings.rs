use crate::settings::{get_settings as get_settings_func, update_settings, AppSettings};
use tauri::command;

#[command]
pub fn get_settings() -> Result<AppSettings, String> {
    Ok(get_settings_func())
}

#[command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    update_settings(settings).map_err(|e| e.to_string())
}

#[command]
pub fn set_auto_launch(enabled: bool) -> Result<(), String> {
    if enabled {
        crate::auto_launch::enable_auto_launch().map_err(|e| e.to_string())
    } else {
        crate::auto_launch::disable_auto_launch().map_err(|e| e.to_string())
    }
}

#[command]
pub fn get_auto_launch_status() -> Result<bool, String> {
    crate::auto_launch::is_auto_launch_enabled().map_err(|e| e.to_string())
}

#[command]
pub fn set_window_theme(theme: String) -> Result<(), String> {
    log::info!("set_window_theme called with: {}", theme);
    Ok(())
}