//! 托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::{Emitter, Manager};

use crate::app_config::AppType;
use crate::store::AppState;

/// 托盘菜单文本（国际化）
#[derive(Clone, Copy)]
pub struct TrayTexts {
    pub show_main: &'static str,
    pub quit: &'static str,
}

impl TrayTexts {
    pub fn from_language(_language: &str) -> Self {
        Self {
            show_main: "打开主界面",
            quit: "退出",
        }
    }
}

/// 创建托盘菜单
pub fn create_tray_menu(app: &tauri::AppHandle, _app_state: &AppState) -> Result<Menu<tauri::Wry>, crate::error::AppError> {
    let settings = crate::settings::get_settings();
    let language = settings.language.as_deref().unwrap_or("zh");
    let texts = TrayTexts::from_language(language);

    let menu = MenuBuilder::new(app)
        .item(&MenuItem::with_id(app, "show", texts.show_main, true, None::<&str>)?)
        .separator()
        .item(&MenuItem::with_id(app, "quit", texts.quit, true, None::<&str>)?)
        .build()?;

    Ok(menu)
}

/// 处理托盘菜单事件
pub fn handle_tray_menu_event(app: &tauri::AppHandle, id: &str) {
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

/// 应用托盘策略（macOS 专用）
#[cfg(target_os = "macos")]
pub fn apply_tray_policy(_app: &tauri::AppHandle, _show_in_dock: bool) {
    // macOS specific implementation would go here
}

#[cfg(not(target_os = "macos"))]
pub fn apply_tray_policy(_app: &tauri::AppHandle, _show_in_dock: bool) {
    // No-op on other platforms
}