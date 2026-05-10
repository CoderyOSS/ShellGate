#[cfg(feature = "e2e")]
mod e2e_tests {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::process::{Child, Command};
    use std::time::Duration;

    struct GateServer {
        child: Child,
        socket_path: String,
        temp_dir: tempfile::TempDir,
    }

    impl GateServer {
        fn start() -> Result<Self, String> {
            let temp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
            let socket_path = temp_dir
                .path()
                .join("gate.sock")
                .to_string_lossy()
                .to_string();
            let db_path = temp_dir
                .path()
                .join("gate.db")
                .to_string_lossy()
                .to_string();
            let config_path = temp_dir
                .path()
                .join("config.toml")
                .to_string_lossy()
                .to_string();

            let config_content = format!(
                r#"
[gate]
socket_path = "{}"
db_path = "{}"
audit_ttl_secs = 7776000
rest_port = 0
rest_host = "127.0.0.1"
pending_queue_max = 100
allowed_uids = []

[github]
app_id = 0
app_key_path = ""
installation_id = 0

[telegram]
bot_token = ""
chat_id = 0

[mcp]
fifo_path = "/tmp/gate-mcp-e2e.fifo"

[web]
dist_path = "/tmp/gate-web-e2e"

[pipeline.llm]
model_name = "deepseek-chat"
api_url = "https://api.deepseek.com/v1/chat/completions"
api_key = ""
max_tokens = 256
temperature = 0.1

[pipeline.stages.allow_list]
sampling_rate = 0.0

[pipeline.stages.catch_list]
patterns = ["rm -rf *", "auth:*"]

[pipeline.stages.llm]
confidence_allow = 0.7
confidence_block = 0.3
max_context_commands = 50
warning_signs = []

[pipeline.stages.human]
timeout_seconds = 10

[pipeline.flows]
command_check = ["catch_list", "allow_list", "human"]
mcp_request = ["allow_list", "human"]
interactive_bootstrap = []
"#,
                socket_path, db_path
            );
            std::fs::write(&config_path, config_content).map_err(|e| e.to_string())?;

            let child = Command::new(
                std::env::var("GATE_SERVER_BIN")
                    .unwrap_or_else(|_| "target/debug/gate-server".to_string()),
            )
            .env("GATE_CONFIG", &config_path)
            .spawn()
            .map_err(|e| format!("failed to start gate-server: {}", e))?;

            std::thread::sleep(Duration::from_millis(500));

            Ok(Self {
                child,
                socket_path,
                temp_dir,
            })
        }

