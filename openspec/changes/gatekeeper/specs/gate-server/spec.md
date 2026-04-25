## ADDED Requirements

### Requirement: Unix socket server for command authorization
The gate-server SHALL listen on a Unix domain socket at `/run/gate.sock` for command authorization requests from patched zsh instances. The server SHALL accept JSON requests containing `{command, args, cwd, pid}` and respond with one of: `{action: "allow"}`, `{action: "allow", env: {...}}`, `{action: "pending", approval_id: "..."}`, or `{action: "reject", reason: "..."}`.

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

### Requirement: GitHub App token generation
The gate-server SHALL hold GitHub App credentials (`GITHUB_APP_ID`, `GITHUB_APP_PRIVATE_KEY_PATH`) and generate short-lived installation access tokens on demand. Tokens SHALL be generated using the same RS256 JWT flow as the current `github-app-token` script. Tokens SHALL NOT be persisted to disk.

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
The gate-server SHALL expose a REST API on port 3000 for the web dashboard and integrations.

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

### Requirement: SQLite database
The gate-server SHALL use a SQLite database at `/opt/gate/db.sqlite` for all persistent state. The database SHALL contain tables: `default_permissions`, `grants`, `approval_requests`, `audit_log`, and `config`.

#### Scenario: Database initialization
- **WHEN** gate-server starts and the database file does not exist
- **THEN** gate-server creates the database with all required tables and seeds default permissions

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
