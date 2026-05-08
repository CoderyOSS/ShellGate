use crate::bonsai::BonsaiModel;
use crate::handler;
use crate::pipeline::BonsaiConfig;
use crate::schema;
use crate::types::{AppState, Config, GateError};

use std::collections::HashMap;
use std::sync::Arc;

pub async fn run_server(config: Config) -> Result<(), GateError> {
    let db_path = config.gate.db_path.clone();
    schema::init_database(&db_path).await?;

    tracing::info!(db = %db_path, "database initialized");

    let state = AppState {
        db_path: db_path.clone(),
        config: config.clone(),
        pending: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    let model = load_bonsai_model(&config.pipeline.bonsai);

    let socket_path = config.gate.socket_path.clone();
    if std::path::Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    tracing::info!(socket = %socket_path, "listening on unix socket");

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _addr)) => {
                        let state = state.clone();
                        let model = model.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handler::handle_connection(stream, &state, model).await {
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

fn load_bonsai_model(config: &BonsaiConfig) -> Option<Arc<BonsaiModel>> {
    match BonsaiModel::load(config) {
        Ok(model) if model.is_available() => {
            tracing::info!("bonsai model loaded");
            Some(Arc::new(model))
        }
        Ok(_) => {
            tracing::info!("bonsai model unavailable (no model file), LLM stage will pass");
            None
        }
        Err(e) => {
            tracing::warn!(error = %e, "bonsai model load failed, LLM stage will pass");
            None
        }
    }
}

pub async fn accept_unix_connection(
    _listener: &tokio::net::UnixListener,
    _state: &AppState,
) -> Result<(), GateError> {
    unimplemented!("inlined into run_server accept loop")
}

pub async fn shutdown_signal() -> Result<(), GateError> {
    tokio::signal::ctrl_c().await.map_err(|e| e.into())
}

pub async fn graceful_shutdown(
    _state: &AppState,
    _timeout_secs: u64,
) -> Result<(), GateError> {
    tracing::info!("graceful shutdown requested");
    Ok(())
}
