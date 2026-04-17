use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use tauri::AppHandle;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::database::Tool;

const OUTPUT_LIMIT: usize = 2000;
const TRUNCATION_SUFFIX: &str = "...(truncated)";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub id: String,
    pub tool_id: String,
    pub tool_name: String,
    pub status: String,
    pub duration: i64,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
    pub params: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Tool execution failed: {0}")]
    Execution(String),
    #[error("Invalid tool type: {0}")]
    InvalidToolType(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Executor;

impl Executor {
    pub async fn execute(
        tool: &Tool,
        params: &HashMap<String, String>,
        _app_handle: &AppHandle,
    ) -> Result<ExecutionResult, ExecutorError> {
        let command = substitute_params(&tool.command, params);
        let started_at = Instant::now();

        match tool.tool_type.as_str() {
            "notification" => Ok(ExecutionResult {
                id: uuid::Uuid::new_v4().to_string(),
                tool_id: tool.id.clone(),
                tool_name: tool.name.clone(),
                status: "success".to_string(),
                duration: started_at.elapsed().as_millis() as i64,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                error: None,
                params: params.clone(),
            }),
            "shell" => {
                Self::execute_command(tool, params, "sh", vec!["-c".to_string(), command], started_at)
                    .await
            }
            "script" => {
                let interpreter = script_interpreter(&command)?;
                Self::execute_command(tool, params, &interpreter.0, interpreter.1, started_at).await
            }
            "open" => {
                #[cfg(target_os = "macos")]
                {
                    Self::execute_command(tool, params, "open", vec![command], started_at).await
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = started_at;
                    Err(ExecutorError::InvalidToolType(
                        "open is only supported on macOS".to_string(),
                    ))
                }
            }
            other => Err(ExecutorError::InvalidToolType(other.to_string())),
        }
    }

    async fn execute_command(
        tool: &Tool,
        params: &HashMap<String, String>,
        program: &str,
        args: Vec<String>,
        started_at: Instant,
    ) -> Result<ExecutionResult, ExecutorError> {
        let working_dir = normalize_working_dir(&tool.working_dir);
        let mut command = Command::new(program);
        command.args(args);
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        let timeout_duration = Duration::from_millis(tool.timeout_ms.max(1) as u64);
        let output = match timeout(timeout_duration, command.output()).await {
            Ok(result) => result?,
            Err(_) => {
                return Ok(ExecutionResult {
                    id: uuid::Uuid::new_v4().to_string(),
                    tool_id: tool.id.clone(),
                    tool_name: tool.name.clone(),
                    status: "timeout".to_string(),
                    duration: started_at.elapsed().as_millis() as i64,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    error: Some(format!("Execution timed out after {}ms", tool.timeout_ms)),
                    params: params.clone(),
                });
            }
        };

        let duration = started_at.elapsed().as_millis() as i64;
        let success = output.status.success();
        let status = if success { "success" } else { "failed" };
        let stdout = truncate_output(String::from_utf8_lossy(&output.stdout).into_owned());
        let stderr = truncate_output(String::from_utf8_lossy(&output.stderr).into_owned());
        let exit_code = output.status.code();
        let error = if success {
            None
        } else {
            Some(match exit_code {
                Some(code) => format!("Process exited with code {code}"),
                None => "Process terminated by signal".to_string(),
            })
        };

        Ok(ExecutionResult {
            id: uuid::Uuid::new_v4().to_string(),
            tool_id: tool.id.clone(),
            tool_name: tool.name.clone(),
            status: status.to_string(),
            duration,
            exit_code,
            stdout,
            stderr,
            error,
            params: params.clone(),
        })
    }
}

fn script_interpreter(command: &str) -> Result<(String, Vec<String>), ExecutorError> {
    let expanded = expand_home(command);
    let path = PathBuf::from(&expanded);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    match extension {
        "sh" => Ok(("bash".to_string(), vec![expanded])),
        "js" => Ok(("node".to_string(), vec![expanded])),
        _ => Err(ExecutorError::InvalidToolType(command.to_string())),
    }
}

fn normalize_working_dir(working_dir: &str) -> Option<String> {
    let trimmed = working_dir.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(expand_home(trimmed))
    }
}

pub fn expand_home(path: &str) -> String {
    if path == "~" {
        return dirs::home_dir()
            .map(|home| home.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped).to_string_lossy().into_owned();
        }
    }

    path.to_string()
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

pub fn substitute_params(command: &str, params: &HashMap<String, String>) -> String {
    params.iter().fold(command.to_string(), |acc, (key, value)| {
        let placeholder = format!("{{{{{key}}}}}");
        acc.replace(&placeholder, &shell_quote(value))
    })
}

pub fn truncate_output(s: String) -> String {
    let char_count = s.chars().count();
    if char_count <= OUTPUT_LIMIT {
        return s;
    }

    let truncated: String = s.chars().take(OUTPUT_LIMIT).collect();
    format!("{truncated}{TRUNCATION_SUFFIX}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Tool;

    fn make_tool(tool_type: &str, command: &str, working_dir: &str, timeout_ms: i64) -> Tool {
        Tool {
            id: format!("{tool_type}-tool"),
            name: format!("{tool_type}-tool"),
            icon: "terminal".to_string(),
            description: None,
            tool_type: tool_type.to_string(),
            command: command.to_string(),
            working_dir: working_dir.to_string(),
            timeout_ms,
            params: Vec::new(),
            sort_order: 0,
            enabled: true,
        }
    }

    #[test]
    fn substitute_params_quotes_values() {
        let params = HashMap::from([
            ("name".to_string(), "hello world".to_string()),
            ("path".to_string(), "a'b".to_string()),
        ]);

        let result = substitute_params("echo {{name}} {{path}}", &params);

        assert_eq!(result, "echo 'hello world' 'a'\"'\"'b'");
    }

    #[test]
    fn expand_home_replaces_tilde_prefix() {
        let home = dirs::home_dir().expect("home dir should exist");

        assert_eq!(expand_home("~"), home.to_string_lossy());
        assert_eq!(
            expand_home("~/work/quicktools"),
            home.join("work/quicktools").to_string_lossy()
        );
        assert_eq!(expand_home("/tmp/demo"), "/tmp/demo");
    }

    #[test]
    fn truncate_output_limits_long_strings() {
        let input = "a".repeat(2100);
        let truncated = truncate_output(input);

        assert_eq!(truncated.chars().count(), OUTPUT_LIMIT + TRUNCATION_SUFFIX.chars().count());
        assert!(truncated.ends_with(TRUNCATION_SUFFIX));
    }

    #[tokio::test]
    async fn execute_returns_timeout_result() {
        let tool = make_tool("shell", "sleep 1", "", 10);
        let params = HashMap::new();

        let result = Executor::execute_command(
            &tool,
            &params,
            "sh",
            vec!["-c".to_string(), "sleep 1".to_string()],
            Instant::now(),
        )
        .await
        .expect("timeout should return result");

        assert_eq!(result.status, "timeout");
        assert!(result.error.is_some());
    }
}
