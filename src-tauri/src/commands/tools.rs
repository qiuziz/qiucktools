use std::collections::HashMap;

use crate::database::{ExecutionLog, LogDao, Tool, ToolDao};
use crate::services::{ExecutionResult, Executor, Notifier};
use crate::AppState;

const TOOLS_JSON_PATH: &str = "~/work/quicktools/tools.json";

#[tauri::command]
pub async fn load_tools(state: tauri::State<'_, AppState>) -> Result<Vec<Tool>, String> {
    let path = crate::services::executor::expand_home(TOOLS_JSON_PATH);

    state
        .db
        .with_tools_dao(|conn| {
            ToolDao::load_from_file(conn, &path)
                .map_err(|err: Box<dyn std::error::Error>| crate::AppError::Message(err.to_string()))?;
            ToolDao::list(conn).map_err(Into::into)
        })
        .map_err(Into::into)
}

#[tauri::command]
pub async fn execute_tool(
    state: tauri::State<'_, AppState>,
    tool_id: String,
    params: HashMap<String, String>,
    app: tauri::AppHandle,
) -> Result<ExecutionResult, String> {
    let tool = state
        .db
        .with_tools_dao(|conn| {
            ToolDao::get(conn, &tool_id)
                .map_err(crate::AppError::from)?
                .ok_or_else(|| crate::AppError::Message(format!("Tool not found: {tool_id}")))
        })
        .map_err(|err| err.to_string())?;

    let result = Executor::execute(&tool, &params, &app)
        .await
        .map_err(|err| err.to_string())?;

    let params_json = serde_json::to_string(&result.params).map_err(|err| err.to_string())?;
    let log = ExecutionLog {
        id: result.id.clone(),
        tool_id: result.tool_id.clone(),
        tool_name: result.tool_name.clone(),
        params: params_json,
        status: result.status.clone(),
        duration_ms: result.duration,
        exit_code: result.exit_code.map(i64::from),
        stdout: result.stdout.clone(),
        stderr: result.stderr.clone(),
        error: result.error.clone(),
        executed_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .db
        .with_logs_dao(|conn| LogDao::insert(conn, &log).map_err(Into::into))
        .map_err(|err| err.to_string())?;

    Notifier::notify_execution(&app, &result.tool_name, &result.status, result.duration)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, LogQuery};
    use tempfile::NamedTempFile;

    fn test_tool_json() -> String {
        serde_json::json!([
            {
                "id": "notify-test",
                "name": "Notify Test",
                "icon": "bell",
                "description": "test",
                "type": "notification",
                "command": "notify",
                "workingDir": "~",
                "timeoutMs": 1000,
                "params": [],
                "sortOrder": 0,
                "enabled": true
            }
        ])
        .to_string()
    }

    #[test]
    fn load_tools_from_temp_file_and_log_execution() {
        let db = Database::memory().expect("memory db should initialize");
        let temp = NamedTempFile::new().expect("temp tools file should be created");
        std::fs::write(temp.path(), test_tool_json()).expect("tools json should be written");

        let loaded: Vec<Tool> = db
            .with_tools_dao(|conn| {
                ToolDao::load_from_file(conn, temp.path().to_str().expect("utf-8 path"))
                    .map_err(|err: Box<dyn std::error::Error>| crate::AppError::Message(err.to_string()))?;
                ToolDao::list(conn).map_err(Into::into)
            })
            .expect("tools should load");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "notify-test");

        let result = ExecutionResult {
            id: uuid::Uuid::new_v4().to_string(),
            tool_id: loaded[0].id.clone(),
            tool_name: loaded[0].name.clone(),
            status: "success".to_string(),
            duration: 5,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            error: None,
            params: HashMap::new(),
        };

        let log = ExecutionLog {
            id: result.id.clone(),
            tool_id: result.tool_id.clone(),
            tool_name: result.tool_name.clone(),
            params: serde_json::to_string(&result.params).expect("params should serialize"),
            status: result.status.clone(),
            duration_ms: result.duration,
            exit_code: result.exit_code.map(i64::from),
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
            error: result.error.clone(),
            executed_at: chrono::Utc::now().to_rfc3339(),
        };

        db.with_logs_dao(|conn| LogDao::insert(conn, &log).map_err(Into::into))
            .expect("log should insert");

        let (logs, total): (Vec<ExecutionLog>, i64) = db
            .with_logs_dao(|conn| LogDao::list(conn, &LogQuery::default()).map_err(Into::into))
            .expect("logs should list");

        assert_eq!(total, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].tool_id, "notify-test");
        assert_eq!(logs[0].status, "success");
    }
}
