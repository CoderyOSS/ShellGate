#[cfg(feature = "seccomp")]
use std::collections::HashMap;
#[cfg(feature = "seccomp")]
use std::os::unix::io::RawFd;
#[cfg(feature = "seccomp")]
use std::sync::Arc;

#[cfg(feature = "seccomp")]
use tokio::sync::RwLock;

#[cfg(feature = "seccomp")]
use crate::derived_grants;
#[cfg(feature = "seccomp")]
use crate::handler;
#[cfg(feature = "seccomp")]
use crate::llm_client::LlmClient;
#[cfg(feature = "seccomp")]
use crate::pipeline::{DeliberationContext, Pipeline, PipelineResult};
#[cfg(feature = "seccomp")]
use crate::seccomp_gate;
#[cfg(feature = "seccomp")]
use crate::stages::allow_list::AllowListStage;
#[cfg(feature = "seccomp")]
use crate::stages::catch_list::CatchListStage;
#[cfg(feature = "seccomp")]
use crate::stages::human::HumanApprovalStage;
#[cfg(feature = "seccomp")]
use crate::stages::llm::LlmStage;
#[cfg(feature = "seccomp")]
use crate::types::{AppState, GateError, GateRequest, Session};

#[cfg(feature = "seccomp")]
pub type Sessions = Arc<RwLock<HashMap<u32, Session>>>;

#[cfg(feature = "seccomp")]
pub fn create_sessions() -> Sessions {
    Arc::new(RwLock::new(HashMap::new()))
}

#[cfg(feature = "seccomp")]
pub async fn spawn_shell(
    state: &AppState,
    sessions: &Sessions,
    model: &Option<Arc<LlmClient>>,
) -> Result<Session, GateError> {
    let (pty_master, pty_slave) = unsafe { seccomp_gate::create_pty()? };
    let (parent_sock, child_sock) = unsafe { seccomp_gate::create_socketpair()? };

    let shell_binary = std::env::var("GATE_SHELL_ENGINE")
        .unwrap_or_else(|_| "/opt/gate/shell-engine".to_string());

    let pid = unsafe {
        seccomp_gate::fork_and_trap(
            &shell_binary,
            pty_master,
            pty_slave,
            child_sock,
        )?
    };

    unsafe { libc::close(child_sock) };
    unsafe { libc::close(pty_slave) };

    let notify_fd = unsafe { seccomp_gate::recv_fd(parent_sock)? };
    unsafe { libc::close(parent_sock) };

    let session = Session {
        id: uuid::Uuid::new_v4().to_string(),
        pid: pid as u32,
        notify_fd,
        pty_fd: pty_master,
        cwd: "/".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    sessions.write().await.insert(pid as u32, session.clone());

    tracing::info!(
        session_id = %session.id,
        pid = pid,
        "shell session spawned"
    );

    let sessions_clone = sessions.clone();
    let state_clone = state.clone();
    let model_clone = model.clone();

    tokio::spawn(async move {
        notify_loop(notify_fd, pid as u32, sessions_clone, state_clone, model_clone).await;
    });

    Ok(session)
}

#[cfg(feature = "seccomp")]
async fn notify_loop(
    notify_fd: RawFd,
    shell_pid: u32,
    sessions: Sessions,
    state: AppState,
    model: Option<Arc<LlmClient>>,
) {
    loop {
        let notification = match seccomp_gate::receive_notification(notify_fd) {
            Ok(n) => n,
            Err(e) => {
                tracing::error!(pid = shell_pid, error = %e, "notify receive failed, stopping notify loop");
                break;
            }
        };

        let argv = unsafe {
            seccomp_gate::read_target_argv(notification.pid, notification.args[1])
                .unwrap_or_else(|e| {
                    tracing::warn!(pid = notification.pid, error = %e, "failed to read argv");
                    vec!["<unknown>".to_string()]
                })
        };

        let command = if argv.is_empty() {
            "<unknown>".to_string()
        } else {
            argv[0].clone()
        };
        let args: Vec<String> = if argv.len() > 1 {
            argv[1..].to_vec()
        } else {
            vec![]
        };

        let cwd = if let Some(session) = sessions.read().await.get(&shell_pid) {
            session.cwd.clone()
        } else {
            "/".to_string()
        };

        let gate_req = GateRequest {
            command: command.clone(),
            args: args.clone(),
            cwd,
            pid: notification.pid,
        };

        let pipeline_result =
            match run_pipeline_for_notify(&state, &model, &gate_req).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "pipeline error");
                    let _ =
                        seccomp_gate::respond_block(notify_fd, notification.id, libc::EPERM);
                    continue;
                }
            };

        let (allowed, errno) =
            seccomp_gate::pipeline_result_to_syscall_response(&pipeline_result);

        let respond_result = if allowed {
            seccomp_gate::respond_continue(notify_fd, notification.id)
        } else {
            seccomp_gate::respond_block(
                notify_fd,
                notification.id,
                errno.unwrap_or(libc::EPERM),
            )
        };

        if let Err(e) = respond_result {
            tracing::error!(error = %e, "failed to respond to notification");
        }

        tracing::info!(
            pid = notification.pid,
            command = %command,
            allowed = allowed,
            "seccomp notification handled"
        );
    }

    seccomp_gate::close_fd(notify_fd);
    sessions.write().await.remove(&shell_pid);
    tracing::info!(pid = shell_pid, "shell session cleaned up");
}

#[cfg(feature = "seccomp")]
async fn run_pipeline_for_notify(
    state: &AppState,
    model: &Option<Arc<LlmClient>>,
    req: &GateRequest,
) -> Result<PipelineResult, GateError> {
    let conn = rusqlite::Connection::open(&state.config.gate.db_path)?;

    let agendas = derived_grants::agendas_to_summaries(&conn)?;
    let recent_history = handler::get_recent_history_raw(
        &conn,
        state.config.pipeline.stages.llm.max_context_commands,
    )?;

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

    let db_path = state.config.gate.db_path.clone();
    let mut stages: Vec<Box<dyn crate::pipeline::DeliberationStage>> = Vec::new();

    for name in &flow {
        match name.as_str() {
            "allow_list" => stages.push(Box::new(AllowListStage::new(
                state.config.pipeline.stages.allow_list.sampling_rate,
                db_path.clone(),
            ))),
            "catch_list" => {
                let catch =
                    CatchListStage::new(&state.config.pipeline.stages.catch_list.patterns)
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
        handler::dispatch_notification(
            notification,
            &state.config.telegram.bot_token,
            &state.config.telegram.chat_id,
        )
        .await;
    }

    Ok(result)
}
