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
            let config = config::load_config("/opt/gate/config.toml").expect("failed to load config");
            mcp::run_mcp_server(&config.gate.db_path)
                .await
                .expect("MCP server failed");
        });
    } else {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            let config_path = std::env::var("GATE_CONFIG")
                .unwrap_or_else(|_| "/opt/gate/config.toml".to_string());
            let config = config::load_config(&config_path).expect("failed to load config");
            server::run_server(config)
                .await
                .expect("server failed");
        });
    }
}
