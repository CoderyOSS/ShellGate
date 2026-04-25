## 1. Project Scaffolding

- [ ] 1.1 Initialize Rust workspace with `gate-server` binary crate (tokio, axum, rusqlite, serde, serde_json, teloxide, tower)
- [ ] 1.2 Create config module: parse TOML config file for Telegram bot token, chat ID, GitHub App ID, GitHub App key path, socket path, DB path, audit TTL
- [ ] 1.3 Create SQLite schema module: `default_permissions`, `grants`, `approval_requests`, `audit_log`, `config` tables with migration support
- [ ] 1.4 Seed default permissions on first run (reads ON/indefinite, writes OFF, auth:* blocked)

## 2. Permission Engine

- [ ] 2.1 Implement command classifier: parse command name + args into structured action (gh:pr:create, git:push, git:local, api:read, api:write, unknown)
- [ ] 2.2 Implement repo extractor: parse `--repo` flag from gh/git args, glob matching for repo patterns
- [ ] 2.3 Implement unified permission check: blocked list → active grants → default state → needs_approval
- [ ] 2.4 Implement grant CRUD: create, read, list, revoke grants with TTL, max_uses, repo pattern matching
- [ ] 2.5 Implement approval request lifecycle: create pending, approve (resolve waiting socket), reject, expire (background task)
- [ ] 2.6 Unit tests for permission engine: default ON/OFF, grant override, grant expiry, repo glob matching, max_uses, blocked list

## 3. Unix Socket Server

- [ ] 3.1 Implement Unix socket listener at `/run/gate.sock` with tokio
- [ ] 3.2 Define wire protocol: JSON length-prefixed frames for request (command, args, cwd, pid) and response (action, env, approval_id, reason)
- [ ] 3.3 Implement request handler: classify command → permission check → respond with allow/allow+env/pending/reject
- [ ] 3.4 Implement pending response flow: store pending channel per approval_id, block on channel, resolve when user approves/rejects
- [ ] 3.5 Implement token generation: GitHub App JWT → installation token flow, return in response env field
- [ ] 3.6 Connection lifecycle: clean up on disconnect, handle multiple concurrent clients

## 4. Proxy Mode Execution

- [ ] 4.1 Implement host-side command executor: spawn `gh.real` with args and GH_TOKEN, capture stdout/stderr/exit_code
- [ ] 4.2 Implement output streaming: stream stdout/stderr chunks back over the unix socket to the patched zsh
- [ ] 4.3 Implement socket keepalive: send ping frames every 60s during long-running commands
- [ ] 4.4 Handle command timeout: configurable max execution time (default 30 min), kill process on timeout

## 5. REST API

- [ ] 5.1 Implement REST API with axum on port 3000: `/api/approvals`, `/api/grants`, `/api/audit`, `/api/permissions`, `/api/config`
- [ ] 5.2 Implement approval endpoints: GET list (with status filter), POST approve, POST reject
- [ ] 5.3 Implement grant endpoints: POST create, GET list, DELETE revoke
- [ ] 5.4 Implement audit log endpoint: GET with pagination, action filter, text search, date range
- [ ] 5.5 Implement permission config endpoints: GET defaults, PUT update default, POST reset-to-defaults
- [ ] 5.6 Implement SSE endpoint for real-time approval updates (`/api/events`)

## 6. Audit Logger

- [ ] 6.1 Implement audit logging: insert row for every command with timestamp, action, repo, command_args, granted_by (grant_id, default, or approval_id), exit_code, agent_id
- [ ] 6.2 Implement TTL cleanup: background tokio task deletes expired audit_log rows every 5 minutes
- [ ] 6.3 Log all approval decisions: grant creation, approval approve/reject, permission config changes

## 7. Telegram Bot

- [ ] 7.1 Initialize teloxide bot with token from config
- [ ] 7.2 Implement approval notification: send formatted message with inline Approve/Reject buttons when approval request created
- [ ] 7.3 Implement callback handler: process Approve/Reject button taps, update message status, resolve waiting socket
- [ ] 7.4 Implement grant commands: `/grant <action> <repo> <ttl> <reason>`, `/grants` (list), `/revoke <id>`
- [ ] 7.5 Handle bot startup failures gracefully (log warning, continue without Telegram)

