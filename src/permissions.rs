use crate::types::{Decision, DefaultPermission, GateError, Grant, PermissionState};

/// Unified permission check — single code path for all decisions.
/// Evaluation order: blocked list → active grants → default state → needs_approval.
///
/// Spec: gate-permissions/spec.md > "Unified permission check"
/// Tasks: 3.1
/// Pure function — no async, no DB access.
/// Suggested return: could also be Result<Decision, GateError> if error handling needed.
pub fn check_permission(
    action: &str,
    repo: &str,
    grants: &[Grant],
    defaults: &[DefaultPermission],
    blocked: &[String],
) -> Decision {
    todo!("check_permission: implement evaluation order — blocked → grants → defaults → needs_approval")
}

/// Seed default permissions on first run.
///
/// Spec: gate-permissions/spec.md > "Default permission states"
/// Tasks: 2.2
/// Async — inserts default rows into default_permissions table.
pub async fn seed_default_permissions(
    conn: &rusqlite::Connection,
) -> Result<(), GateError> {
    todo!("seed_default_permissions: insert ON/OFF defaults for all permission categories, set auth:* as blocked")
}

/// Get all default permission states.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async — reads from default_permissions table.
pub async fn get_default_permissions(
    conn: &rusqlite::Connection,
) -> Result<Vec<DefaultPermission>, GateError> {
    todo!("get_default_permissions: SELECT all rows from default_permissions")
}

/// Update a single default permission state.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async — updates default_permissions row.
pub async fn update_default_permission(
    conn: &rusqlite::Connection,
    action: &str,
    state: PermissionState,
) -> Result<(), GateError> {
    todo!("update_default_permission: UPDATE default_permissions SET state = ? WHERE action = ?")
}

/// Reset all permissions to initial seed defaults.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async — deletes all rows, re-seeds defaults.
pub async fn reset_permissions_to_defaults(
    conn: &rusqlite::Connection,
) -> Result<(), GateError> {
    todo!("reset_permissions_to_defaults: DELETE FROM default_permissions, then seed_default_permissions")
}
