use crate::types::{CommandOutput, GateError, OutputChunk};

/// Execute a command on the host (gh.real) with given environment variables.
///
/// Spec: gate-server/spec.md > "Proxy mode execution for gh commands"
/// Tasks: 4.8
/// Async — spawns child process, waits for completion.
pub async fn execute_command(
    cmd: &str,
    args: &[String],
    env: &[(String, String)],
) -> Result<CommandOutput, GateError> {
    todo!("execute_command: spawn Command::new(cmd).args(args).envs(env), capture stdout/stderr/exit_code")
}

/// Stream output chunks from a running child process.
///
/// Spec: gate-server/spec.md > "Proxy mode execution for gh commands"
/// Tasks: 4.9
/// Async — reads stdout/stderr as they become available.
pub async fn stream_output(
    _stdout: tokio::process::ChildStdout,
    _stderr: tokio::process::ChildStderr,
) -> tokio_stream::wrappers::ReceiverStream<OutputChunk> {
    todo!("stream_output: merge stdout/stderr tokio::io::BufReader lines into OutputChunk stream")
}

/// Send keepalive ping frame every 60 seconds during long-running commands.
///
/// Spec: gate-server/spec.md > "Long-running gh command"
/// Tasks: 4.10
/// Async — periodic interval task.
pub async fn send_keepalive(
    stream: &mut tokio::net::UnixStream,
) -> Result<(), GateError> {
    todo!("send_keepalive: spawn tokio interval at 60s, write ping frame to stream")
}

/// Run a command with a configurable timeout, killing the process on expiry.
///
/// Spec: gate-server/spec.md > "Proxy mode execution for gh commands"
/// Tasks: 4.11
/// Async — wraps execute_command with tokio::time::timeout.
pub async fn run_with_timeout(
    cmd: &str,
    args: &[String],
    env: &[(String, String)],
    timeout_secs: u64,
) -> Result<CommandOutput, GateError> {
    todo!("run_with_timeout: wrap execute_command in tokio::time::timeout, kill process on timeout")
}
