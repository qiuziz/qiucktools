//! 鸿蒙新版本集成分支工具
//!
//! 流程：输入新版本分支名 → 确认当前分支 → 创建 MR 并 merge → 从 master 创建新分支

use std::process::Stdio;
use tauri::{AppHandle, State};
use tokio::process::Command;
use serde::{Deserialize, Serialize};

const PROJECT_ID: &str = "87817"; // _fe/harmonyajkproject

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchPreview {
    pub new_branch: String,
    pub current_branch: String,
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationResult {
    pub success: bool,
    pub mr_url: Option<String>,
    pub new_branch: String,
    pub message: String,
}

/// 从新分支名推导当前分支
/// 例如: release-17.37 -> release-17.36
fn derive_current_branch(new_branch: &str) -> Option<String> {
    let prefix = "release-";
    if !new_branch.starts_with(prefix) {
        return None;
    }
    let version = &new_branch[prefix.len()..];
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let major: u32 = parts.get(0)?.parse().ok()?;
    let minor: u32 = parts.get(1)?.parse().ok()?;
    let patch: Option<u32> = parts.get(2).and_then(|s| s.parse().ok());

    if minor == 0 && patch.is_none() {
        // e.g. release-1.0 -> release-0.xx 不合理，跳过
        return None;
    }

    if minor > 0 {
        let new_minor = minor - 1;
        if let Some(p) = patch {
            Some(format!("{prefix}{major}.{new_minor}.{p}"))
        } else {
            Some(format!("{prefix}{major}.{new_minor}"))
        }
    } else if let Some(p) = patch {
        // minor == 0, e.g. 1.0.5 -> 0.xx
        if p == 0 {
            return None;
        }
        Some(format!("{prefix}{major}.0.{}", p - 1))
    } else {
        None
    }
}

/// 预览：检查分支并返回推导的当前分支
#[tauri::command]
pub async fn harmony_branch_preview(
    new_branch: String,
) -> Result<BranchPreview, String> {
    // 校验格式
    if !new_branch.starts_with("release-") {
        return Err("分支名必须以 release- 开头".to_string());
    }
    let version = &new_branch["release-".len()..];
    if version.is_empty() || !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err("版本号格式错误，应为 release-x.xx 或 release-x.xx.xx".to_string());
    }

    let current_branch = derive_current_branch(&new_branch)
        .ok_or_else(|| "无法从版本号推导当前分支".to_string())?;

    // 检查当前分支是否存在
    let exists = check_branch_exists(&current_branch).await?;

    Ok(BranchPreview {
        new_branch,
        current_branch,
        exists,
    })
}

/// 执行集成分支流程
#[tauri::command]
pub async fn harmony_branch_integrate(
    app: AppHandle,
    state: State<'_, crate::AppState>,
    new_branch: String,
    current_branch: String,
) -> Result<IntegrationResult, String> {
    log::info!(
        "开始鸿蒙集成分支: new={}, current={}",
        new_branch,
        current_branch
    );

    // 1. 创建 MR (当前分支 -> master)
    let mr_iid = create_mr(&current_branch, "master").await?;

    // 2. 合并 MR
    merge_mr(mr_iid).await?;

    // 3. 从 master 创建新分支
    create_branch(&new_branch, "master").await?;

    // 记录日志
    let result = IntegrationResult {
        success: true,
        mr_url: Some(format!("https://igit.58corp.com/_fe/harmonyajkproject/-/merge_requests/{}", mr_iid)),
        new_branch: new_branch.clone(),
        message: format!(
            "已成功: MR #{} 已合并，新分支 {} 已创建",
            mr_iid, new_branch
        ),
    };

    // 发系统通知
    let _ = crate::services::Notifier::notify_execution(
        &app,
        "鸿蒙新版本集成分支",
        "success",
        0,
    );

    // 写数据库日志
    let params_json = serde_json::to_string(&serde_json::json!({
        "newBranch": new_branch,
        "currentBranch": current_branch,
    })).map_err(|e| e.to_string())?;

    let log = crate::database::ExecutionLog {
        id: uuid::Uuid::new_v4().to_string(),
        tool_id: "harmony-branch-integration".to_string(),
        tool_name: "鸿蒙新版本集成分支".to_string(),
        params: params_json,
        status: "success".to_string(),
        duration_ms: 0,
        exit_code: Some(0),
        stdout: serde_json::to_string(&result).map_err(|e| e.to_string())?,
        stderr: String::new(),
        error: None,
        executed_at: chrono::Utc::now().to_rfc3339(),
    };

    let _ = state.db.with_logs_dao(|conn| {
        crate::database::LogDao::insert(conn, &log).map_err(Into::into)
    });

    log::info!("鸿蒙集成分支完成: {}", result.message);
    Ok(result)
}

