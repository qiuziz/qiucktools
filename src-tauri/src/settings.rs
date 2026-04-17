use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use crate::error::AppError;

/// 主页面显示的应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibleApps {
    #[serde(default = "default_true")]
    pub claude: bool,
    #[serde(default = "default_true")]
    pub codex: bool,
    #[serde(default = "default_true")]
    pub gemini: bool,
    #[serde(default = "default_true")]
    pub opencode: bool,
    #[serde(default = "default_true")]
    pub openclaw: bool,
}

impl Default for VisibleApps {
    fn default() -> Self {
        Self {
            claude: true,
            codex: true,
            gemini: true,
            opencode: true,
            openclaw: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_show_in_tray() -> bool {
    true
}

fn default_minimize_to_tray_on_close() -> bool {
    true
}

/// 应用设置结构
///
/// 存储设备级别设置，保存在本地 `~/.cc-switch/settings.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // ===== 设备级 UI 设置 =====
    #[serde(default = "default_show_in_tray")]
    pub show_in_tray: bool,
    #[serde(default = "default_minimize_to_tray_on_close")]
    pub minimize_to_tray_on_close: bool,
    #[serde(default)]
    pub use_app_window_controls: bool,
    /// 是否开机自启
    #[serde(default)]
    pub launch_on_startup: bool,
    /// 静默启动（程序启动时不显示主窗口，仅托盘运行）
    #[serde(default)]
    pub silent_startup: bool,
    /// User has confirmed the first-run welcome notice
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_run_notice_confirmed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    // ===== 主页面显示的应用 =====
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_apps: Option<VisibleApps>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            show_in_tray: true,
            minimize_to_tray_on_close: true,
            use_app_window_controls: false,
            launch_on_startup: false,
            silent_startup: false,
            first_run_notice_confirmed: None,
            language: None,
            visible_apps: None,
        }
    }
}

static SETTINGS: OnceLock<RwLock<AppSettings>> = OnceLock::new();

fn get_settings_path() -> Option<PathBuf> {
    Some(
        get_home_dir()
            .join(".cc-switch")
            .join("settings.json"),
    )
}

fn normalize_paths(_settings: &mut AppSettings) {
    // No path fields in simplified version
}

/// Load settings from disk into global cache
fn load_settings() -> AppSettings {
    let path = match get_settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    if !path.exists() {
        return AppSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(mut settings) => {
                normalize_paths(&mut settings);
                settings
            }
            Err(e) => {
                log::warn!("Failed to parse settings.json: {e}");
                AppSettings::default()
            }
        },
        Err(e) => {
            log::warn!("Failed to read settings.json: {e}");
            AppSettings::default()
        }
    }
}

/// Get settings from global cache
pub fn get_settings() -> AppSettings {
    SETTINGS
        .get_or_init(|| RwLock::new(load_settings()))
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Update settings and save to disk
pub fn update_settings(new_settings: AppSettings) -> Result<(), AppError> {
    let mut settings = new_settings;
    normalize_paths(&mut settings);

    // Save to file
    let path = get_settings_path().ok_or_else(|| {
        AppError::Config("Cannot determine settings path".to_string())
    })?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| AppError::Config(format!("Failed to serialize settings: {e}")))?;

    let mut file = fs::File::create(&path)
        .map_err(|e| AppError::io(&path, e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| AppError::io(&path, e))?;

    // Update global cache
    if let Some(global) = SETTINGS.get() {
        if let Ok(mut guard) = global.write() {
            *guard = settings;
        }
    }

    Ok(())
}

/// Get home directory
fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}