## 8. Patched zsh

- [ ] 8.1 Create zsh patch: add gate hook in `Src/exec.c` before fork for external commands, calling a gate_check function that connects to `/run/gate.sock`
- [ ] 8.2 Implement gate_check in new `Src/gate.c`: serialize command + args to JSON, send to unix socket, parse response
- [ ] 8.3 Implement allow path: if response is allow, proceed with normal fork+execve
- [ ] 8.4 Implement allow+env path: if response includes env vars, set them in child process after fork, before execve
- [ ] 8.5 Implement pending path: poll socket until resolved, print "gate: waiting for approval..." to stderr
- [ ] 8.6 Implement reject path: print reason to stderr, skip execution, set exit code to 1
- [ ] 8.7 Implement local git fast-path: detect git subcommands in {add,commit,status,diff,log,branch,stash,checkout,merge,rebase,reset,tag} and skip socket round-trip
- [ ] 8.8 Implement proxy mode for gh: when command is `gh`, send full command to gate-server, stream output back
- [ ] 8.9 Create Dockerfile.zsh-patch: download zsh release, apply patch, build, produce binary artifact
- [ ] 8.10 Test patched zsh: verify allow, allow+env, pending, reject, local git, gh proxy paths

## 9. Web UI

- [ ] 9.1 Initialize Astro project with React integration and TypeScript
- [ ] 9.2 Create dashboard layout: sidebar navigation (Approvals, Grants, Audit, Permissions)
- [ ] 9.3 Build Approvals page: list pending requests with Approve/Reject buttons, SSE for real-time updates
- [ ] 9.4 Build Grants page: create grant form (action, repo, TTL, reason), active grants list with revoke
- [ ] 9.5 Build Audit page: log table with filtering by action, searching by text, pagination
- [ ] 9.6 Build Permissions page: toggle grid for default permissions, reset-to-defaults button
- [ ] 9.7 Implement API client: fetch wrapper for all REST endpoints
- [ ] 9.8 Build static site (`astro build`), configure gate-server to serve from `dist/` at `/`

## 10. MCP Server

- [ ] 10.1 Implement MCP protocol over stdio: initialize, tools/list, tools/call
- [ ] 10.2 Implement `request_pre_approval` tool: accept actions, repos, ttl, reason; create pre-approval request (itself needs human approval)
- [ ] 10.3 Implement `check_approval_status` tool: accept approval_id, return current status
- [ ] 10.4 Implement `list_grants` tool: return active grants
- [ ] 10.5 Implement `explain_blocked` tool: return permissions currently OFF, partial grant coverage
- [ ] 10.6 Add `--mcp` flag / `mcp` subcommand to gate-server binary

## 11. Integration and Deployment

- [ ] 11.1 Create Willow service definition: `services/gate.yml` with socket path, port, health check, supervisord config
- [ ] 11.2 Update `Dockerfile.sandbox`: install patched zsh as `/bin/zsh`, symlink `/bin/bash → /bin/zsh`, remove `github-app-token`, remove `github-push`, remove `gh` binary, add `/run/gate.sock` volume mount
- [ ] 11.3 Create host supervisord config for gate-server: auto-restart, logging
- [ ] 11.4 Move GitHub App credentials from container to host: update env vars and secret mount paths
- [ ] 11.5 Create GitHub Actions workflow: build gate-server, build patched zsh, build web UI, deploy to VPS
- [ ] 11.6 End-to-end test: verify full flow — agent runs `gh pr create`, Telegram notification, approve, command executes
- [ ] 11.7 Document rollback procedure: restore original bash, restore credentials, remove socket mount

## 12. zsh Upstream PR (post-launch)

- [ ] 12.1 Research zsh contribution guidelines and coding standards
- [ ] 12.2 Design `pre_exec` hookdef API: module registers handler, receives (command, args, env), returns (allow/allow+env/reject)
- [ ] 12.3 Implement hookdef in zsh source with documentation
- [ ] 12.4 Write tests for the hookdef in zsh test suite
- [ ] 12.5 Submit PR to zsh/zsh on GitHub
- [ ] 12.6 Convert patch to pure zsh module once PR is merged
