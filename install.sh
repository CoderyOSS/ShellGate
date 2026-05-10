#!/bin/sh
set -e

VERSION="${SHELLGATE_VERSION:-0.1.0-alpha.1}"
INSTALL_DIR="/opt/gate"
RELEASE_BASE="https://github.com/anomalyco/ShellGate/releases/download/v${VERSION}"

UNAME_M=$(uname -m)
case "$UNAME_M" in
    x86_64)  ARCH="x86_64" ;;
    aarch64) ARCH="aarch64" ;;
    *)       echo "Unsupported architecture: $UNAME_M"; exit 1 ;;
esac

LIBC_TYPE="gnu"
if ldd /bin/sh 2>/dev/null | grep -q musl 2>/dev/null; then
    LIBC_TYPE="musl"
elif [ -f /etc/alpine-release ]; then
    LIBC_TYPE="musl"
fi

TARGET="${ARCH}-unknown-linux-${LIBC_TYPE}"

echo "ShellGate installer v${VERSION}"
echo "  arch:  ${ARCH}"
echo "  libc:  ${LIBC_TYPE}"
echo "  target: ${TARGET}"

if [ -f /etc/debian_version ]; then
    apt-get update -qq
    apt-get install -y -qq libseccomp2
elif [ -f /etc/alpine-release ]; then
    apk add --no-cache libseccomp
elif [ -f /etc/redhat-release ]; then
    yum install -y libseccomp
fi

mkdir -p "${INSTALL_DIR}"
mkdir -p "${INSTALL_DIR}/lib"

echo "Downloading gate-server..."
curl -fsSL "${RELEASE_BASE}/gate-server-${VERSION}-${TARGET}.tar.gz" \
    | tar xz -C "${INSTALL_DIR}"

echo "Downloading sgsh-connect..."
curl -fsSL "${RELEASE_BASE}/sgsh-connect-${VERSION}-${TARGET}.tar.gz" \
    | tar xz -C "${INSTALL_DIR}"

chmod 755 "${INSTALL_DIR}/gate-server"
chmod 755 "${INSTALL_DIR}/sgsh-connect"

if [ -f /bin/bash ] && [ ! -f "${INSTALL_DIR}/shell-engine" ]; then
    cp /bin/bash "${INSTALL_DIR}/shell-engine"
    chmod 700 "${INSTALL_DIR}/shell-engine"
    echo "Copied /bin/bash → ${INSTALL_DIR}/shell-engine"
elif [ ! -f "${INSTALL_DIR}/shell-engine" ]; then
    echo "WARNING: /bin/bash not found, shell-engine not installed"
    echo "  Set GATE_SHELL_ENGINE to your shell path"
fi

if [ ! -L /bin/sh ] || [ "$(readlink /bin/sh)" != "${INSTALL_DIR}/sgsh-connect" ]; then
    if [ -f /bin/sh ] && [ ! -L /bin/sh ]; then
        mv /bin/sh /bin/sh.orig 2>/dev/null || true
    fi
    ln -sf "${INSTALL_DIR}/sgsh-connect" /bin/sh
fi

if [ ! -L /bin/bash ] || [ "$(readlink /bin/bash)" != "${INSTALL_DIR}/sgsh-connect" ]; then
    if [ -f /bin/bash ] && [ ! -L /bin/bash ]; then
        mv /bin/bash /bin/bash.orig 2>/dev/null || true
    fi
    ln -sf "${INSTALL_DIR}/sgsh-connect" /bin/bash
fi

getent group gate >/dev/null || groupadd -r gate

if [ ! -f "${INSTALL_DIR}/config.toml" ]; then
    cat > "${INSTALL_DIR}/config.toml" << 'CONFEOF'
[gate]
socket_path = "/run/gate.sock"
db_path = "/opt/gate/gate.db"
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
fifo_path = "/tmp/gate-mcp.fifo"

[web]
dist_path = "/opt/gate/web/dist"

[pipeline.llm]
model_name = "deepseek-chat"
api_url = "https://api.deepseek.com/v1/chat/completions"
api_key = ""
max_tokens = 256
temperature = 0.1

[pipeline.stages.allow_list]
sampling_rate = 0.05

[pipeline.stages.catch_list]
patterns = ["auth:*", "rm -rf /"]

[pipeline.stages.llm]
confidence_allow = 0.7
confidence_block = 0.3
max_context_commands = 50
warning_signs = [
    "pip install from non-PyPI",
    "curl | bash or sh",
    "wget to /tmp",
    "chmod 777",
    "npm install from git URL not registry",
    "commands touching paths outside project dir",
    "base64 decode + execute",
]

[pipeline.stages.human]
timeout_seconds = 1800

[pipeline.flows]
command_check = ["allow_list", "catch_list", "llm", "human"]
mcp_request = ["allow_list", "llm", "human"]
interactive_bootstrap = ["llm_questions", "human_answers"]
CONFEOF
fi

chmod 640 "${INSTALL_DIR}/config.toml" 2>/dev/null || true

echo ""
echo "ShellGate v${VERSION} installed to ${INSTALL_DIR}"
echo ""
echo "  Binaries:  gate-server, sgsh-connect"
echo "  Config:    ${INSTALL_DIR}/config.toml"
echo "  Shell:     /bin/sh → ${INSTALL_DIR}/sgsh-connect"
echo ""
echo "  Set secrets via environment:"
echo "    GATE_LLM_API_KEY"
echo "    GATE_TELEGRAM_BOT_TOKEN"
echo "    GATE_TELEGRAM_CHAT_ID"
echo ""
echo "  Add to Dockerfile:"
echo "    ENTRYPOINT [\"/opt/gate/gate-server\"]"
echo ""
