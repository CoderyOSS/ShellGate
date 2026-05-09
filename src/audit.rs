use crate::types::{AuditEntry, AuditQueryParams, GateError};

/// Insert an audit log entry for a command execution.
///
/// Spec: gate-server/spec.md > "Audit log retention"
/// Tasks: 2.3
/// Async — inserts row into audit_log table.
#[allow(clippy::too_many_arguments)]
pub async fn log_command(
    _conn: &rusqlite::Connection,
    _timestamp: &str,
    _command: &str,
    _args: &str,
    _action: &str,
    _repo: &str,
    _granted_by: &str,
    _exit_code: Option<i32>,
    _agent_id: &str,
) -> Result<(), GateError> {
    todo!("log_command: insert row into audit_log with all fields")
}

/// Log an approval decision (grant creation, approval approve/reject, permission config change).
///
/// Spec: gate-server/spec.md > "Audit log retention"
/// Tasks: 2.5
/// Async — inserts decision audit entry.
pub async fn log_approval_decision(
    _conn: &rusqlite::Connection,
    _approval_id: &str,
    _decision: &str,
    _resolved_by: &str,
    _reason: Option<&str>,
) -> Result<(), GateError> {
    todo!("log_approval_decision: insert audit row for approval decision")
}

/// Delete expired audit log rows based on TTL.
///
/// Spec: gate-server/spec.md > "Audit log TTL cleanup"
/// Tasks: 2.4
/// Async — deletes rows where ttl_until < now, runs VACUUM if > 10% removed.
pub async fn cleanup_expired(
    _conn: &rusqlite::Connection,
    _retention_days: u32,
    _max_db_size_mb: u32,
) -> Result<u64, GateError> {
    todo!("cleanup_expired: DELETE FROM audit_log WHERE created_at < now - retention, VACUUM if > 10% removed, check max_db_size")
}

/// Query audit log with filtering, searching, pagination.
///
/// Spec: gate-server/spec.md > "View audit log"
/// Tasks: 4.24
/// Async — reads from audit_log with dynamic WHERE clause.
pub async fn query_audit_log(
    _conn: &rusqlite::Connection,
    _params: &AuditQueryParams,
) -> Result<Vec<AuditEntry>, GateError> {
    todo!("query_audit_log: SELECT from audit_log with limit/offset/action/search/date filters, ORDER BY timestamp DESC")
}
