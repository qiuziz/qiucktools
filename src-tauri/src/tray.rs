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
fn build_tools_submenu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    _type_key: &str,
    label: &str,
    icon: &str,
    tools: &[crate::database::Tool],
) -> Result<Submenu<R>, crate::error::AppError> {
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
pub fn create_tray_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    app_state: &AppState,
) -> Result<Menu<R>, crate::error::AppError> {
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

    // Warn about tools with unknown types that won't appear in the tray menu
    for type_key in grouped.keys() {
        if !ToolTypeMenu::ALL.iter().any(|m| m.type_key == *type_key) {
            log::warn!(
                "Unknown tool type '{}' for tool '{}', skipping tray menu item",
                type_key,
                grouped[type_key]
                    .iter()
                    .map(|t| t.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
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
pub fn handle_tray_menu_event<R: tauri::Runtime>(app: &tauri::AppHandle<R>, id: &str) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tauri::Listener;

    fn make_tool(id: &str, name: &str, tool_type: &str) -> crate::database::Tool {
        crate::database::Tool {
            id: id.to_string(),
            name: name.to_string(),
            icon: "test-icon".to_string(),
            description: None,
            tool_type: tool_type.to_string(),
            command: "echo test".to_string(),
            working_dir: "".to_string(),
            timeout_ms: 5000,
            params: vec![],
            sort_order: 0,
            enabled: true,
        }
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "muda MenuChild requires main thread on macOS")]
    fn test_build_tools_submenu_multiple_tools() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();
        let tools = vec![
            make_tool("shell-echo", "Echo", "shell"),
            make_tool("shell-date", "Date", "shell"),
        ];

        let result = build_tools_submenu(&app_handle, "shell", "Shell 工具", "🔧", &tools);
        assert!(result.is_ok(), "should build submenu with multiple tools: {:?}", result.err());
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "muda MenuChild requires main thread on macOS")]
    fn test_build_tools_submenu_empty() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();
        let tools: Vec<crate::database::Tool> = vec![];

        let result = build_tools_submenu(&app_handle, "shell", "Shell 工具", "🔧", &tools);
        assert!(result.is_ok(), "should build submenu with disabled placeholder: {:?}", result.err());
    }

    #[test]
    fn test_handle_tray_menu_event_tool_pattern() {
        // Use a channel to capture the emitted event from a background thread
        let (tx, rx) = mpsc::channel();
        let tx_thread = std::sync::Mutex::new(Some(tx));

        let app = tauri::test::mock_app();
        let app_handle = app.handle();

        // Register listener on the App to capture the event
        app.listen("open_param_dialog", move |event| {
            if let Some(tx) = tx_thread.lock().unwrap().take() {
                // event.payload() returns &str containing JSON
                let payload_str = event.payload();
                let parsed: serde_json::Value = serde_json::from_str(payload_str).unwrap();
                let _ = tx.send(parsed);
            }
        });

        // Spawn a thread to call the handler; emit() is async in Tauri 2
        let handle_clone = app_handle.clone();
        let jh = thread::spawn(move || {
            handle_tray_menu_event(&handle_clone, "tool:my-tool-id");
        });

        // Wait up to 2s for the event to arrive via the channel
        let payload = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("event should be emitted within 2s");

        assert_eq!(
            payload,
            serde_json::json!({ "toolId": "my-tool-id" }),
            "tool:{{id}} pattern should emit correct {{toolId}} payload"
        );

        let _ = jh.join();
    }

    #[test]
    fn test_tool_type_menu_all_has_expected_types() {
        let type_keys: Vec<&str> = ToolTypeMenu::ALL.iter().map(|m| m.type_key).collect();
        assert!(type_keys.contains(&"shell"), "ALL should contain 'shell'");
        assert!(type_keys.contains(&"open"), "ALL should contain 'open'");
        assert!(type_keys.contains(&"notification"), "ALL should contain 'notification'");
        assert_eq!(type_keys.len(), 3, "ALL should have exactly 3 entries");
    }
}