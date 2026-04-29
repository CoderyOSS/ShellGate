## Why

AI agents with unrestricted GitHub access can cause unintended damage — pushing to main, deleting repos, merging unreviewed PRs. The current setup gives agents a GitHub App token with broad permissions and no human oversight. We need a gate between agent intent and GitHub execution, with an approval workflow accessible on-the-go via messaging apps and a web dashboard.

## What Changes

- New **gate-server** Rust binary running on the host that holds all GitHub credentials, evaluates command-level permissions, and executes approved commands
- New **patched zsh** with a pre-exec hook that sends every external command to gate-server for allow/deny/env-inject decisions before fork+execve
- Unified **permission model** where every action is a toggle with a TTL — reads default to ON with indefinite TTL, writes default to OFF, pre-approvals turn permissions ON with a configurable TTL
- **Telegram bot** for push notifications with inline approve/reject buttons
- **Web dashboard** (Astro + React, static build) for managing approvals, pre-approvals, audit log, and permission configuration
- **SQLite audit log** with configurable TTL for every command executed through the gate
- **MCP server** exposing tools for agents to request pre-approvals and check approval status
- **BREAKING**: Sandbox container loses all direct GitHub credentials (no tokens, no SSH keys, no gh auth). All GitHub access routed through gate-server.

## Capabilities

### New Capabilities
- `gate-server`: Host-side Rust binary — unix socket server, permission engine, GitHub App token generation, command execution, audit logging, REST API
- `gate-shell`: Patched zsh with pre-exec hook that checks every external command against gate-server before execution
- `gate-permissions`: Unified permission model — default states, pre-approval grants with TTL, approval request workflow
- `gate-telegram`: Telegram bot integration — push notifications, inline approve/reject, pre-approval management
- `gate-web`: Astro + React web dashboard — pending approvals, pre-approval management, audit log viewer, permission config editor
- `gate-mcp`: MCP server tools — request_pre_approval, get_approval_status, list_grants, explain_blocked

### Modified Capabilities

## Impact

- **Willow repo**: New service definition for gate-server, modified Dockerfile.sandbox (install patched zsh, remove credentials, add unix socket volume mount), supervisor config for gate-server
- **Sandbox container**: Loses all GitHub credentials, gains patched zsh as default shell, bash symlinked to zsh, gate-server unix socket bind-mounted from host
- **Host**: gate-server runs as supervised process alongside caddy/tailscale/orchestrator, owns GitHub App credentials
- **Dependencies**: zsh source (for patch), Rust toolchain, SQLite, Telegram Bot API, Astro/Node for web UI build
