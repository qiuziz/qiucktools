use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub command: String,
    #[serde(rename = "workingDir")]
    pub working_dir: String,
    #[serde(rename = "timeoutMs")]
    pub timeout_ms: i64,
    pub params: Vec<ToolParam>,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolParam {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
    pub options: Option<Vec<ToolParamOption>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolParamOption {
    pub value: String,
    pub label: String,
}

/// Tool DAO - operates on a &Connection reference.
/// Caller holds the lock for the duration of the call.
pub struct ToolDao;

impl ToolDao {
    pub fn load_from_file(conn: &Connection, path: &str) -> Result<Vec<Tool>, Box<dyn Error>> {
        let json = std::fs::read_to_string(path)?;
        let tools: Vec<Tool> = serde_json::from_str(&json)?;
        Self::upsert_batch(conn, &tools)?;
        Ok(tools)
    }

    pub fn upsert_batch(conn: &Connection, tools: &[Tool]) -> SqlResult<()> {
        for tool in tools {
            let params_json =
                serde_json::to_string(&tool.params).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                r#"INSERT INTO tools (id, name, icon, description, type, command, working_dir, timeout_ms, params, sort_order, enabled, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)
                   ON CONFLICT(id) DO UPDATE SET
                     name = excluded.name,
                     icon = excluded.icon,
                     description = excluded.description,
                     type = excluded.type,
                     command = excluded.command,
                     working_dir = excluded.working_dir,
                     timeout_ms = excluded.timeout_ms,
                     params = excluded.params,
                     sort_order = excluded.sort_order,
                     enabled = excluded.enabled,
                     updated_at = CURRENT_TIMESTAMP"#,
                params![
                    tool.id,
                    tool.name,
                    tool.icon,
                    tool.description,
                    tool.tool_type,
                    tool.command,
                    tool.working_dir,
                    tool.timeout_ms,
                    params_json,
                    tool.sort_order,
                    tool.enabled,
                ],
            )?;
        }
        Ok(())
    }

    pub fn list(conn: &Connection) -> SqlResult<Vec<Tool>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, description, type, command, working_dir, timeout_ms, params, sort_order, enabled FROM tools ORDER BY sort_order ASC, name ASC",
        )?;
        let tools = stmt.query_map([], |row| {
            let params_json: String = row.get(8)?;
            let params: Vec<ToolParam> =
                serde_json::from_str(&params_json).unwrap_or_default();
            Ok(Tool {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                tool_type: row.get(4)?,
                command: row.get(5)?,
                working_dir: row.get(6)?,
                timeout_ms: row.get(7)?,
                params,
                sort_order: row.get(9)?,
                enabled: row.get::<_, i64>(10)? != 0,
            })
        })?;
        tools.collect()
    }

    pub fn get(conn: &Connection, id: &str) -> SqlResult<Option<Tool>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, description, type, command, working_dir, timeout_ms, params, sort_order, enabled FROM tools WHERE id = ?1",
        )?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            let params_json: String = row.get(8)?;
            let params: Vec<ToolParam> =
                serde_json::from_str(&params_json).unwrap_or_default();
            Ok(Some(Tool {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                tool_type: row.get(4)?,
                command: row.get(5)?,
                working_dir: row.get(6)?,
                timeout_ms: row.get(7)?,
                params,
                sort_order: row.get(9)?,
                enabled: row.get::<_, i64>(10)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn clear_all(conn: &Connection) -> SqlResult<()> {
        conn.execute("DELETE FROM tools", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_dao_upsert_and_list() {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migration::run_migrations(&conn).unwrap();

        let tool = Tool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            icon: "terminal".to_string(),
            description: Some("A test tool".to_string()),
            tool_type: "shell".to_string(),
            command: "echo hello".to_string(),
            working_dir: "~".to_string(),
            timeout_ms: 30000,
            params: vec![ToolParam {
                name: "arg".to_string(),
                label: "参数".to_string(),
                param_type: "text".to_string(),
                required: Some(true),
                default: None,
                options: None,
                min: None,
                max: None,
            }],
            sort_order: 0,
            enabled: true,
        };

        ToolDao::upsert_batch(&conn, &[tool.clone()]).unwrap();
        let tools = ToolDao::list(&conn).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "test-tool");
        assert_eq!(tools[0].name, "Test Tool");
        assert_eq!(tools[0].params.len(), 1);

        // Test get
        let found = ToolDao::get(&conn, "test-tool").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test-tool");

        // Test not found
        let missing = ToolDao::get(&conn, "nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_tool_dao_update() {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migration::run_migrations(&conn).unwrap();

        let tool1 = Tool {
            id: "update-test".to_string(),
            name: "Original Name".to_string(),
            icon: "star".to_string(),
            description: None,
            tool_type: "shell".to_string(),
            command: "echo original".to_string(),
            working_dir: "".to_string(),
            timeout_ms: 5000,
            params: vec![],
            sort_order: 0,
            enabled: true,
        };
        ToolDao::upsert_batch(&conn, &[tool1]).unwrap();

        let tool2 = Tool {
            id: "update-test".to_string(),
            name: "Updated Name".to_string(),
            icon: "star".to_string(),
            description: None,
            tool_type: "shell".to_string(),
            command: "echo updated".to_string(),
            working_dir: "".to_string(),
            timeout_ms: 10000,
            params: vec![],
            sort_order: 5,
            enabled: false,
        };
        ToolDao::upsert_batch(&conn, &[tool2]).unwrap();

        let tools = ToolDao::list(&conn).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "Updated Name");
        assert_eq!(tools[0].sort_order, 5);
        assert!(!tools[0].enabled);
    }
}