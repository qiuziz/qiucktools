use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 获取用户主目录，带回退
pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// 获取应用配置目录（macOS: ~/Library/Application Support/QuickTools）
pub fn get_app_config_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("QuickTools"))
        .unwrap_or_else(get_home_dir)
}

/// 获取 Claude Code 配置目录路径
pub fn get_claude_config_dir() -> PathBuf {
    get_home_dir().join(".claude")
}

/// 默认 Claude MCP 配置文件路径 (~/.claude.json)
pub fn get_claude_mcp_path() -> PathBuf {
    get_home_dir().join(".claude.json")
}

/// 默认 Claude 设置文件路径 (~/.claude/settings.json)
pub fn get_claude_settings_path() -> PathBuf {
    get_claude_config_dir().join("settings.json")
}

/// 读取 JSON 文件
pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::io(path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| AppError::json(path, e))
}