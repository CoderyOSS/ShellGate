mod agenda;
mod api;
mod approvals;
mod audit;
mod bonsai;
mod bootstrap;
mod classifier;
mod config;
mod derived_grants;
mod grants;
mod handler;
mod mcp;
mod permissions;
mod pipeline;
mod prompts;
mod proxy;
mod protocol;
mod schema;
mod server;
mod stages;
mod telegram;
mod tokens;
mod types;
mod watcher;

fn main() {
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
