use crate::bonsai::BonsaiModel;
use crate::derived_grants;
use crate::pipeline::{DeliberationContext, Pipeline};
use crate::stages::allow_list::AllowListStage;
use crate::stages::catch_list::CatchListStage;
use crate::stages::human::HumanApprovalStage;
use crate::stages::llm::LlmStage;
use crate::types::{AppState, AuditEntry, GateError, GateRequest, GateResponse};

use std::sync::Arc;

pub async fn handle_check_request(
    req: &GateRequest,
    state: &AppState,
    model: Option<Arc<BonsaiModel>>,
) -> Result<GateResponse, GateError> {
    let conn = rusqlite::Connection::open(&state.config.gate.db_path)?;

    let agendas = derived_grants::agendas_to_summaries(&conn)?;
    let recent_history = get_recent_history(&conn, state.config.pipeline.stages.llm.max_context_commands)?;

    let ctx = DeliberationContext {
        command: req.command.clone(),
        args: req.args.clone(),
        cwd: req.cwd.clone(),
        pid: req.pid,
        agendas,
        recent_history,
        config: state.config.pipeline.clone(),
    };

    let flow = state
        .config
        .pipeline
        .get_flow("command_check")
        .cloned()
        .unwrap_or_default();

    let mut stages: Vec<Box<dyn crate::pipeline::DeliberationStage>> = Vec::new();
    let db_path = state.config.gate.db_path.clone();

    for name in &flow {
        match name.as_str() {
            "allow_list" => stages.push(Box::new(AllowListStage::new(
                state.config.pipeline.stages.allow_list.sampling_rate,
                db_path.clone(),
            ))),
            "catch_list" => {
                let catch = CatchListStage::new(&state.config.pipeline.stages.catch_list.patterns)
                    .map_err(|e| format!("catch list pattern error: {}", e))?;
                stages.push(Box::new(catch));
            }
            "llm" => {
                if let Some(ref m) = model {
                    stages.push(Box::new(LlmStage::new(m.clone(), db_path.clone())));
                }
            }
            "human" => stages.push(Box::new(HumanApprovalStage {
                timeout_secs: state.config.pipeline.stages.human.timeout_seconds,
                pending: state.pending.clone(),
            })),
            _ => tracing::warn!(stage = %name, "unknown pipeline stage, skipping"),
        }
    }

    let pipeline = Pipeline::new(stages);
    let result = pipeline.run(&ctx);

    for notification in &result.notifications {
        send_notification(notification, &state.config.telegram.bot_token, &state.config.telegram.chat_id).await;
    }

    if result.allowed {
        Ok(GateResponse {
            action: "allow".to_string(),
            env: None,
            approval_id: None,
            reason: result.block_reason,
        })
    } else {
        Ok(GateResponse {
            action: "reject".to_string(),
            env: None,
            approval_id: None,
            reason: result.block_reason,
        })
    }
}

pub async fn wait_for_approval(
    state: &AppState,
    approval_id: &str,
) -> Result<GateResponse, GateError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.pending.write().await.insert(approval_id.to_string(), tx);

    match tokio::time::timeout(
        std::time::Duration::from_secs(state.config.pipeline.stages.human.timeout_seconds),
        rx,
    )
    .await
    {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) => Ok(GateResponse {
            action: "reject".to_string(),
            env: None,
            approval_id: None,
            reason: Some("approval channel dropped".into()),
        }),
        Err(_) => {
            state.pending.write().await.remove(approval_id);
            Ok(GateResponse {
                action: "reject".to_string(),
                env: None,
                approval_id: Some(approval_id.to_string()),
                reason: Some("approval request expired".into()),
            })
        }
    }
}

pub async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: &AppState,
    model: Option<Arc<BonsaiModel>>,
) -> Result<(), GateError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::protocol;

    let mut buf = Vec::new();
    let (mut reader, mut writer) = stream.into_split();
    reader.read_to_end(&mut buf).await?;

    let request = protocol::parse_request(&buf)?;
    let response = handle_check_request(&request, state, model).await?;

    let response_bytes = protocol::serialize_response(&response);
    writer.write_all(&response_bytes).await?;

    Ok(())
}

fn get_recent_history(
    conn: &rusqlite::Connection,
    limit: usize,
) -> Result<Vec<AuditEntry>, GateError> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, command, args, action, repo, granted_by, exit_code, agent_id FROM audit_log ORDER BY id DESC LIMIT ?1",
    )?;

    let entries = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                command: row.get(2)?,
                args: row.get(3)?,
                action: row.get(4)?,
                repo: row.get(5)?,
                granted_by: row.get(6)?,
                exit_code: row.get(7)?,
                agent_id: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

async fn send_notification(
    msg: &crate::pipeline::NotifyMessage,
    _bot_token: &str,
    _chat_id: &i64,
) {
    tracing::info!(
        strategy = ?msg.strategy,
        command = %msg.command,
        reason = ?msg.reason,
        "notification dispatched"
    );
}
