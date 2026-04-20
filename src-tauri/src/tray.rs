//! 托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use std::collections::HashMap;

use tauri::menu::{Menu, MenuBuilder, MenuItem, Submenu, SubmenuBuilder};
use tauri::{Emitter, Manager};

use crate::database::ToolDao;
use crate::store::AppState;

/// 工具类型的菜单元数据
struct ToolTypeMenu {
    type_key: &'static str,
    label: &'static str,
    icon: &'static str,
}

impl ToolTypeMenu {
    const ALL: [ToolTypeMenu; 3] = [
        ToolTypeMenu {
            type_key: "shell",
            label: "Shell 工具",
            icon: "🔧",
        },
        ToolTypeMenu {
            type_key: "open",
            label: "Open 工具",
            icon: "📂",
        },
        ToolTypeMenu {
            type_key: "notification",
            label: "Notification 工具",
            icon: "🔔",
        },
    ];
}

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

/// 构建工具子菜单
fn build_tools_submenu(
    app: &tauri::AppHandle,
    _type_key: &str,
    label: &str,
    icon: &str,
    tools: &[crate::database::Tool],
) -> Result<Submenu<tauri::Wry>, crate::error::AppError> {
    let menu_label = format!("{icon} {label}");
    let mut builder = SubmenuBuilder::new(app, &menu_label);

    if tools.is_empty() {
        let disabled_item = MenuItem::with_id(app, "no-tools", "（无可用工具）", false, None::<&str>)?;
        builder = builder.item(&disabled_item);
    } else {
        for tool in tools {
            let item_id = format!("tool:{}", tool.id);
            builder = builder.item(&MenuItem::with_id(
                app,
                &item_id,
                &tool.name,
                true,
                None::<&str>,
            )?);
        }
    }

    builder
        .build()
        .map_err(|e| crate::error::AppError::Message(e.to_string()))
}

/// 创建托盘菜单
pub fn create_tray_menu(
    app: &tauri::AppHandle,
    app_state: &AppState,
) -> Result<Menu<tauri::Wry>, crate::error::AppError> {
    let settings = crate::settings::get_settings();
    let language = settings.language.as_deref().unwrap_or("zh");
    let texts = TrayTexts::from_language(language);

    // Load enabled tools from database
    let tools: Vec<crate::database::Tool> = app_state
        .db
        .with_tools_dao(|conn| {
            let all: Vec<crate::database::Tool> = ToolDao::list(conn)?;
            Ok(all)
        })?
        .into_iter()
        .filter(|t| t.enabled)
        .collect();

    // Group tools by type
    let mut grouped: HashMap<&str, Vec<crate::database::Tool>> = HashMap::new();
    for tool in &tools {
        grouped.entry(&tool.tool_type).or_default().push(tool.clone());
    }

    let mut builder = MenuBuilder::new(app);
    builder = builder.item(&MenuItem::with_id(
        app,
        "show",
        texts.show_main,
        true,
        None::<&str>,
    )?);

    // Add submenus for each known tool type
    for type_meta in ToolTypeMenu::ALL {
        if let Some(type_tools) = grouped.get(type_meta.type_key) {
            let submenu = build_tools_submenu(
                app,
                type_meta.type_key,
                type_meta.label,
                type_meta.icon,
                type_tools,
            )?;
            builder = builder.item(&submenu);
        }
    }

    builder = builder.separator().item(&MenuItem::with_id(
        app,
        "quit",
        texts.quit,
        true,
        None::<&str>,
    )?);

    Ok(builder.build()?)
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
        _ => {
            // Handle tool clicks: "tool:{tool_id}"
            if let Some(tool_id) = id.strip_prefix("tool:") {
                let payload = serde_json::json!({ "toolId": tool_id });
                let _ = app.emit("open_param_dialog", payload);
            }
        }
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