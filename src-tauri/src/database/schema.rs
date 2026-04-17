//! Schema 定义和迁移
//!
//! 负责数据库表结构的创建和版本迁移。

use crate::database::lock_conn;
use crate::database::Database;
use crate::error::AppError;
use rusqlite::Connection;

impl Database {
    /// 创建所有数据库表
    pub(crate) fn create_tables(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        Self::create_tables_on_conn(&conn)
    }

    /// 在指定连接上创建表（供迁移和测试使用）
    pub(crate) fn create_tables_on_conn(conn: &Connection) -> Result<(), AppError> {
        // Settings 表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Log config 表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS log_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled BOOLEAN NOT NULL DEFAULT 1,
                level TEXT NOT NULL DEFAULT 'info'
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 初始化默认 log_config
        conn.execute(
            "INSERT OR IGNORE INTO log_config (id, enabled, level) VALUES (1, 1, 'info')",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}