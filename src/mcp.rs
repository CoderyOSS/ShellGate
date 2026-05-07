use crate::agenda;
use crate::bonsai::BonsaiModel;
use crate::stages::llm::generate_rules_for_agenda;
use crate::types::GateError;

use std::sync::Arc;

pub async fn run_mcp_server(db_path: &str) -> Result<(), GateError> {
    let conn = rusqlite::Connection::open(db_path)?;
    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let stdout = tokio::io::stdout();

    loop {
        let mut line = String::new();
        if tokio::io::BufRead::read_line(&mut stdin, &mut line).await? == 0 {
            break;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "failed to parse MCP request");
                continue;
            }
        };

        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = request.get("id").cloned();

        let response = match method {
            "initialize" => mcp_initialize(id),
            "tools/list" => mcp_tools_list(id),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or_default();
                mcp_tools_call(id, params, &conn).await
            }
            _ => mcp_error(id, format!("unknown method: {}", method)),
        };

        if let Some(output) = response {
            println!("{}", serde_json::to_string(&output)?);
        }
    }

    Ok(())
}

fn mcp_initialize(id: Option<serde_json::Value>) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "gate-server", "version": "0.2.0" }
        }
    }))
}

fn mcp_tools_list(id: Option<serde_json::Value>) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "set_agenda",
                    "description": "Set the current project agenda so ShellGate can pre-approve expected commands",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "description": { "type": "string", "description": "Brief description of the current task" },
                            "scope": { "type": "string", "description": "File paths/globs relevant to the task" },
                            "ttl_secs": { "type": "integer", "description": "How long this agenda should remain active (default 86400)" }
                        },
                        "required": ["description"]
                    }
                },
                {
                    "name": "request_pre_approval",
                    "description": "Request pre-approval for specific actions/repos. Checks allow_list before escalating to human.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "actions": { "type": "array", "items": { "type": "string" }, "description": "Actions to pre-approve (e.g. [\"git:push\", \"pr:create\"])" },
                            "repos": { "type": "array", "items": { "type": "string" }, "description": "Repo patterns (e.g. [\"owner/repo\"])" },
                            "ttl": { "type": "string", "description": "Duration for the grant (e.g. \"2h\")" },
                            "reason": { "type": "string", "description": "Why this pre-approval is needed" }
                        },
                        "required": ["actions", "reason"]
                    }
                },
                {
                    "name": "get_approval_status",
                    "description": "Check status of an approval request",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string", "description": "Approval request ID" }
                        },
                        "required": ["id"]
                    }
                },
                {
                    "name": "list_grants",
                    "description": "List active pre-approval grants",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "explain_blocked",
                    "description": "Show which permission categories are OFF and why",
                    "inputSchema": { "type": "object", "properties": {} }
                }
            ]
        }
    }))
}

async fn mcp_tools_call(
    id: Option<serde_json::Value>,
    params: serde_json::Value,
    conn: &rusqlite::Connection,
) -> Option<serde_json::Value> {
    let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    let result = match tool_name {
        "set_agenda" => tool_set_agenda(conn, &arguments),
        "request_pre_approval" => tool_request_pre_approval(conn, &arguments),
        "get_approval_status" => tool_get_approval_status(conn, &arguments),
        "list_grants" => tool_list_grants(conn),
        "explain_blocked" => tool_explain_blocked(conn),
        _ => Err(format!("unknown tool: {}", tool_name).into()),
    };

    match result {
        Ok(value) => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }] }
        })),
        Err(e) => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32000, "message": format!("{}", e) }
        })),
    }
}

fn tool_set_agenda(
    conn: &rusqlite::Connection,
    args: &serde_json::Value,
) -> Result<serde_json::Value, GateError> {
    let description = args.get("description").and_then(|d| d.as_str()).unwrap_or("");
    let scope = args.get("scope").and_then(|s| s.as_str());
    let ttl_secs = args.get("ttl_secs").and_then(|t| t.as_u64());

    let agenda = agenda::create_agenda(conn, "mcp", description, scope, ttl_secs)?;

    Ok(serde_json::json!({
        "agenda_id": agenda.id,
        "description": agenda.description,
        "scope": agenda.scope,
        "expires_at": agenda.expires_at,
        "status": "active"
    }))
}

fn tool_request_pre_approval(
    conn: &rusqlite::Connection,
    args: &serde_json::Value,
) -> Result<serde_json::Value, GateError> {
    let actions = args.get("actions")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
        .unwrap_or_default();
    let reason = args.get("reason").and_then(|r| r.as_str()).unwrap_or("");

    let request_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO approval_requests (id, command, args, action, repo, status, created_at, reason) VALUES (?1, 'pre_approval', '', '', '', 'pending', ?2, ?3)",
        rusqlite::params![&request_id, &now, reason],
    )?;

    tracing::info!(request_id = %request_id, actions = ?actions, "MCP pre-approval request created");

    Ok(serde_json::json!({
        "request_id": request_id,
        "status": "pending",
        "actions": actions,
        "message": "Pre-approval request submitted. Use get_approval_status to check."
    }))
}

fn tool_get_approval_status(
    conn: &rusqlite::Connection,
    args: &serde_json::Value,
) -> Result<serde_json::Value, GateError> {
    let id = args.get("id").and_then(|i| i.as_str()).unwrap_or("");

    let result: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT status, reason FROM approval_requests WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    match result {
        Some((status, reason)) => Ok(serde_json::json!({
            "id": id,
            "status": status,
            "reason": reason
        })),
        None => Ok(serde_json::json!({
            "id": id,
            "status": "not_found"
        })),
    }
}

fn tool_list_grants(conn: &rusqlite::Connection) -> Result<serde_json::Value, GateError> {
    let mut stmt = conn.prepare(
        "SELECT id, action, repo_pattern, expires_at, max_uses, use_count, reason FROM grants ORDER BY created_at DESC"
    )?;

    let grants: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "action": row.get::<_, String>(1)?,
                "repo_pattern": row.get::<_, String>(2)?,
                "expires_at": row.get::<_, Option<String>>(3)?,
                "max_uses": row.get::<_, Option<u64>>(4)?,
                "use_count": row.get::<_, u64>(5)?,
                "reason": row.get::<_, String>(6)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(serde_json::json!({ "grants": grants }))
}

fn tool_explain_blocked(conn: &rusqlite::Connection) -> Result<serde_json::Value, GateError> {
    let mut stmt = conn.prepare(
        "SELECT action, state FROM default_permissions WHERE state = 'off'"
    )?;

    let blocked: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "action": row.get::<_, String>(0)?,
                "state": row.get::<_, String>(1)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(serde_json::json!({ "blocked_categories": blocked }))
}

fn mcp_error(id: Option<serde_json::Value>, message: String) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": message }
    }))
}
