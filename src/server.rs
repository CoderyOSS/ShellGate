use crate::types::{AppState, Config, GateError};

/// Start the gate-server: unix socket listener + REST API server.
///
/// Spec: gate-server/spec.md > "Unix socket server for command authorization"
/// Tasks: 1.1, 1.3
/// Async — binds unix socket, binds TCP for REST, runs both servers.
pub async fn run_server(config: Config) -> Result<(), GateError> {
    todo!("run_server: init database, build AppState, bind unix socket at config.gate.socket_path, start REST API on config.gate.rest_port, start background tasks (TTL cleanup, token refresh, approval expiry), serve static web UI")
}

/// Accept and dispatch a unix socket connection.
///
/// Spec: gate-server/spec.md > "Socket peer authentication"
/// Tasks: 1.3
/// Async — accepts connection, spawns handler task.
pub async fn accept_unix_connection(
    listener: &tokio::net::UnixListener,
    state: &AppState,
) -> Result<(), GateError> {
    todo!("accept_unix_connection: loop on listener.accept(), spawn tokio task for each connection calling handler::handle_connection")
}

/// Wait for shutdown signal (SIGTERM/SIGQUIT).
///
/// Spec: gate-server/spec.md > "Graceful shutdown"
/// Tasks: graceful shutdown
/// Async — listens for unix signals.
pub async fn shutdown_signal() -> Result<(), GateError> {
    todo!("shutdown_signal: listen for SIGTERM (graceful) and SIGQUIT (immediate) via tokio::signal")
}

/// Perform graceful shutdown: stop accepting, drain requests, checkpoint WAL.
///
/// Spec: gate-server/spec.md > "Graceful shutdown"
/// Tasks: graceful shutdown
/// Async — drains connections with 10s timeout, runs PRAGMA wal_checkpoint.
pub async fn graceful_shutdown(state: &AppState, timeout_secs: u64) -> Result<(), GateError> {
    todo!("graceful_shutdown: stop accepting new connections, drain in-flight with {timeout_secs}s timeout, PRAGMA wal_checkpoint, exit 0")
}
