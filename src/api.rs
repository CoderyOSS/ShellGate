use crate::types::{AppState, AuditQueryParams, CreateGrantRequest};
use axum::extract::{Path, Query, State};
use axum::response::sse::Event;
use axum::response::Sse;
use axum::Json;
use serde::Serialize;
use std::convert::Infallible;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

/// Build the axum router with all API routes.
///
/// Spec: gate-server/spec.md > "REST API for web UI and integrations"
/// Tasks: 4.21
/// Pure function — constructs router.
pub fn build_router(state: AppState) -> axum::Router {
    todo!("build_router: create axum Router with /api/approvals, /api/grants, /api/audit, /api/permissions, /api/config, /api/events, /health routes")
}

/// Health check endpoint.
///
/// Spec: gate-server/spec.md > "Health check"
/// Tasks: 4.21
/// Async handler — checks DB connectivity.
pub async fn health_handler() -> Json<serde_json::Value> {
    todo!("health_handler: return {{\"status\":\"ok\",\"db_connected\":true,\"uptime_secs\":N}}")
}

/// List approval requests with optional status filter.
///
/// Spec: gate-server/spec.md > "List pending approvals"
/// Tasks: 4.22
/// Async handler — queries approval_requests.
pub async fn list_approvals_handler(
    State(_state): State<AppState>,
    Query(_params): Query<serde_json::Value>,
) -> Json<serde_json::Value> {
    todo!("list_approvals_handler: parse status filter, call approvals::list_approvals, return JSON array")
}

/// Approve an approval request.
///
/// Spec: gate-server/spec.md > "Approve an approval request"
/// Tasks: 4.22
/// Async handler — atomic approve, resolves waiting socket.
pub async fn approve_handler(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!("approve_handler: call approvals::resolve_approval(id, true), notify pending channel, return 409 if already resolved")
}

/// Reject an approval request.
///
/// Spec: gate-server/spec.md > "Reject an approval request"
/// Tasks: 4.22
/// Async handler — atomic reject, resolves waiting socket.
pub async fn reject_handler(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!("reject_handler: call approvals::resolve_approval(id, false, reason), notify pending channel, return 409 if already resolved")
}

/// Create a new pre-approval grant.
///
/// Spec: gate-server/spec.md > "Create a pre-approval grant"
/// Tasks: 4.23
/// Async handler — validates input, creates grant.
pub async fn create_grant_handler(
    State(_state): State<AppState>,
    Json(_body): Json<CreateGrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!("create_grant_handler: validate action/repo/ttl, call grants::create_grant, return created grant")
}

/// List active grants.
///
/// Spec: gate-server/spec.md > "List grants"
/// Tasks: 4.23
/// Async handler — returns grant list.
pub async fn list_grants_handler(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    todo!("list_grants_handler: call grants::list_grants, return JSON array")
}

/// Revoke a grant by ID.
///
/// Spec: gate-server/spec.md > "Revoke a grant"
/// Tasks: 4.23
/// Async handler — revokes grant.
pub async fn revoke_grant_handler(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!("revoke_grant_handler: call grants::revoke_grant(id), return 404 if not found")
}

/// Query audit log with pagination, filtering, search.
///
/// Spec: gate-server/spec.md > "View audit log"
/// Tasks: 4.24
/// Async handler — queries audit_log.
pub async fn audit_log_handler(
    State(_state): State<AppState>,
    Query(_params): Query<AuditQueryParams>,
) -> Json<serde_json::Value> {
    todo!("audit_log_handler: parse query params, call audit::query_audit_log, return JSON array")
}

/// Get all default permission states.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async handler — returns permission defaults.
pub async fn get_permissions_handler(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    todo!("get_permissions_handler: call permissions::get_default_permissions, return JSON array")
}

/// Update a default permission state.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async handler — updates permission, logs change.
pub async fn update_permission_handler(
    State(_state): State<AppState>,
    Json(_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!("update_permission_handler: parse action+state, call permissions::update_default_permission, log change to audit")
}

/// Reset all permissions to seed defaults.
///
/// Spec: gate-permissions/spec.md > "Permission configuration via web UI and API"
/// Tasks: 4.25
/// Async handler — resets permissions.
pub async fn reset_permissions_handler(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    todo!("reset_permissions_handler: call permissions::reset_permissions_to_defaults, return confirmation")
}

/// SSE endpoint for real-time approval and grant updates.
///
/// Spec: gate-web/spec.md > "Real-time update on new request"
/// Tasks: 4.26
/// Async handler — streams SSE events.
pub async fn sse_handler(
    State(_state): State<AppState>,
) -> Sse<tokio_stream::wrappers::ReceiverStream<Result<Event, Infallible>>> {
    todo!("sse_handler: create event stream, subscribe to approval/grant changes, include keepalive heartbeat every 30s")
}
