use tauri::AppHandle;

pub struct Notifier;

impl Notifier {
    pub fn notify_execution(
        app: &AppHandle,
        tool_name: &str,
        status: &str,
        duration_ms: i64,
    ) -> Result<(), String> {
        let body = match status {
            "success" => format!("{tool_name} 执行成功（{duration_ms}ms）"),
            "timeout" => format!("{tool_name} 执行超时"),
            _ => format!("{tool_name} 执行失败"),
        };

        tauri_plugin_notification::NotificationExt::notification(app)
            .builder()
            .title("QuickTools")
            .body(body)
            .show()
            .map_err(|err| err.to_string())
    }
}