/// 检查分支是否存在
async fn check_branch_exists(branch: &str) -> Result<bool, String> {
    let output = Command::new("glab")
        .args(["api", &format!("projects/{PROJECT_ID}/repository/branches/{}", branch)])
        .env("GITLAB_HOST", "igit.58corp.com")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("glab 执行失败: {e}"))?;

    Ok(output.status.success())
}

/// 创建 MR
async fn create_mr(source_branch: &str, target_branch: &str) -> Result<u32, String> {
    let output = Command::new("glab")
        .args([
            "api",
            "--method", "POST",
            "projects/87817/merge_requests",
            "--field", &format!("source_branch={}", source_branch),
            "--field", &format!("target_branch={}", target_branch),
            "--field", "title=Auto merge from integration",
            "--field", "squash=false",
        ])
        .env("GITLAB_HOST", "igit.58corp.com")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("glab mr create 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("创建 MR 失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 解析 iid
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析 MR 响应失败: {e}"))?;
    let iid = parsed.get("iid")
        .and_then(|v| v.as_u64())
        .ok_or("响应中未找到 iid")? as u32;

    log::info!("创建 MR #{}", iid);
    Ok(iid)
}

/// 合并 MR
async fn merge_mr(mr_iid: u32) -> Result<(), String> {
    let output = Command::new("glab")
        .args([
            "api",
            "--method", "PUT",
            &format!("projects/87817/merge_requests/{}/merge", mr_iid),
        ])
        .env("GITLAB_HOST", "igit.58corp.com")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("glab mr merge 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("合并 MR 失败: {}", stderr));
    }

    log::info!("MR #{} 已合并", mr_iid);
    Ok(())
}

/// 从 master 创建新分支
async fn create_branch(branch: &str, base: &str) -> Result<(), String> {
    let output = Command::new("glab")
        .args([
            "api",
            "--method", "POST",
            "projects/87817/repository/branches",
            "--field", &format!("branch={}", branch),
            "--field", &format!("ref={}", base),
        ])
        .env("GITLAB_HOST", "igit.58corp.com")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("glab branch create 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 如果分支已存在，也算成功
        if stderr.contains("already exists") {
            log::warn!("分支 {} 已存在，跳过创建", branch);
            return Ok(());
        }
        return Err(format!("创建分支失败: {}", stderr));
    }

    log::info!("分支 {} 已从 {} 创建", branch, base);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_current_branch() {
        assert_eq!(derive_current_branch("release-17.37"), Some("release-17.36".to_string()));
        assert_eq!(derive_current_branch("release-17.1"), Some("release-17.0".to_string()));
        assert_eq!(derive_current_branch("release-1.0.5"), Some("release-1.0.4".to_string()));
        assert_eq!(derive_current_branch("release-17.0"), None); // 不合理
        assert_eq!(derive_current_branch("release-1.0.0"), None); // 不合理
        assert_eq!(derive_current_branch("main"), None);
        assert_eq!(derive_current_branch("release-"), None);
        assert_eq!(derive_current_branch("master"), None);
    }
}