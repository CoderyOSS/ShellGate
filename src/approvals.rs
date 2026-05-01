use crate::types::{ApprovalRequest, ApprovalStatus, GateError};

/// Create a new approval request in pending state.
///
/// Spec: gate-permissions/spec.md > "Approval request workflow"
/// Tasks: 3.2
/// Async — inserts row into approval_requests table.
pub async fn create_approval_request(
    conn: &rusqlite::Connection,
    command: &str,
    args: &[String],
    action: &str,
    repo: &str,
    agent_id: &str,
) -> Result<ApprovalRequest, GateError> {
    todo!("create_approval_request: generate UUID, insert into approval_requests with status=pending, return request")
}

/// Resolve an approval request (approve or reject). Uses atomic UPDATE WHERE status='pending'.
///
/// Spec: gate-server/spec.md > "Concurrent approval prevented"
/// Tasks: 3.2, 4.22
/// Async — updates approval_requests row, returns false if already resolved.
pub async fn resolve_approval(
    conn: &rusqlite::Connection,
    id: &str,
    approved: bool,
    resolved_by: &str,
    reason: Option<&str>,
) -> Result<bool, GateError> {
    todo!("resolve_approval: UPDATE approval_requests SET status=approved/rejected WHERE id=? AND status=pending, return rows_affected > 0")
}

/// List approval requests with optional status filter.
///
/// Spec: gate-server/spec.md > "List pending approvals"
/// Tasks: 4.22
/// Async — reads from approval_requests with optional WHERE status=?.
pub async fn list_approvals(
    conn: &rusqlite::Connection,
    status: Option<&ApprovalStatus>,
) -> Result<Vec<ApprovalRequest>, GateError> {
    todo!("list_approvals: SELECT from approval_requests with optional status filter")
}

/// Get the status of a single approval request.
///
/// Spec: gate-mcp/spec.md > "Check approval status tool"
/// Tasks: 5.3
/// Async — reads single row from approval_requests.
pub async fn get_approval_status(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<ApprovalStatus, GateError> {
    todo!("get_approval_status: SELECT status FROM approval_requests WHERE id = ?")
}

/// Expire approval requests that have been pending too long.
///
/// Spec: gate-permissions/spec.md > "Approval request expires"
/// Tasks: 3.2
/// Async — updates expired pending requests, notifies waiting channels.
pub async fn expire_old_approvals(
    conn: &rusqlite::Connection,
    timeout_secs: u64,
) -> Result<u64, GateError> {
    todo!("expire_old_approvals: UPDATE approval_requests SET status=expired WHERE status=pending AND created_at < now - timeout")
}
