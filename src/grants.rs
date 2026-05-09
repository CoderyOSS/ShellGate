use crate::types::{GateError, Grant};

/// Create a new pre-approval grant.
///
/// Spec: gate-permissions/spec.md > "Pre-approval grants"
/// Tasks: 4.3
/// Async — validates patterns, calculates expires_at, inserts into grants table.
pub async fn create_grant(
    _conn: &rusqlite::Connection,
    _action: &str,
    _repo_pattern: &str,
    _ttl_secs: u64,
    _max_uses: Option<u64>,
    _reason: &str,
    _created_by: &str,
) -> Result<Grant, GateError> {
    todo!("create_grant: validate action/repo_pattern, calculate expires_at = now + ttl_secs, insert into grants table")
}

/// List all active (non-expired, non-revoked) grants.
///
/// Spec: gate-server/spec.md > "List grants"
/// Tasks: 4.23
/// Async — reads from grants table.
pub async fn list_grants(
    _conn: &rusqlite::Connection,
) -> Result<Vec<Grant>, GateError> {
    todo!("list_grants: SELECT from grants WHERE revoked_at IS NULL AND (expires_at IS NULL OR expires_at > now)")
}

/// Revoke a grant by ID.
///
/// Spec: gate-server/spec.md > "Revoke a grant"
/// Tasks: 4.23
/// Async — sets revoked_at timestamp.
pub async fn revoke_grant(
    _conn: &rusqlite::Connection,
    _id: &str,
) -> Result<bool, GateError> {
    todo!("revoke_grant: UPDATE grants SET revoked_at = now WHERE id = ? AND revoked_at IS NULL, return rows_affected > 0")
}

/// Find grants matching a given action and repo (for permission check).
///
/// Spec: gate-permissions/spec.md > "Grant overrides default"
/// Tasks: 3.1
/// Async — queries grants with glob matching on repo_pattern.
pub async fn find_matching_grants(
    _conn: &rusqlite::Connection,
    _action: &str,
    _repo: &str,
) -> Result<Vec<Grant>, GateError> {
    todo!("find_matching_grants: SELECT grants where action matches and repo_pattern glob-matches repo, not expired, not revoked")
}
