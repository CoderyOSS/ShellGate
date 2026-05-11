#!/bin/bash
DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cat > /tmp/gate-probes-config.toml <<'TOML'
[gate]
socket_path = "/tmp/gate-probes.sock"
db_path = "/tmp/gate-probes.db"
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
fifo_path = "/tmp/gate-mcp-probes.fifo"

[web]
dist_path = "/tmp/gate-web-probes"

[pipeline.llm]
model_name = "deepseek-chat"
api_url = "http://127.0.0.1:19876/v1/chat/completions"
api_key = "test"
max_tokens = 256
temperature = 0.1

[pipeline.stages.allow_list]
sampling_rate = 0.0

[pipeline.stages.catch_list]
patterns = ["rm -rf *", "auth:*", "curl * | *"]

[pipeline.stages.llm]
confidence_allow = 0.7
confidence_block = 0.3
max_context_commands = 50
warning_signs = []

[pipeline.stages.human]
timeout_seconds = 30

[pipeline.flows]
command_check = ["catch_list", "allow_list", "llm", "human"]
mcp_request = ["allow_list", "human"]
interactive_bootstrap = []
TOML
export GATE_CONFIG=/tmp/gate-probes-config.toml
exec "$DIR/target/debug/gate-server"
