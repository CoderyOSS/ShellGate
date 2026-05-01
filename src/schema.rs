use crate::types::GateError;

/// Initialize SQLite database with required tables and WAL mode.
///
/// Spec: gate-server/spec.md > "SQLite database"
/// Tasks: 2.1, 2.2
/// Async — creates/opens database file, runs DDL.
pub async fn init_database(db_path: &str) -> Result<rusqlite::Connection, GateError> {
    todo!("init_database: open SQLite at {db_path}, set PRAGMA journal_mode=WAL, busy_timeout=5000, create tables if not exist")
}

/// Run pending schema migrations.
///
/// Spec: gate-server/spec.md > "Schema migration"
/// Tasks: 2.1
/// Async — reads current version from config table, applies sequential SQL migrations.
pub async fn run_migrations(conn: &rusqlite::Connection) -> Result<(), GateError> {
    todo!("run_migrations: check config table for schema version, apply pending migrations in transactions")
}
