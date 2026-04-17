use serde::{Deserialize, Serialize};

/// Application type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
    Codex,
    Gemini,
    OpenCode,
    OpenClaw,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppType::Claude => "claude",
            AppType::Codex => "codex",
            AppType::Gemini => "gemini",
            AppType::OpenCode => "opencode",
            AppType::OpenClaw => "openclaw",
        }
    }

    pub fn all() -> Vec<AppType> {
        vec![
            AppType::Claude,
            AppType::Codex,
            AppType::Gemini,
            AppType::OpenCode,
            AppType::OpenClaw,
        ]
    }

    pub fn is_additive_mode(&self) -> bool {
        matches!(self, AppType::OpenCode | AppType::OpenClaw)
    }
}

/// Installed skill info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub id: String,
    pub name: String,
    pub directory: String,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server_config: String,
    pub description: Option<String>,
}

/// MCP apps (mapping of app types to MCP servers)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpApps {
    pub claude: Vec<String>,
    pub codex: Vec<String>,
    pub gemini: Vec<String>,
}

/// Multi-app configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiAppConfig {
    pub app_type: AppType,
    pub current_provider: Option<String>,
    pub providers: Vec<ProviderConfig>,
    pub mcp_servers: Vec<McpServer>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub settings_config: String,
}

/// Skill apps (mapping of app types to skills)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillApps {
    pub claude: Vec<InstalledSkill>,
    pub codex: Vec<InstalledSkill>,
    pub gemini: Vec<InstalledSkill>,
}