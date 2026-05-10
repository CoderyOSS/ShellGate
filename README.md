# ShellGate

AI agent shell command gateway — seccomp-based kernel enforcement.

Every `execve` call in the shell process tree is trapped by the Linux kernel.
The command passes through a multi-stage deliberation pipeline (allow list →
catch list → LLM → human approval) before execution proceeds or is blocked.

**Unbypassable.** Not a shell wrapper. Not an LD_PRELOAD. Kernel-level.

## Status: Pre-Alpha (v0.1.0-alpha.1)

**Not production ready.** Under active development.

| Component | Status |
|-----------|--------|
| Pipeline engine (allow_list, catch_list, llm, human) | Done |
| Agenda + derived grant CRUD | Done |
| Telegram notifications | Done |
| MCP protocol server | Done |
| OpenSpec watcher | Done |
| Seccomp kernel gateway | In progress |
| Full testing pass | Pending |
| Classifier, grants, approvals, audit, permissions, tokens | Stubbed |

### Known gaps
- ~10 modules stubbed with `todo!()`
- No integration tests for seccomp path
- Seccomp untested on aarch64
- No fuzzing or security audit

## Architecture

```
┌─ Container ───────────────────────────────────────────┐
│  gate-server (daemon)                                  │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────────┐  │
│  │ session  │  │ seccomp  │  │ pipeline             │  │
│  │ manager  │  │ notify   │  │ allow_list ▸ catch    │  │
│  │          │  │ epoll    │  │ ▸ llm ▸ human        │  │
│  └────┬─────┘  └────┬─────┘  └─────────┬───────────┘  │
│       │              │                  │              │
│  ┌────┴──────────────┴──────────────────┴──────────┐  │
│  │  Shell Session 1      Shell Session 2           │  │
│  │  bash (filtered)      bash (filtered)           │  │
│  │    └─ git, cargo...     └─ npm, pip...          │  │
│  │       ALL trapped         ALL trapped           │  │
│  └─────────────────────────────────────────────────┘  │
│                                                       │
│  /run/gate.sock  ← OpenCode connects for PTY sessions  │
│  MCP stdio       ← agents set agendas, check status    │
└───────────────────────────────────────────────────────┘
```

## Installation

### Dockerfile (one-liner)

```dockerfile
FROM debian:bookworm
RUN curl -fsSL https://releases.shellgate.dev/install.sh | bash
ENTRYPOINT ["/opt/gate/gate-server"]
```

### Runtime

```bash
docker run --security-opt seccomp=unconfined ...
```

### Dev Container Feature

```json
{
  "features": {
    "ghcr.io/anomalyco/shellgate:latest": {}
  },
  "securityOpt": ["seccomp=unconfined"]
}
```

## Configuration

### Config file (`/opt/gate/config.toml`)

```toml
[gate]
socket_path = "/run/gate.sock"
db_path = "/opt/gate/gate.db"
audit_ttl_secs = 7776000

[telegram]
bot_token = ""
chat_id = 0

[pipeline]
[pipeline.llm]
model_name = "deepseek-chat"
api_url = "https://api.deepseek.com/v1/chat/completions"
api_key = ""

[pipeline.stages.catch_list]
patterns = ["auth:*", "rm -rf /"]
```

### Environment variable overrides

| Variable | Config key |
|----------|-----------|
| `GATE_SOCKET_PATH` | `gate.socket_path` |
| `GATE_DB_PATH` | `gate.db_path` |
| `GATE_LLM_API_KEY` | `pipeline.llm.api_key` |
| `GATE_LLM_MODEL` | `pipeline.llm.model_name` |
| `GATE_LLM_API_URL` | `pipeline.llm.api_url` |
| `GATE_TELEGRAM_BOT_TOKEN` | `telegram.bot_token` |
| `GATE_TELEGRAM_CHAT_ID` | `telegram.chat_id` |
| `GATE_CONFIG` | Config file path |
| `GATE_SHELL_ENGINE` | Shell binary path (default: `/opt/gate/shell-engine`) |
| `GATE_PROJECTS_DIR` | Watched projects directory |

## Security Model

### Seccomp enforcement
- Filter traps `execve` and `execveat` in shell process trees
- Child processes inherit filter permanently
- `seccomp()` syscall blocked in children (prevents filter override)
- `prctl(PR_SET_NO_NEW_PRIVS)` prevents privilege escalation

### Access control
- Unix socket (`/run/gate.sock`) with `SO_PEERCRED` uid/gid checks
- `gate` group: agent access (set agenda, request shell)
- `gate-admin` group: human operator (approve/reject)

### Pipeline stages
1. **allow_list** — pre-approved derived grants from active agenda
2. **catch_list** — hard blocklist (glob patterns)
3. **llm** — AI judgment with confidence thresholds
4. **human** — Telegram notification with inline Approve/Reject buttons

## MCP Tools

Exposed to coding agents via MCP stdio:

| Tool | Purpose |
|------|---------|
| `set_agenda` | Declare current task |
| `request_pre_approval` | Request pre-approval for planned actions |
| `get_approval_status` | Check approval state |
| `list_grants` | View active allow grants |
| `explain_blocked` | See why a command was blocked |

## Build

```bash
# Default build (with seccomp)
cargo build --release

# Requires libseccomp-dev
apt install -y libseccomp-dev

# Build without seccomp (legacy voluntary gateway mode)
cargo build --release --no-default-features
```

## Roadmap

| Version | Scope |
|---------|-------|
| `v0.1.0-alpha.N` | Seccomp gateway, session manager, sgsh-connect, packaging |
| `v0.1.0-beta.1` | Testing pass, stub resolution, coverage ≥80% |
| `v0.1.0` | First stable release |

## License

MIT OR Apache-2.0
