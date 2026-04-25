//! 托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::{Emitter, Manager};

use crate::store::AppState;

const TOOLS_JSON_PATH: &str = "~/work/quicktools/tools.json";

/// 与 tools.json 格式匹配的 Tool 结构（JSON 用 "type"，非 "toolType"）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolForTray {
    id: String,
    name: String,
    #[serde(rename = "type")]
    tool_type: String,
    #[serde(default)]
    params: Vec<serde_json::Value>,
    enabled: Option<bool>,
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

fn tool_menu_item<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    tool: &ToolForTray,
) -> Result<MenuItem<R>, crate::error::AppError> {
    let item_id = format!("tool:{}", tool.id);
    Ok(MenuItem::with_id(
        app,
        &item_id,
        &tool.name,
        true,
        None::<&str>,
    )?)
}

fn no_tools_menu_item<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<MenuItem<R>, crate::error::AppError> {
    Ok(MenuItem::with_id(
        app,
        "no-tools",
        "（无可用工具）",
        false,
        None::<&str>,
    )?)
}

/// 从 tools.json 直接加载工具（跳过数据库，启动时数据库可能还是空的）
fn load_tools_from_json() -> Vec<ToolForTray> {
    let path = expand_home(TOOLS_JSON_PATH);
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Vec<ToolForTray>>(&content) {
            Ok(tools) => tools,
            Err(e) => {
                log::warn!("Failed to parse {}: {e}", path.display());
                vec![]
            }
        },
        Err(e) => {
            log::warn!("Failed to read {}: {e}", path.display());
            vec![]
        }
    }
}

/// 无参数工具可从托盘直接执行；有参数工具才需要打开参数弹窗。
pub fn tool_needs_param_dialog(tool_id: &str) -> bool {
    load_tools_from_json()
        .into_iter()
        .find(|tool| tool.id == tool_id)
        .map(|tool| !tool.params.is_empty())
        .unwrap_or(true)
}

fn expand_home(path: &str) -> std::path::PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.trim_start_matches("~/"));
        }
    }
    std::path::PathBuf::from(path)
}

/// 创建托盘菜单
pub fn create_tray_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    _app_state: &AppState,
) -> Result<Menu<R>, crate::error::AppError> {
    let settings = crate::settings::get_settings();
    let language = settings.language.as_deref().unwrap_or("zh");
    let texts = TrayTexts::from_language(language);

    // Load enabled tools directly from tools.json (DB may be empty at startup)
    let tools: Vec<ToolForTray> = load_tools_from_json()
        .into_iter()
        .filter(|t| t.enabled.unwrap_or(true))
        .collect();

    let mut builder = MenuBuilder::new(app);
    builder = builder.item(&MenuItem::with_id(
        app,
        "show",
        texts.show_main,
        true,
        None::<&str>,
    )?);

    if tools.is_empty() {
        builder = builder.item(&no_tools_menu_item(app)?);
    } else {
        for tool in &tools {
            builder = builder.item(&tool_menu_item(app, tool)?);
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
                if tool_needs_param_dialog(tool_id) {
                    // Show and focus the main window first, then open the param dialog.
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let payload = serde_json::json!({ "toolId": tool_id });
                    let _ = app.emit("open_param_dialog", payload);
                } else {
                    log::debug!(
                        "Ignoring direct tray execution in generic handler for tool {tool_id}"
                    );
                }
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

    fn make_tool(id: &str, name: &str, tool_type: &str) -> ToolForTray {
        ToolForTray {
            id: id.to_string(),
            name: name.to_string(),
            tool_type: tool_type.to_string(),
            params: vec![],
            enabled: Some(true),
        }
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "muda MenuChild requires main thread on macOS"
    )]
    fn test_tool_menu_item() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();
        let tool = make_tool("shell-echo", "Echo", "shell");

        let result = tool_menu_item(&app_handle, &tool);
        assert!(
            result.is_ok(),
            "should build tool menu item: {:?}",
            result.err()
        );
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "muda MenuChild requires main thread on macOS"
    )]
    fn test_no_tools_menu_item() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();

        let result = no_tools_menu_item(&app_handle);
        assert!(
            result.is_ok(),
            "should build disabled placeholder: {:?}",
            result.err()
        );
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
    fn test_tool_menu_item_accepts_any_tool_type() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();
        let tool = make_tool("custom-tool", "Custom", "custom");

        let result = tool_menu_item(&app_handle, &tool);
        assert!(
            result.is_ok(),
            "flat tray menu should accept any enabled tool type: {:?}",
            result.err()
        );
    }
}
