use crate::pipeline::PipelineConfig;
use crate::types::{Config, GateConfig, GateError, GitHubConfig, McpConfig, TelegramConfig, WebConfig};

pub fn load_config(path: &str) -> Result<Config, GateError> {
    let content = std::fs::read_to_string(path)?;
    let mut config: Config = toml::from_str(&content)?;
    apply_env_overrides(&mut config);
    Ok(config)
}

pub fn default_config() -> Config {
    Config {
        gate: GateConfig {
            socket_path: "/run/gate.sock".into(),
            db_path: "/opt/gate/gate.db".into(),
            audit_ttl_secs: 86400 * 90,
            rest_port: 0,
            rest_host: "127.0.0.1".into(),
            pending_queue_max: 100,
            allowed_uids: vec![],
        },
        github: GitHubConfig {
            app_id: 0,
            app_key_path: String::new(),
            installation_id: 0,
        },
        telegram: TelegramConfig {
            bot_token: String::new(),
            chat_id: 0,
        },
        mcp: McpConfig {
            fifo_path: "/tmp/gate-mcp.fifo".into(),
        },
        web: WebConfig {
            dist_path: "/opt/gate/web/dist".into(),
        },
        pipeline: PipelineConfig::default(),
    }
}

pub fn apply_env_overrides(config: &mut Config) {
    if let Ok(v) = std::env::var("GATE_SOCKET_PATH") {
        config.gate.socket_path = v;
    }
    if let Ok(v) = std::env::var("GATE_DB_PATH") {
        config.gate.db_path = v;
    }
    if let Ok(v) = std::env::var("GATE_AUDIT_TTL_SECS") {
        if let Ok(n) = v.parse() {
            config.gate.audit_ttl_secs = n;
        }
    }
    if let Ok(v) = std::env::var("GATE_REST_PORT") {
        if let Ok(n) = v.parse() {
            config.gate.rest_port = n;
        }
    }
    if let Ok(v) = std::env::var("GATE_REST_HOST") {
        config.gate.rest_host = v;
    }
    if let Ok(v) = std::env::var("GATE_LLM_API_KEY") {
        config.pipeline.llm.api_key = v;
    }
    if let Ok(v) = std::env::var("GATE_LLM_MODEL") {
        config.pipeline.llm.model_name = v;
    }
    if let Ok(v) = std::env::var("GATE_LLM_API_URL") {
        config.pipeline.llm.api_url = v;
    }
    if let Ok(v) = std::env::var("GATE_LLM_MAX_TOKENS") {
        if let Ok(n) = v.parse() {
            config.pipeline.llm.max_tokens = n;
        }
    }
    if let Ok(v) = std::env::var("GATE_LLM_TEMPERATURE") {
        if let Ok(n) = v.parse() {
            config.pipeline.llm.temperature = n;
        }
    }
    if let Ok(v) = std::env::var("GATE_TELEGRAM_BOT_TOKEN") {
        config.telegram.bot_token = v;
    }
    if let Ok(v) = std::env::var("GATE_TELEGRAM_CHAT_ID") {
        if let Ok(n) = v.parse() {
            config.telegram.chat_id = n;
        }
    }

    tracing::info!(
        socket = %config.gate.socket_path,
        db = %config.gate.db_path,
        model = %config.pipeline.llm.model_name,
        "config loaded with env overrides"
    );
}

pub async fn start_config_watcher(config_path: String) -> Result<(), GateError> {
    use inotify::{Inotify, WatchMask};

    let mut inotify = Inotify::init()?;
    inotify
        .watches()
        .add(&config_path, WatchMask::MODIFY | WatchMask::CLOSE_WRITE)?;

    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            let events = match inotify.read_events(&mut buffer) {
                Ok(events) => events,
                Err(e) => {
                    tracing::error!(error = %e, "inotify read error, config watcher stopped");
                    return;
                }
            };

            if events.count() > 0 {
                tracing::info!(path = %config_path, "config file changed, reload triggered");
            }
        }
    });

    Ok(())
}
