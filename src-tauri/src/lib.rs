mod app_config;
mod app_store;
mod auto_launch;
mod commands;
mod config;
mod database;
mod error;
mod init_status;
mod panic_hook;
#[cfg(target_os = "linux")]
mod linux_fix;
mod services;
mod settings;
mod store;

mod tray;

pub use app_config::{AppType, InstalledSkill, McpApps, McpServer, MultiAppConfig, SkillApps};
pub use commands::*;
pub use config::{get_claude_mcp_path, get_claude_settings_path, read_json_file};
pub use database::Database;
pub use error::AppError;
pub use settings::{update_settings, AppSettings};
pub use store::AppState;

use std::sync::Arc;
#[cfg(target_os = "macos")]
use tauri::image::Image;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::RunEvent;
use tauri::{Emitter, Manager};

/// 更新托盘菜单的Tauri命令
#[tauri::command]
async fn update_tray_menu(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    match tray::create_tray_menu(&app, state.inner()) {
        Ok(new_menu) => {
            if let Some(tray) = app.tray_by_id("main") {
                tray.set_menu(Some(new_menu))
                    .map_err(|e| format!("更新托盘菜单失败: {e}"))?;
                return Ok(true);
            }
            Ok(false)
        }
        Err(err) => {
            log::error!("创建托盘菜单失败: {err}");
            Ok(false)
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_tray_icon() -> Option<Image<'static>> {
    const ICON_BYTES: &[u8] = include_bytes!("../icons/tray/macos/statusbar_template_3x.png");

    match Image::from_bytes(ICON_BYTES) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("Failed to load macOS tray icon: {err}");
            None
        }
    }
}

/// 获取 macOS 标准应用数据目录
fn get_mac_data_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("QuickTools"))
}

