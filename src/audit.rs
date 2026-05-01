use crate::types::{AuditEntry, AuditQueryParams, GateError};

/// Insert an audit log entry for a command execution.
///
/// Spec: gate-server/spec.md > "Audit log retention"
/// Tasks: 2.3
/// Async — inserts row into audit_log table.
pub async fn log_command(
    conn: &rusqlite::Connection,
    timestamp: &str,
    command: &str,
    args: &str,
    action: &str,
    repo: &str,
    granted_by: &str,
    exit_code: Option<i32>,
    agent_id: &str,
) -> Result<(), GateError> {
    todo!("log_command: insert row into audit_log with all fields")
}

/// Log an approval decision (grant creation, approval approve/reject, permission config change).
///
/// Spec: gate-server/spec.md > "Audit log retention"
/// Tasks: 2.5
/// Async — inserts decision audit entry.
pub async fn log_approval_decision(
    conn: &rusqlite::Connection,
    approval_id: &str,
    decision: &str,
    resolved_by: &str,
    reason: Option<&str>,
) -> Result<(), GateError> {
    todo!("log_approval_decision: insert audit row for approval decision")
}

/// Delete expired audit log rows based on TTL.
///
/// Spec: gate-server/spec.md > "Audit log TTL cleanup"
/// Tasks: 2.4
/// Async — deletes rows where ttl_until < now, runs VACUUM if > 10% removed.
pub async fn cleanup_expired(
    conn: &rusqlite::Connection,
    retention_days: u32,
    max_db_size_mb: u32,
) -> Result<u64, GateError> {
    todo!("cleanup_expired: DELETE FROM audit_log WHERE created_at < now - retention, VACUUM if > 10% removed, check max_db_size")
}

/// Query audit log with filtering, searching, pagination.
///
/// Spec: gate-server/spec.md > "View audit log"
/// Tasks: 4.24
/// Async — reads from audit_log with dynamic WHERE clause.
pub async fn query_audit_log(
    conn: &rusqlite::Connection,
    params: &AuditQueryParams,
) -> Result<Vec<AuditEntry>, GateError> {
    todo!("query_audit_log: SELECT from audit_log with limit/offset/action/search/date filters, ORDER BY timestamp DESC")
}