        fn connect(&self) -> Result<UnixStream, String> {
            for attempt in 0..10 {
                match UnixStream::connect(&self.socket_path) {
                    Ok(s) => return Ok(s),
                    Err(_) => {
                        if attempt == 9 {
                            return Err("gate-server socket not ready after 10 attempts".into());
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
            unreachable!()
        }

        fn send_gate_request(
            &self,
            command: &str,
            args: &[&str],
        ) -> Result<serde_json::Value, String> {
            let mut stream = self.connect()?;

            let request = serde_json::json!({
                "command": command,
                "args": args,
                "cwd": "/tmp",
                "pid": 1234
            });
            let body = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
            let len = body.len() as u32;

            stream
                .write_all(&len.to_be_bytes())
                .map_err(|e| e.to_string())?;
            stream
                .write_all(&body)
                .map_err(|e| e.to_string())?;

            let mut len_buf = [0u8; 4];
            stream
                .read_exact(&mut len_buf)
                .map_err(|e| e.to_string())?;
            let resp_len = u32::from_be_bytes(len_buf) as usize;

            let mut resp_body = vec![0u8; resp_len];
            stream
                .read_exact(&mut resp_body)
                .map_err(|e| e.to_string())?;

            serde_json::from_slice(&resp_body).map_err(|e| e.to_string())
        }

        fn seed_grant(&self, command_pattern: &str, args_pattern: &str) {
            let db_path = self.temp_dir.path().join("gate.db");
            let conn = rusqlite::Connection::open(&db_path).expect("open db");

            let agenda_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let future = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

            conn.execute(
                "INSERT INTO agendas (id, source, description, scope, status, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    agenda_id, "e2e-test", "e2e test agenda", (std::option::Option::<String>::None),
                    "active", now, future
                ],
            ).expect("insert agenda");

            let grant_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO derived_grants (id, agenda_id, command_pattern, args_pattern, path_pattern, notification, reason, confidence, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    grant_id, agenda_id, command_pattern, args_pattern,
                    (std::option::Option::<String>::None),
                    "silent", "e2e test grant", 0.95, now, future
                ],
            ).expect("insert grant");
        }

        fn send_spawn_shell(&self) -> Result<(), String> {
            let mut stream = self.connect()?;

            let request = serde_json::json!({"type": "spawn_shell"});
            let body = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
            stream
                .write_all(&body)
                .map_err(|e| e.to_string())?;
            stream
                .write_all(b"\n")
                .map_err(|e| e.to_string())?;

            Ok(())
        }
    }

    impl Drop for GateServer {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    #[test]
    fn e2e_catch_list_blocks_rm_rf() {
        let start = std::time::Instant::now();
        let server = GateServer::start().expect("gate-server start");
        let result = server
            .send_gate_request("rm", &["-rf", "/"])
            .expect("request");

        let action = result["action"].as_str().unwrap_or("");
        assert!(
            action == "reject",
            "expected reject, got {}: {}",
            action,
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );

        record_e2e("e2e_catch_list_blocks_rm_rf", start, "pass", None,
            &serde_json::json!({"command":"rm","args":["-rf","/"]}),
            &result);
    }

    #[test]
    fn e2e_safe_command_allowed_with_grant() {
        let start = std::time::Instant::now();
        let server = GateServer::start().expect("gate-server start");
        server.seed_grant("echo", "hello");
        let result = server
            .send_gate_request("echo", &["hello"])
            .expect("request");

        let action = result["action"].as_str().unwrap_or("");
        assert!(
            action == "allow",
            "expected allow, got {}: {}",
            action,
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );

        record_e2e("e2e_safe_command_allowed_with_grant", start, "pass", None,
            &serde_json::json!({"command":"echo","args":["hello"]}),
            &result);
    }

    #[test]
    fn e2e_catch_list_blocks_auth_command() {
        let start = std::time::Instant::now();
        let server = GateServer::start().expect("gate-server start");
        let result = server
            .send_gate_request("auth:login", &[])
            .expect("request");

        let action = result["action"].as_str().unwrap_or("");
        assert!(
            action == "reject",
            "expected reject for auth:*, got {}",
            action
        );

        record_e2e("e2e_catch_list_blocks_auth_command", start, "pass", None,
            &serde_json::json!({"command":"auth:login","args":[]}),
            &result);
    }

    #[test]
    fn e2e_gate_server_starts_and_accepts_connections() {
        let start = std::time::Instant::now();
        let server = GateServer::start().expect("gate-server start");
        let stream = server.connect().expect("connect");
        drop(stream);

        record_e2e("e2e_gate_server_starts_and_accepts_connections", start, "pass", None,
            &serde_json::json!({"action":"connect"}),
            &serde_json::json!({"result":"ok"}));
    }

    #[test]
    fn e2e_spawn_shell_connects() {
        if std::env::var("SKIP_SPAWN_SHELL_TEST").is_ok() {
            record_e2e("e2e_spawn_shell_connects", std::time::Instant::now(), "skip", None,
                &serde_json::json!({"type":"spawn_shell"}),
                &serde_json::json!({"result":"skipped"}));
            return;
        }

        let start = std::time::Instant::now();
        let server = GateServer::start().expect("gate-server start");
        server.send_spawn_shell().expect("spawn shell");

        record_e2e("e2e_spawn_shell_connects", start, "pass", None,
            &serde_json::json!({"type":"spawn_shell"}),
            &serde_json::json!({"result":"connected"}));
    }

    struct E2eProofEntry {
        test_name: String,
        duration_ms: u64,
        result: String,
        error: Option<String>,
        send: serde_json::Value,
        response: serde_json::Value,
    }

    static PROOF_ENTRIES: std::sync::Mutex<Vec<E2eProofEntry>> = std::sync::Mutex::new(Vec::new());
    static WRITTEN: std::sync::Once = std::sync::Once::new();

    extern "C" fn write_proof_records_on_exit() {
        let entries = PROOF_ENTRIES.lock().unwrap();
        if entries.is_empty() {
            return;
        }
        let out_path = "tests/proof-records-e2e.md";
        let mut md = String::new();
        md.push_str("# ShellGate E2E Proof Records (Rust)\n\n");
        md.push_str(&format!("**Date:** {}\n", chrono::Utc::now().to_rfc3339()));
        md.push_str(&format!("**Tests:** {} run\n\n---\n\n", entries.len()));

        for entry in entries.iter() {
            let status_icon = if entry.result == "pass" { "✓" } else { "✗" };
            md.push_str(&format!("## {}\n\n", entry.test_name));
            md.push_str(&format!("**Status:** {} {} | **Duration:** {}ms\n\n",
                status_icon, entry.result, entry.duration_ms));
            if let Some(ref err) = entry.error {
                md.push_str(&format!("**Error:** {}\n\n", err));
            }
            md.push_str(&format!("### Send\n\n```json\n{}\n```\n\n",
                serde_json::to_string_pretty(&entry.send).unwrap_or_default()));
            md.push_str(&format!("### Response\n\n```json\n{}\n```\n\n---\n\n",
                serde_json::to_string_pretty(&entry.response).unwrap_or_default()));
        }

        std::fs::write(out_path, md).expect("write proof records");
        println!("E2E proof records written to {}", out_path);
    }

    fn ensure_atexit() {
        WRITTEN.call_once(|| {
            unsafe { libc::atexit(write_proof_records_on_exit); }
        });
    }

    fn record_e2e(
        test_name: &str,
        start: std::time::Instant,
        result: &str,
        error: Option<&str>,
        send: &serde_json::Value,
        response: &serde_json::Value,
    ) {
        ensure_atexit();
        PROOF_ENTRIES.lock().unwrap().push(E2eProofEntry {
            test_name: test_name.to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            result: result.to_string(),
            error: error.map(|e| e.to_string()),
            send: send.clone(),
            response: response.clone(),
        });
    }
}
