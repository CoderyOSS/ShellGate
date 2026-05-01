use crate::types::GateError;

/// Run MCP server over stdio — handles initialize, tools/list, tools/call.
///
/// Spec: gate-mcp/spec.md > "MCP server runs as part of gate-server"
/// Tasks: 5.1
/// Async — reads from stdin, writes to stdout, speaks MCP JSON-RPC protocol.
pub async fn run_mcp_server(db_path: &str) -> Result<(), GateError> {
    todo!("run_mcp_server: open SQLite at db_path, read MCP JSON-RPC from stdin, dispatch to tool handlers, write responses to stdout")
}

/// MCP tool: request_pre_approval — agent requests pre-approval for actions/repos.
///
/// Spec: gate-mcp/spec.md > "MCP server tools for agent pre-approval requests"
/// Tasks: 5.2
/// Async — creates pre-approval request (itself needs human approval), sends notification.
pub async fn tool_request_pre_approval(
    db_path: &str,
    actions: &[String],
    repos: &[String],
    ttl: &str,
    reason: &str,
) -> Result<serde_json::Value, GateError> {
    todo!("tool_request_pre_approval: create approval request with type=pre_approval, write to mcp-notify.fifo, return {{status:\"pending\", request_id}}")
}

/// MCP tool: get_approval_status — check status of an approval request.
///
/// Spec: gate-mcp/spec.md > "Check approval status tool"
/// Tasks: 5.3
/// Async — reads approval_requests row.
pub async fn tool_get_approval_status(
    db_path: &str,
    id: &str,
) -> Result<serde_json::Value, GateError> {
    todo!("tool_get_approval_status: call approvals::get_approval_status, return {{status, reason?}}")
}

/// MCP tool: list_grants — list active pre-approval grants.
///
/// Spec: gate-mcp/spec.md > "List active grants tool"
/// Tasks: 5.4
/// Async — reads grants table.
pub async fn tool_list_grants(
    db_path: &str,
) -> Result<serde_json::Value, GateError> {
    todo!("tool_list_grants: call grants::list_grants, return JSON array with action patterns, repo patterns, TTL remaining, use counts")
}

/// MCP tool: explain_blocked — show which commands are blocked and why.
///
/// Spec: gate-mcp/spec.md > "Explain blocked commands tool"
/// Tasks: 5.5
/// Async — reads default_permissions + grants to determine coverage gaps.
pub async fn tool_explain_blocked(
    db_path: &str,
) -> Result<serde_json::Value, GateError> {
    todo!("tool_explain_blocked: read default_permissions where state=OFF, check for partial grant coverage, return list of blocked categories with coverage info")
}
