use crate::types::{AppState, GateError, GateRequest, GateResponse};

/// Handle an incoming /check request: classify → permission check → respond.
///
/// Spec: gate-server/spec.md > "Unix socket server for command authorization"
/// Tasks: 4.5
/// Async — coordinates classifier, permissions, approvals, tokens.
pub async fn handle_check_request(
    req: &GateRequest,
    state: &AppState,
) -> Result<GateResponse, GateError> {
    todo!("handle_check_request: classify command, check permissions, return allow/allow+env/pending/reject response")
}

/// Block until an approval request is resolved (approved/rejected/expired).
///
/// Spec: gate-server/spec.md > "Command requires approval"
/// Tasks: 4.6
/// Async — creates oneshot channel, stores in pending map, awaits resolution.
pub async fn wait_for_approval(
    state: &AppState,
    approval_id: &str,
) -> Result<GateResponse, GateError> {
    todo!("wait_for_approval: create oneshot channel, insert into pending map, await sender half")
}

/// Handle a single unix socket connection lifecycle.
///
/// Spec: gate-server/spec.md > "Socket peer authentication"
/// Tasks: 4.7
/// Async — verifies SO_PEERCRED, reads requests, sends responses, cleans up on disconnect.
pub async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: &AppState,
) -> Result<(), GateError> {
    todo!("handle_connection: verify peer UID via SO_PEERCRED, read request, call handle_check_request, write response, cleanup on disconnect")
}
