#!/bin/sh
set -e

VERSION="${VERSION:-0.1.0-alpha.1}"
RELEASE_BASE="https://github.com/anomalyco/ShellGate/releases/download/v${VERSION}"
INSTALL_DIR="/opt/gate"

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

echo "Installing ShellGate v${VERSION} (${TARGET})"

if command -v apt-get >/dev/null 2>&1; then
    apt-get update -qq
    apt-get install -y -qq libseccomp2
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache libseccomp
elif command -v yum >/dev/null 2>&1; then
    yum install -y libseccomp
fi

mkdir -p "${INSTALL_DIR}"

curl -fsSL "${RELEASE_BASE}/gate-server-${VERSION}-${TARGET}.tar.gz" \
    | tar xz -C "${INSTALL_DIR}"
curl -fsSL "${RELEASE_BASE}/sgsh-connect-${VERSION}-${TARGET}.tar.gz" \
    | tar xz -C "${INSTALL_DIR}"

chmod 755 "${INSTALL_DIR}/gate-server"
chmod 755 "${INSTALL_DIR}/sgsh-connect"

SHELL_ENGINE="${SHELLENGINE}"
if [ -z "${SHELL_ENGINE}" ] && [ -f /bin/bash ]; then
    SHELL_ENGINE="/bin/bash"
elif [ -z "${SHELL_ENGINE}" ] && [ -f /bin/sh ]; then
    SHELL_ENGINE="/bin/sh"
fi

if [ -n "${SHELL_ENGINE}" ] && [ ! -f "${INSTALL_DIR}/shell-engine" ]; then
    cp "${SHELL_ENGINE}" "${INSTALL_DIR}/shell-engine"
    chmod 700 "${INSTALL_DIR}/shell-engine"
fi

if [ ! -L /bin/sh ] || [ "$(readlink /bin/sh 2>/dev/null)" != "${INSTALL_DIR}/sgsh-connect" ]; then
    if [ -f /bin/sh ] && [ ! -L /bin/sh ]; then
        mv /bin/sh /bin/sh.orig 2>/dev/null || true
    fi
    ln -sf "${INSTALL_DIR}/sgsh-connect" /bin/sh
fi

if [ ! -L /bin/bash ] || [ "$(readlink /bin/bash 2>/dev/null)" != "${INSTALL_DIR}/sgsh-connect" ]; then
    if [ -f /bin/bash ] && [ ! -L /bin/bash ]; then
        mv /bin/bash /bin/bash.orig 2>/dev/null || true
    fi
    ln -sf "${INSTALL_DIR}/sgsh-connect" /bin/bash
fi

getent group gate >/dev/null || groupadd -r gate

cat > "${INSTALL_DIR}/config.toml" << CONFEOF
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
bot_token = "${TELEGRAMBOTTOKEN}"
chat_id = ${TELEGRAMCHATID:-0}

[mcp]
fifo_path = "/tmp/gate-mcp.fifo"

[web]
dist_path = "/opt/gate/web/dist"

[pipeline.llm]
model_name = "${LLMMODEL:-deepseek-chat}"
api_url = "${LLMAPIURL:-https://api.deepseek.com/v1/chat/completions}"
api_key = "${LLMAPIKEY}"
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

chmod 640 "${INSTALL_DIR}/config.toml" 2>/dev/null || true

echo "ShellGate v${VERSION} installed"
