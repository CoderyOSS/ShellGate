use crate::types::GateError;

pub async fn init_database(db_path: &str) -> Result<rusqlite::Connection, GateError> {
    let conn = rusqlite::Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), GateError> {
    let version: i64 = conn
        .query_row(
            "SELECT value FROM config WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let migrations: Vec<&str> = vec![
        MIGRATION_1,
        MIGRATION_2_AGENDAS,
    ];

    for (i, sql) in migrations.iter().enumerate() {
        let target = (i + 1) as i64;
        if version < target {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES ('schema_version', ?1)",
                [&target.to_string()],
            )?;
        }
    }

    Ok(())
}

const MIGRATION_1: &str = "
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS default_permissions (
    action TEXT PRIMARY KEY,
    state TEXT NOT NULL DEFAULT 'off',
    ttl_secs INTEGER
);

CREATE TABLE IF NOT EXISTS grants (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    repo_pattern TEXT NOT NULL,
    expires_at TEXT,
    max_uses INTEGER,
    use_count INTEGER NOT NULL DEFAULT 0,
    reason TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS approval_requests (
    id TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    args TEXT NOT NULL,
    action TEXT NOT NULL,
    repo TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_by TEXT,
    resolved_at TEXT,
    reason TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    command TEXT NOT NULL,
    args TEXT NOT NULL,
    action TEXT NOT NULL,
    repo TEXT NOT NULL,
    granted_by TEXT NOT NULL,
    exit_code INTEGER,
    agent_id TEXT NOT NULL,
    ttl_until TEXT
);
";

const MIGRATION_2_AGENDAS: &str = "
CREATE TABLE IF NOT EXISTS agendas (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    description TEXT NOT NULL,
    scope TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS derived_grants (
    id TEXT PRIMARY KEY,
    agenda_id TEXT NOT NULL REFERENCES agendas(id) ON DELETE CASCADE,
    command_pattern TEXT NOT NULL,
    args_pattern TEXT NOT NULL,
    path_pattern TEXT,
    notification TEXT NOT NULL DEFAULT 'silent',
    reason TEXT,
    confidence REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agendas_status ON agendas(status);
CREATE INDEX IF NOT EXISTS idx_agendas_expires ON agendas(expires_at);
CREATE INDEX IF NOT EXISTS idx_derived_grants_agenda ON derived_grants(agenda_id);
CREATE INDEX IF NOT EXISTS idx_derived_grants_command ON derived_grants(command_pattern);
";
