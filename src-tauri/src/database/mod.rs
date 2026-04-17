//! 数据库模块 - SQLite 数据持久化
//!
//! 此模块提供应用的核心数据存储功能，包括：
//! - Tools 管理
//! - 执行日志
//! - 通用设置

mod dao;
mod migration;
mod schema;

pub use dao::{ExecutionLog, LogDao, LogQuery, Tool, ToolDao, ToolParam, ToolParamOption};

use crate::config::get_app_config_dir;
use crate::error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// 数据库连接封装
///
/// 使用 Mutex 包装 Connection 以支持在多线程环境（如 Tauri State）中共享。
/// rusqlite::Connection 本身不是 Sync 的，因此需要这层包装。
pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    /// 初始化数据库连接并运行迁移
    ///
    /// 数据库文件位于应用配置目录下的 `quicktools.db`
    pub fn init() -> Result<Self, AppError> {
        let db_path = Self::get_db_path();

        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
        }

        let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e.to_string()))?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;

        Ok(db)
    }

    /// 获取数据库文件路径
    fn get_db_path() -> PathBuf {
        get_app_config_dir().join("quicktools.db")
    }

    /// 运行数据库迁移
    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        migration::run_migrations(&conn).map_err(|e| AppError::Database(e.to_string()))
    }

    /// 创建内存数据库（用于测试）
    #[cfg(test)]
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory().map_err(|e| AppError::Database(e.to_string()))?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;

        Ok(db)
    }

    /// 使用 Tools DAO 访问数据库
    pub fn with_tools_dao<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Database(e.to_string()))?;
        f(&conn)
    }

    /// 使用 Logs DAO 访问数据库
    pub fn with_logs_dao<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Database(e.to_string()))?;
        f(&conn)
    }

    /// Get a setting value by key
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?",
            [key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            [key, value],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get log configuration
    pub fn get_log_config(&self) -> Result<LogConfig, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.query_row(
            "SELECT enabled, level FROM log_config WHERE id = 1",
            [],
            |row| {
                let enabled: bool = row.get(0)?;
                let level: String = row.get(1)?;
                Ok(LogConfig { enabled, level })
            },
        )
        .map_err(|e| AppError::Database(e.to_string()))
    }
}

/// Log configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub enabled: bool,
    pub level: String,
}

impl LogConfig {
    pub fn to_level_filter(&self) -> log::LevelFilter {
        if !self.enabled {
            return log::LevelFilter::Off;
        }
        match self.level.as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        }
    }
}