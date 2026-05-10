#[cfg(feature = "seccomp")]
mod seccomp_gate;
#[cfg(feature = "seccomp")]
mod session;

use std::sync::Arc;

#[allow(dead_code)]
mod agenda;
#[allow(dead_code)]
mod api;
#[allow(dead_code)]
mod approvals;
#[allow(dead_code)]
mod audit;
#[allow(dead_code)]
mod bootstrap;
#[allow(dead_code)]
mod classifier;
mod config;
#[allow(dead_code)]
mod derived_grants;
#[allow(dead_code)]
mod grants;
#[allow(dead_code)]
mod handler;
#[allow(dead_code)]
mod llm_client;
#[allow(dead_code)]
mod mcp;
#[allow(dead_code)]
mod permissions;
#[allow(dead_code)]
mod pipeline;
#[allow(dead_code)]
mod prompts;
#[allow(dead_code)]
mod proxy;
#[allow(dead_code)]
mod protocol;
#[allow(dead_code)]
mod schema;
#[allow(dead_code)]
mod server;
#[allow(dead_code)]
mod stages;
#[allow(dead_code)]
mod telegram;
#[allow(dead_code)]
mod tokens;
#[allow(dead_code)]
mod types;
#[allow(dead_code)]
mod watcher;

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--mcp" || args[1] == "mcp") {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            let config_path = std::env::var("GATE_CONFIG")
                .unwrap_or_else(|_| "/opt/gate/config.toml".to_string());
            let config = config::load_config(&config_path).expect("failed to load config");
            mcp::run_mcp_server(&config.gate.db_path)
                .await
                .expect("MCP server failed");
        });
    } else {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            run_gate_server().await.expect("gate server failed");
        });
    }
}

async fn run_gate_server() -> Result<(), types::GateError> {
    use std::collections::HashMap;

    let config_path = std::env::var("GATE_CONFIG")
        .unwrap_or_else(|_| "/opt/gate/config.toml".to_string());
    let mut config = config::load_config(&config_path).unwrap_or_else(|e| {
        tracing::warn!(path = %config_path, error = %e, "config file not found, using defaults");
        config::default_config()
    });
    config::apply_env_overrides(&mut config);

    schema::init_database(&config.gate.db_path).await?;

    let state = types::AppState {
        db_path: config.gate.db_path.clone(),
        config: config.clone(),
        pending: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    let model = server::load_llm_client(&config.pipeline.llm);

    let sessions = session::create_sessions();

    let telegram_config = config.telegram.clone();
    let telegram_state = state.clone();
    tokio::spawn(async move {
        if !telegram_config.bot_token.is_empty() {
            if let Err(e) = telegram::start_bot(telegram_config, telegram_state).await {
                tracing::error!(error = %e, "telegram bot failed");
            }
        }
    });

    if let Some(ref llm_model) = model {
        let watcher = watcher::OpenSpecWatcher::new(
            std::env::var("GATE_PROJECTS_DIR").unwrap_or_else(|_| "/home/gem/projects".to_string()),
            config.gate.db_path.clone(),
            llm_model.clone(),
        );
        tokio::spawn(async move {
            if let Err(e) = watcher.run().await {
                tracing::error!(error = %e, "openspec watcher failed");
            }
        });
    }

    if let Err(e) = config::start_config_watcher(config_path).await {
        tracing::warn!(error = %e, "config watcher not available, skipping inotify");
    }

    let socket_path = config.gate.socket_path.clone();
    if std::path::Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let parent_dir = std::path::Path::new(&socket_path)
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"));
    let _ = std::fs::create_dir_all(parent_dir);

    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    tracing::info!(socket = %socket_path, "gate-server listening");

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _addr)) => {
                        let state = state.clone();
                        let model = model.clone();
                        let sessions = sessions.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_gate_connection(stream, state, model, sessions).await {
                                tracing::error!(error = %e, "connection handler error");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "accept error");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down");
                break;
            }
        }
    }

    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

