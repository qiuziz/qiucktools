use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLog {
    pub id: String,
    #[serde(rename = "toolId")]
    pub tool_id: String,
    #[serde(rename = "toolName")]
    pub tool_name: String,
    pub params: String,
    pub status: String,
    #[serde(rename = "durationMs")]
    pub duration_ms: i64,
    #[serde(rename = "exitCode")]
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
    #[serde(rename = "executedAt")]
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    #[serde(rename = "toolId")]
    pub tool_id: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<i64>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
}

/// Log DAO - operates on a &Connection reference.
/// Caller holds the lock for the duration of the call.
pub struct LogDao;

impl LogDao {
    pub fn insert(conn: &Connection, log: &ExecutionLog) -> SqlResult<()> {
        conn.execute(
            r#"INSERT INTO execution_logs (id, tool_id, tool_name, params, status, duration_ms, exit_code, stdout, stderr, error)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                log.id,
                log.tool_id,
                log.tool_name,
                log.params,
                log.status,
                log.duration_ms,
                log.exit_code,
                log.stdout,
                log.stderr,
                log.error,
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection, query: &LogQuery) -> SqlResult<(Vec<ExecutionLog>, i64)> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut conditions = Vec::new();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref tool_id) = query.tool_id {
            conditions.push("tool_id = ?");
            args.push(Box::new(tool_id.clone()));
        }
        if let Some(ref status) = query.status {
            conditions.push("status = ?");
            args.push(Box::new(status.clone()));
        }
        if let Some(ref from) = query.from {
            conditions.push("executed_at >= ?");
            args.push(Box::new(from.clone()));
        }
        if let Some(ref to) = query.to {
            conditions.push("executed_at <= ?");
            args.push(Box::new(to.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM execution_logs {}", where_clause);
        let total: i64 = {
            let mut stmt = conn.prepare(&count_sql)?;
            let args_refs: Vec<&dyn rusqlite::ToSql> =
                args.iter().map(|b| b.as_ref()).collect();
            stmt.query_row(args_refs.as_slice(), |r| r.get(0))?
        };

        // Fetch page
        let select_sql = format!(
            "SELECT id, tool_id, tool_name, params, status, duration_ms, exit_code, stdout, stderr, error, executed_at FROM execution_logs {} ORDER BY executed_at DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut stmt = conn.prepare(&select_sql)?;
        let mut args_refs: Vec<&dyn rusqlite::ToSql> =
            args.iter().map(|b| b.as_ref()).collect();
        args_refs.push(&page_size);
        args_refs.push(&offset);

        let logs = stmt.query_map(args_refs.as_slice(), |row| {
            Ok(ExecutionLog {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                tool_name: row.get(2)?,
                params: row.get(3)?,
                status: row.get(4)?,
                duration_ms: row.get(5)?,
                exit_code: row.get(6)?,
                stdout: row.get(7)?,
                stderr: row.get(8)?,
                error: row.get(9)?,
                executed_at: row.get(10)?,
            })
        })?;

        let result: SqlResult<Vec<ExecutionLog>> = logs.collect();
        Ok((result?, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_log(id: &str, tool_id: &str, status: &str, duration: i64) -> ExecutionLog {
        ExecutionLog {
            id: id.to_string(),
            tool_id: tool_id.to_string(),
            tool_name: "Test Tool".to_string(),
            params: "{}".to_string(),
            status: status.to_string(),
            duration_ms: duration,
            exit_code: Some(0),
            stdout: "output".to_string(),
            stderr: "".to_string(),
            error: None,
            executed_at: "2026-04-17T12:00:00".to_string(),
        }
    }

    #[test]
    fn test_log_dao_insert_and_paginate() {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migration::run_migrations(&conn).unwrap();

        LogDao::insert(&conn, &make_log("log-1", "git-pull", "success", 100)).unwrap();
        LogDao::insert(&conn, &make_log("log-2", "git-pull", "failed", 200)).unwrap();
        LogDao::insert(&conn, &make_log("log-3", "git-status", "success", 50)).unwrap();
        LogDao::insert(&conn, &make_log("log-4", "git-pull", "timeout", 60000)).unwrap();
        LogDao::insert(&conn, &make_log("log-5", "git-status", "success", 30)).unwrap();

        // List all
        let (logs, total) = LogDao::list(&conn, &LogQuery::default()).unwrap();
        assert_eq!(total, 5);
        assert_eq!(logs.len(), 5);

        // Filter by tool_id
        let (logs, total) = LogDao::list(
            &conn,
            &LogQuery {
                tool_id: Some("git-pull".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(total, 3);
        assert_eq!(logs.len(), 3);

        // Filter by status
        let (logs, total) = LogDao::list(
            &conn,
            &LogQuery {
                status: Some("success".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(total, 3);

        // Pagination
        let (logs, total) = LogDao::list(
            &conn,
            &LogQuery {
                page: Some(1),
                page_size: Some(2),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(total, 5);
        assert_eq!(logs.len(), 2);

        // Page 2
        let (logs, total) = LogDao::list(
            &conn,
            &LogQuery {
                page: Some(2),
                page_size: Some(2),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(total, 5);
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_log_dao_ordering() {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migration::run_migrations(&conn).unwrap();

        LogDao::insert(&conn, &make_log("a-1", "tool-a", "success", 100)).unwrap();
        LogDao::insert(&conn, &make_log("b-1", "tool-b", "success", 100)).unwrap();
        LogDao::insert(&conn, &make_log("c-1", "tool-c", "success", 100)).unwrap();

        let (logs, _) = LogDao::list(&conn, &LogQuery::default()).unwrap();
        // Should be ordered by executed_at DESC (most recent first)
        assert_eq!(logs[0].id, "c-1");
        assert_eq!(logs[1].id, "b-1");
        assert_eq!(logs[2].id, "a-1");
    }
}