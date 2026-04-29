## ADDED Requirements

### Requirement: Unix socket server for command authorization
The gate-server SHALL listen on a Unix domain socket at `/run/gate.sock` for command authorization requests from patched zsh instances. Communication SHALL use HTTP-over-Unix-socket (axum's Unix listener). The server SHALL accept POST requests to `/check` with JSON body `{command, args, cwd, pid}` and respond with one of: `{action: "allow"}`, `{action: "allow", env: {...}}`, `{action: "pending", approval_id: "..."}`, or `{action: "reject", reason: "..."}`. Content-Type: application/json. HTTP/1.1 persistent connections. Timeout: 30s per request, 503 on timeout.

#### Scenario: Allowed command without env modification
- **WHEN** patched zsh sends a request for `git add .`
- **AND** the permission check returns allow with no env injection needed
- **THEN** gate-server responds with `{action: "allow"}`

#### Scenario: Allowed command with token injection
- **WHEN** patched zsh sends a request for `git push origin main`
- **AND** the permission check returns allow with GitHub token needed
- **THEN** gate-server responds with `{action: "allow", env: {"GH_TOKEN": "ghs_..."}}`

#### Scenario: Command requires approval
- **WHEN** patched zsh sends a request for `gh pr create --repo rm-rf-etc/Willow --title "fix"`
- **AND** no active grant covers this action
- **AND** the default permission state is OFF
- **THEN** gate-server creates an approval request in the database
- **AND** responds with `{action: "pending", approval_id: "uuid"}`

#### Scenario: Command rejected
- **WHEN** the user rejects an approval request via Telegram or web UI
- **THEN** gate-server responds to the waiting patched zsh with `{action: "reject", reason: "..."}`
- **AND** the patched zsh prints the reason to stderr and exits with code 1

#### Scenario: Socket peer authentication
- **WHEN** a process connects to `/run/gate.sock`
- **THEN** gate-server verifies the connecting process UID using `SO_PEERCRED`
- **AND** only connections from UIDs in the configured `allowed_uids` list (default: `[1000]`) are accepted
- **AND** unexpected peers receive HTTP 403 and connection close
- **AND** credentials are checked on every `accept()`, not cached

### Requirement: GitHub App token generation
The gate-server SHALL hold GitHub App credentials (`GITHUB_APP_ID`, `GITHUB_APP_PRIVATE_KEY_PATH`) and generate short-lived installation access tokens on demand. Tokens SHALL be generated using the same RS256 JWT flow as the current `github-app-token` script. Tokens SHALL NOT be persisted to disk. Gate-server SHALL cache the token with TTL tracking. A background task SHALL refresh the token at the 50-minute mark (before 1-hour expiry). If refresh fails, the cached token serves requests for up to 5 additional minutes (grace period), after which the next request triggers synchronous regeneration with a 5-second timeout.

#### Scenario: Token generated for approved command
- **WHEN** a command is approved that requires GitHub credentials
- **THEN** gate-server generates a fresh GitHub App installation token
- **AND** returns it in the response env field
- **AND** the token has a maximum TTL of 1 hour

#### Scenario: Credentials not found
- **WHEN** gate-server starts and cannot find the GitHub App private key
- **THEN** gate-server logs an error and exits with non-zero status
- **AND** no commands requiring GitHub credentials can be approved

### Requirement: Command classification
The gate-server SHALL classify incoming commands into structured actions. Classification SHALL parse the command name and arguments to determine the action type (e.g., `gh pr create` → `gh:pr:create`, `git push` → `git:push`, `curl -X POST api.github.com` → `api:write`).

#### Scenario: gh subcommand classification
- **WHEN** command is `gh` with subcommand `pr create`
- **THEN** classified as `gh:pr:create` with repo extracted from `--repo` flag
- **AND** `gh --repo` flag overrides any git remote resolution

#### Scenario: git subcommand classification
- **WHEN** command is `git` with subcommand `push`
- **THEN** classified as `git:push`
- **AND** the repo is extracted by reading `cwd/.git/config` to resolve the remote alias to a URL
- **AND** the URL is parsed as `https://github.com/owner/repo.git` → `owner/repo`
- **AND** if `.git/config` is unreadable or remote not found, returns `pending` with reason "cannot determine target repo"

#### Scenario: git clone classification
- **WHEN** command is `git clone https://github.com/org/repo.git`
- **THEN** the repo is extracted directly from the URL argument (no `.git/config` needed)

#### Scenario: git subcommand classification
- **WHEN** command is `git` with subcommand `push`
- **THEN** classified as `git:push`

#### Scenario: git local-only command classification
- **WHEN** command is `git` with subcommand `add`, `commit`, `status`, `diff`, `log`, `branch`, `stash`, `checkout`, `merge`, `rebase`, or `reset`
- **THEN** classified as `git:local` and auto-allowed without socket round-trip (handled by patched zsh locally)

#### Scenario: curl to GitHub classification
- **WHEN** command is `curl` with arguments containing `github.com` or `api.github.com`
- **THEN** classified as `api:read` (GET/default) or `api:write` (-X POST/PUT/PATCH/DELETE)

#### Scenario: Unknown command classification
- **WHEN** the command does not match any known pattern
- **THEN** classified as `unknown` and auto-allowed (no gate check)

### Requirement: Proxy mode execution for gh commands
For `gh` CLI commands, gate-server SHALL execute the command on the host using the real `gh` binary and stream stdout/stderr back to the patched zsh. The container SHALL NOT have a real `gh` binary installed.

#### Scenario: gh command approved and executed
- **WHEN** a `gh` command is approved
- **THEN** gate-server executes `gh.real` on the host with the original args and `GH_TOKEN` in environment
- **AND** streams stdout and stderr back to the patched zsh over the unix socket
- **AND** returns the exit code

#### Scenario: Long-running gh command
- **WHEN** `gh run watch` is approved and runs for several minutes
- **THEN** gate-server keeps the socket connection open and streams output until the command completes
- **AND** socket keepalive pings are sent every 60 seconds

### Requirement: REST API for web UI and integrations
The gate-server SHALL expose a REST API on port 3000, bound to localhost only (127.0.0.1), for the web dashboard and integrations. All `/api/*` endpoints SHALL require authentication. The root path `/` and non-API paths SHALL serve the built web UI, also requiring authentication. The only unauthenticated endpoint SHALL be `GET /health`.

Authentication SHALL use two methods:
1. **Caddy reverse-proxy with Tailscale auth** (primary, for web dashboard access)
2. **X-API-Key header** (secondary, for programmatic access from authorized hosts)

The API key SHALL be configurable in the TOML config and stored as a SHA-256 hash in SQLite.

#### Scenario: Health check
- **WHEN** a GET request is made to `/health`
- **THEN** returns 200 OK with JSON `{"status":"ok","db_connected":true,"uptime_secs":123}`

#### Scenario: List pending approvals
- **WHEN** a GET request is made to `/api/approvals?status=pending`
- **THEN** returns a JSON array of pending approval requests with id, command, action, repo, timestamp

#### Scenario: Approve an approval request
- **WHEN** a POST request is made to `/api/approvals/{id}/approve`
- **THEN** the approval request status changes to approved
- **AND** the waiting patched zsh receives the approval response

#### Scenario: Reject an approval request
- **WHEN** a POST request is made to `/api/approvals/{id}/reject`
- **THEN** the approval request status changes to rejected
- **AND** the waiting patched zsh receives the rejection response

#### Scenario: Create a pre-approval grant
- **WHEN** a POST request is made to `/api/grants` with `{actions, repos, ttl, reason}`
- **THEN** a new grant is created in the database
- **AND** matching commands are auto-approved until the grant expires

#### Scenario: List grants
- **WHEN** a GET request is made to `/api/grants`
- **THEN** returns a JSON array of active grants with id, actions, repos, expires_at, use_count

#### Scenario: Revoke a grant
- **WHEN** a DELETE request is made to `/api/grants/{id}`
- **THEN** the grant is disabled immediately
- **AND** subsequent commands matching this grant require fresh approval

#### Scenario: View audit log
- **WHEN** a GET request is made to `/api/audit?limit=100`
- **THEN** returns a JSON array of audit log entries ordered by timestamp descending

#### Scenario: Concurrent approval prevented
- **WHEN** two simultaneous POST requests are made to `/api/approvals/{id}/approve`
- **THEN** the first request updates the row using `UPDATE ... WHERE status = 'pending'`
- **AND** if `rows_affected == 1`, the approval succeeds and the waiting socket receives the response
- **AND** the second request finds `rows_affected == 0` and returns HTTP 409 Conflict with `{"error":"request already resolved"}`
- **AND** Telegram callback queries also use the same atomic update pattern
- **AND** each approval request has a unique idempotency key to prevent duplicate processing

#### Scenario: Approval request rate limiting
- **WHEN** a container exceeds 10 pending approval requests
- **THEN** additional requests from that container return HTTP 429 Too Many Requests
- **AND** duplicate commands (same argv + cwd within 60s) return the existing request ID instead of creating new
- **AND** the pending queue max is configurable via `pending_queue_max` in TOML (default 10)

#### Scenario: Approval prompt flood protection
- **WHEN** a container exceeds 10 approval prompts per minute
- **THEN** exceeding prompts are queued and sent as a single batch summary to the operator
- **AND** the counter resets when the operator acts on the batch (approve or reject)

### Requirement: Config file
The gate-server SHALL read a TOML config file at `/opt/gate/config.toml` (overridable via `--config` flag). The config SHALL include the following sections:

```toml
[gate]
socket_path = "/run/gate.sock"
db_path = "/opt/gate/db.sqlite"
audit_ttl_secs = 2592000  # 30 days, 0 = indefinite
rest_port = 3000
rest_host = "127.0.0.1"
pending_queue_max = 10

[github]
app_id = 123456
app_key_path = "/opt/gate/github-app.pem"
installation_id = 789012

[telegram]
bot_token = ""  # empty = bot disabled
chat_id = 0

[mcp]
fifo_path = "/opt/gate/mcp-notify.fifo"

[web]
dist_path = "/opt/gate/dist"  # static web UI build output
```

Telegram `bot_token` MAY also be provided via the `TELEGRAM_BOT_TOKEN` environment variable (overrides config file, avoids token in TOML).

### Requirement: Graceful shutdown
The gate-server SHALL handle SIGTERM and SIGQUIT signals for controlled termination.

#### Scenario: Graceful shutdown
- **WHEN** gate-server receives SIGTERM (e.g., during blue/green deploy)
- **THEN** it stops accepting new connections (both unix socket and REST)
- **AND** drains in-flight requests with a 10-second timeout
- **AND** runs `PRAGMA wal_checkpoint` to finalize SQLite WAL
- **AND** exits with code 0

#### Scenario: Fast shutdown
- **WHEN** gate-server receives SIGQUIT
- **THEN** it exits immediately without draining
- **AND** exits with code 1

### Requirement: Credential storage
GitHub App private key and client credentials SHALL NOT be stored in the config file. They SHALL be loaded via one of:
1. Environment variable `GATE_GITHUB_APP_KEY` (base64-encoded PEM)
2. File at path from `GATE_GITHUB_APP_KEY_FILE` env var (0600 permissions enforced on read)
3. File at path from config `github.app_key_path` (0600 permissions verified at startup)

Startup SHALL fail with a clear error if the key is missing or has wrong permissions. No fallback to less secure alternatives.

### Requirement: Audit log retention
The audit log SHALL have configurable retention and disk usage limits to prevent unbounded growth.

#### Scenario: Audit log cleanup
- **WHEN** a background task runs (configurable interval, default daily)
- **THEN** rows in `audit_log` with `created_at` older than `audit_log_retention_days` (default 90) are deleted
- **AND** if more than 10% of rows were removed, `VACUUM` is executed to reclaim space
- **AND** if the database file exceeds `max_db_size_mb` (default 500), new commands are rejected with HTTP 507 Insufficient Storage

### Requirement: SQLite database
The gate-server SHALL use a SQLite database at `/opt/gate/db.sqlite` for all persistent state. The database SHALL be opened with `PRAGMA journal_mode=WAL` and `PRAGMA busy_timeout=5000` to enable concurrent readers and writer without `SQLITE_BUSY` errors. All timestamps SHALL be stored as ISO 8601 text in UTC. Daily WAL checkpoint + VACUUM SHALL be performed via a background task. The database SHALL contain tables: `default_permissions`, `grants`, `approval_requests`, `audit_log`, and `config`.

#### Scenario: Database initialization
- **WHEN** gate-server starts and the database file does not exist
- **THEN** gate-server creates the database with all required tables and seeds default permissions

#### Scenario: Schema migration
- **WHEN** gate-server starts and the database schema version does not match the expected version
- **THEN** gate-server checks the current version stored in the `config` table
- **AND** applies any pending sequential SQL migrations (embedded in the binary or at `/opt/gate/migrations/`)
- **AND** each migration runs in a transaction, recording the new version on success
- **AND** if a migration fails, gate-server exits with a clear error message
- **AND** rollback requires restoring the previous binary and database backup

#### Scenario: Audit log TTL cleanup
- **WHEN** a background task runs (configurable interval, default every 5 minutes)
- **THEN** rows in `audit_log` where `ttl_until` is in the past are deleted

### Requirement: Static web UI serving
The gate-server SHALL serve the static web UI files from the Astro+React build at the root path `/` on port 3000. The REST API SHALL be served at `/api/*` on the same port.

#### Scenario: Web UI served
- **WHEN** a GET request is made to `/` or any non-API path
- **THEN** gate-server serves the corresponding static file from the built web UI

### Requirement: Supervisord management
The gate-server SHALL run as a supervised process on the host via supervisord, with auto-restart on failure.

#### Scenario: gate-server crashes
- **WHEN** gate-server process exits unexpectedly
- **THEN** supervisord restarts it within seconds
- **AND** the unix socket becomes available again