async fn handle_gate_connection(
    mut stream: tokio::net::UnixStream,
    state: types::AppState,
    model: Option<Arc<llm_client::LlmClient>>,
    sessions: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<u32, types::Session>>>,
) -> Result<(), types::GateError> {
    use std::os::unix::io::AsRawFd;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const SCM_RIGHTS: i32 = 1;

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let body = &buf[..n];

    if body.starts_with(b"{") {
        let msg: serde_json::Value = serde_json::from_slice(body)
            .map_err(|e| format!("invalid JSON: {}", e))?;

        match msg.get("type").and_then(|v| v.as_str()) {
            Some("spawn_shell") => {
                #[cfg(feature = "seccomp")]
                {
                    let session = session::spawn_shell(&state, &sessions, &model).await?;
                    let pty_fd = session.pty_fd;
                    let raw_sock = stream.as_raw_fd();

                    let data: u8 = 0;
                    let iov = libc::iovec {
                        iov_base: &data as *const u8 as *mut libc::c_void,
                        iov_len: 1,
                    };

                    let cmsg_len = unsafe {
                        libc::CMSG_SPACE(std::mem::size_of::<std::os::unix::io::RawFd>() as u32)
                    };
                    let mut cmsg_buf = vec![0u8; cmsg_len as usize];

                    let msgh = libc::msghdr {
                        msg_name: std::ptr::null_mut(),
                        msg_namelen: 0,
                        msg_iov: &iov as *const libc::iovec as *mut libc::iovec,
                        msg_iovlen: 1,
                        msg_control: cmsg_buf.as_mut_ptr() as *mut libc::c_void,
                        msg_controllen: cmsg_len as usize,
                        msg_flags: 0,
                    };

                    let cmsg = unsafe { libc::CMSG_FIRSTHDR(&msgh) };
                    unsafe {
                        (*cmsg).cmsg_level = libc::SOL_SOCKET;
                        (*cmsg).cmsg_type = SCM_RIGHTS;
                        (*cmsg).cmsg_len = libc::CMSG_LEN(
                            std::mem::size_of::<std::os::unix::io::RawFd>() as u32,
                        ) as usize;
                        std::ptr::copy(
                            &pty_fd,
                            libc::CMSG_DATA(cmsg) as *mut std::os::unix::io::RawFd,
                            1,
                        );
                    }

                    let ret = unsafe { libc::sendmsg(raw_sock, &msgh, 0) };
                    if ret < 0 {
                        return Err(format!(
                            "sendmsg failed: {}",
                            std::io::Error::last_os_error()
                        ).into());
                    }
                }
                #[cfg(not(feature = "seccomp"))]
                {
                    let _ = stream.write_all(b"seccomp feature not enabled").await;
                }
            }
            Some("check_command") => {
                if let Some(req_val) = msg.get("request") {
                    let req: types::GateRequest = serde_json::from_value(req_val.clone())
                        .map_err(|e| format!("invalid request: {}", e))?;
                    let response =
                        handler::handle_check_request(&req, &state, model).await?;
                    let resp_bytes = protocol::serialize_response(&response);
                    stream.write_all(&resp_bytes).await?;
                }
            }
            _ => {
                let _ = stream.write_all(b"unknown message type").await;
            }
        }
    } else {
        let len =
            u32::from_be_bytes([body[0], body[1], body[2], body[3]]) as usize;
        let mut full_body = body[4..].to_vec();
        while full_body.len() < len {
            let mut chunk = vec![0u8; len - full_body.len()];
            let n = stream.read(&mut chunk).await?;
            full_body.extend_from_slice(&chunk[..n]);
        }

        let request: types::GateRequest = serde_json::from_slice(&full_body)?;
        let response =
            handler::handle_check_request(&request, &state, model).await?;
        let response_bytes = protocol::serialize_response(&response);
        stream.write_all(&response_bytes).await?;
    }

    Ok(())
}
