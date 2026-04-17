use crate::database::{ExecutionLog, LogDao, LogQuery};
use crate::AppState;

#[tauri::command]
pub async fn get_logs(
    state: tauri::State<'_, AppState>,
    tool_id: Option<String>,
    status: Option<String>,
    from: Option<String>,
    to: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<ExecutionLog>, i64), String> {
    let query = LogQuery {
        tool_id,
        status,
        from,
        to,
        page: Some(page),
        page_size: Some(page_size),
    };

    state
        .db
        .with_logs_dao(|conn| LogDao::list(conn, &query).map_err(Into::into))
        .map_err(Into::into)
}