/// 从旧配置目录迁移到新目录
fn migrate_old_config() {
    let old_dir = dirs::home_dir().map(|h| h.join(".cc-switch"));
    let new_dir = get_mac_data_dir();

    let (Some(old), Some(new)) = (old_dir, new_dir) else {
        return;
    };

    if !old.exists() {
        return;
    }
    if new.exists() {
        return;
    }

    eprintln!(
        "[QuickTools] Migrating config from {} to {}",
        old.display(),
        new.display()
    );

    // Create new directory
    if let Err(e) = std::fs::create_dir_all(&new) {
        eprintln!("[QuickTools] Failed to create new config dir: {e}");
        return;
    }

    // Migrate each file
    let files = ["quicktools.db", "settings.json"];
    for file in files {
        let src = old.join(file);
        let dst = new.join(file);
        if src.exists() {
            if let Err(e) = std::fs::copy(&src, &dst) {
                eprintln!("[QuickTools] Failed to copy {file}: {e}");
            } else {
                eprintln!("[QuickTools] Migrated {file}");
            }
        }
    }

    // Migrate logs directory
    let logs_src = old.join("logs");
    let logs_dst = new.join("logs");
    if logs_src.exists() && logs_src.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&logs_src) {
            if let Err(e) = std::fs::create_dir_all(&logs_dst) {
                eprintln!("[QuickTools] Failed to create logs dir: {e}");
            } else {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let src_file = logs_src.join(&file_name);
                    let dst_file = logs_dst.join(&file_name);
                    let _ = std::fs::copy(&src_file, &dst_file);
                }
                eprintln!("[QuickTools] Migrated logs/");
            }
        }
    }

    // Archive old crash log
    let crash_log = old.join("crash.log");
    let crash_log_bak = old.join("crash.log.bak");
    if crash_log.exists() {
        let _ = std::fs::rename(&crash_log, &crash_log_bak);
        eprintln!("[QuickTools] Archived crash.log -> crash.log.bak");
    }

    eprintln!("[QuickTools] Migration complete");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 迁移旧配置目录（如果存在）
    migrate_old_config();

    // 设置 panic hook，在应用崩溃时记录日志到 <app_config_dir>/crash.log
    panic_hook::setup_panic_hook();

    let mut builder = tauri::Builder::default();

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("=== Single Instance Callback Triggered ===");

            // Show and focus window regardless
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
                #[cfg(target_os = "linux")]
                {
                    linux_fix::nudge_main_window(window.clone());
                }
            }
        }));
    }

    let builder = builder
        // 拦截窗口关闭：根据设置决定是否最小化到托盘
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = crate::settings::get_settings();

                if settings.minimize_to_tray_on_close {
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    {
                        let _ = window.set_skip_taskbar(true);
                    }
                    #[cfg(target_os = "macos")]
                    {
                        tray::apply_tray_policy(window.app_handle(), false);
                    }
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // 预先刷新 Store 覆盖配置，确保后续路径读取正确（日志/数据库等）
            app_store::refresh_app_config_dir_override(app.handle());
            panic_hook::init_app_config_dir(crate::config::get_app_config_dir());

            // 初始化日志（单文件输出到 <app_config_dir>/logs/quicktools.log）
            {
                use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

                let log_dir = panic_hook::get_log_dir();

                // 确保日志目录存在
                if let Err(e) = std::fs::create_dir_all(&log_dir) {
                    eprintln!("创建日志目录失败: {e}");
                }

                // 启动时删除旧日志文件，实现单文件覆盖效果
                let log_file_path = log_dir.join("quicktools.log");
                let _ = std::fs::remove_file(&log_file_path);

                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        // 初始化为 Trace，允许后续通过 log::set_max_level() 动态调整级别
                        .level(log::LevelFilter::Trace)
                        .targets([
                            Target::new(TargetKind::Stdout),
                            Target::new(TargetKind::Folder {
                                path: log_dir,
                                file_name: Some("quicktools".into()),
                            }),
                        ])
                        // 单文件模式：启动时删除旧文件，达到大小时轮转
                        // 注意：KeepSome(n) 内部会做 n-2 运算，n=1 会导致 usize 下溢
                        // KeepSome(2) 是最小安全值，表示不保留轮转文件
                        .rotation_strategy(RotationStrategy::KeepSome(2))
                        // 单文件大小限制 1GB
                        .max_file_size(1024 * 1024 * 1024)
                        .timezone_strategy(TimezoneStrategy::UseLocal)
                        .build(),
                )?;
            }

            // 初始化数据库
            let app_config_dir = crate::config::get_app_config_dir();
            let db_path = app_config_dir.join("quicktools.db");

            let db = loop {
                match crate::database::Database::init() {
                    Ok(db) => break Arc::new(db),
                    Err(e) => {
                        log::error!("Failed to init database: {e}");
                        // 直接退出，不再弹出对话框
                        std::process::exit(1);
                    }
                }
            };

            let app_state = AppState::new(db);

            // 创建动态托盘菜单
            let menu = tray::create_tray_menu(app.handle(), &app_state)?;

            // 构建托盘
            let mut tray_builder = TrayIconBuilder::with_id("main")
                .on_tray_icon_event(|_tray, event| match event {
                    // 左键点击已通过 show_menu_on_left_click(true) 打开菜单，这里不再额外处理
                    TrayIconEvent::Click { .. } => {}
                    _ => log::debug!("unhandled event {event:?}"),
                })
                .menu(&menu)
                .on_menu_event(|app, event| {
                    tray::handle_tray_menu_event(app, &event.id.0);
                })
                .show_menu_on_left_click(true);

            // 使用平台对应的托盘图标（macOS 使用模板图标适配深浅色）
            #[cfg(target_os = "macos")]
            {
                if let Some(icon) = macos_tray_icon() {
                    tray_builder = tray_builder.icon(icon);
                } else if let Some(icon) = app.default_window_icon() {
                    log::warn!("Falling back to default window icon for tray");
                    tray_builder = tray_builder.icon(icon.clone());
                } else {
                    log::warn!("Failed to load macOS tray icon for tray");
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                if let Some(icon) = app.default_window_icon() {
                    tray_builder = tray_builder.icon(icon.clone());
                } else {
                    log::warn!("Failed to get default window icon for tray");
                }
            }

            let _tray = tray_builder.build(app)?;

            // 将同一个实例注入到全局状态，避免重复创建导致的不一致
            app.manage(app_state);

            // 从数据库加载日志配置并应用
            {
                let db = &app.state::<AppState>().db;
                if let Ok(log_config) = db.get_log_config() {
                    log::set_max_level(log_config.to_level_filter());
                    log::info!(
                        "已加载日志配置: enabled={}, level={}",
                        log_config.enabled,
                        log_config.level
                    );
                }
            }

            // 迁移旧的 app_config_dir 配置到 Store
            if let Err(e) = app_store::migrate_app_config_dir_from_settings(app.handle()) {
                log::warn!("迁移 app_config_dir 失败: {e}");
            }

            // Linux: 禁用 WebKitGTK 硬件加速，防止 EGL 初始化失败导致白屏
            #[cfg(target_os = "linux")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.with_webview(|webview| {
                        use webkit2gtk::{WebViewExt, SettingsExt, HardwareAccelerationPolicy};
                        let wk_webview = webview.inner();
                        if let Some(settings) = WebViewExt::settings(&wk_webview) {
                            SettingsExt::set_hardware_acceleration_policy(&settings, HardwareAccelerationPolicy::Never);
                            log::info!("已禁用 WebKitGTK 硬件加速");
                        }
                    });
                }
            }

            // 静默启动：根据设置决定是否显示主窗口
            let settings = crate::settings::get_settings();
            if let Some(window) = app.get_webview_window("main") {
                // 在窗口首次显示前同步装饰状态，避免前端加载后再切换导致标题栏闪烁
                // 仅 Linux 生效：解决 Wayland 下系统窗口按钮不可用的问题
                #[cfg(target_os = "linux")]
                let _ = window.set_decorations(!settings.use_app_window_controls);
                if settings.silent_startup {
                    // 静默启动模式：保持窗口隐藏
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    let _ = window.set_skip_taskbar(true);
                    #[cfg(target_os = "macos")]
                    tray::apply_tray_policy(app.handle(), false);
                    log::info!("静默启动模式：主窗口已隐藏");
                } else {
                    // 正常启动模式：显示窗口
                    let _ = window.show();
                    log::info!("正常启动模式：主窗口已显示");

                    // Linux: 解决首次启动 UI 无响应问题（Tauri #10746 + wry #637）。
                    // 启动时 webview 未获取焦点 + surface 尺寸协商失败，导致点击无效。
                    // 这里做 set_focus + 伪 resize，等价于无视觉版本的"最大化-还原"。
                    #[cfg(target_os = "linux")]
                    {
                        linux_fix::nudge_main_window(window.clone());
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::set_auto_launch,
            commands::get_auto_launch_status,
            commands::set_window_theme,
            commands::load_tools,
            commands::execute_tool,
            commands::get_logs,
            update_tray_menu,
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| {
        // 处理退出请求（所有平台）
        if let RunEvent::ExitRequested { api, code, .. } = &event {
            // code 为 None 表示运行时自动触发（如隐藏窗口的 WebView 被回收导致无存活窗口），
            // 此时应仅阻止退出、保持托盘后台运行；
            // code 为 Some(_) 表示用户主动调用 app.exit() 退出（如托盘菜单"退出"），
            // 此时执行清理后退出。
            if code.is_none() {
                log::info!("运行时触发退出请求（无存活窗口），阻止退出以保持托盘后台运行");
                api.prevent_exit();
                return;
            }

            log::info!("收到用户主动退出请求 (code={code:?})，开始清理...");
            api.prevent_exit();

            let app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                log::info!("清理完成，退出应用");

                // 使用 std::process::exit 避免再次触发 ExitRequested
                std::process::exit(0);
            });
            return;
        }

        #[cfg(target_os = "macos")]
        {
            match event {
                // macOS 在 Dock 图标被点击并重新激活应用时会触发 Reopen 事件，这里手动恢复主窗口
                RunEvent::Reopen { .. } => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = window.set_skip_taskbar(false);
                        }
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        tray::apply_tray_policy(app_handle, true);
                    }
                }
                _ => {}
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app_handle, event);
        }
    });
}