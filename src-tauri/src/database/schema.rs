// schema.rs — 数据库建表语句

pub const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS log_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    enabled BOOLEAN NOT NULL DEFAULT 1,
    level TEXT NOT NULL DEFAULT 'info'
);

INSERT OR IGNORE INTO log_config (id, enabled, level) VALUES (1, 1, 'info');

CREATE TABLE IF NOT EXISTS tools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT DEFAULT 'terminal',
    description TEXT,
    type TEXT NOT NULL CHECK(type IN ('shell', 'script', 'open', 'notification')),
    command TEXT NOT NULL,
    working_dir TEXT DEFAULT '',
    timeout_ms INTEGER DEFAULT 60000,
    params TEXT DEFAULT '[]',
    sort_order INTEGER DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS execution_logs (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    params TEXT DEFAULT '{}',
    status TEXT NOT NULL CHECK(status IN ('success', 'failed', 'timeout')),
    duration_ms INTEGER NOT NULL,
    exit_code INTEGER,
    stdout TEXT DEFAULT '',
    stderr TEXT DEFAULT '',
    error TEXT,
    executed_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_logs_tool_id ON execution_logs(tool_id);
CREATE INDEX IF NOT EXISTS idx_logs_status ON execution_logs(status);
CREATE INDEX IF NOT EXISTS idx_logs_executed_at ON execution_logs(executed_at);
"#;