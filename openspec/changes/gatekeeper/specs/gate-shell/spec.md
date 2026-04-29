## ADDED Requirements

### Requirement: Pre-exec hook in zsh
The patched zsh SHALL include a pre-exec hook that fires before every external command execution (before fork+execve). The hook SHALL send the command name, arguments, current working directory, and process ID to gate-server via the unix socket at `/run/gate.sock` and wait for a response before proceeding.

#### Scenario: External command allowed
- **WHEN** zsh is about to execute an external command (e.g., `git push origin main`)
- **AND** gate-server responds with `{action: "allow"}`
- **THEN** zsh proceeds with normal fork+execve

#### Scenario: External command allowed with env injection
- **WHEN** zsh is about to execute `git push origin main`
- **AND** gate-server responds with `{action: "allow", env: {"GH_TOKEN": "ghs_..."}}`
- **THEN** zsh sets the specified environment variables in the child process (after fork, before execve)
- **AND** the parent process environment is NOT modified

#### Scenario: External command pending approval
- **WHEN** zsh is about to execute a command
- **AND** gate-server responds with `{action: "pending", approval_id: "uuid"}`
- **THEN** zsh blocks (polls the socket) until a resolution is received
- **AND** displays "gate: waiting for approval..." on stderr

#### Scenario: External command rejected
- **WHEN** zsh receives `{action: "reject", reason: "..."}`
- **THEN** zsh prints the reason to stderr
- **AND** does NOT execute the command
- **AND** sets the exit code to 1

#### Scenario: Local-only git commands bypass gate
- **WHEN** zsh is about to execute a git command with subcommand in: `add`, `commit`, `status`, `diff`, `log`, `branch`, `stash`, `checkout`, `merge`, `rebase`, `reset`, `tag`
- **THEN** zsh executes the command locally without contacting gate-server
- **AND** no socket round-trip occurs

### Requirement: Proxy mode for gh commands
The patched zsh SHALL NOT have a real `gh` binary installed. When the agent runs `gh` commands, the patched zsh SHALL send the full command to gate-server in proxy mode. Gate-server executes `gh.real` on the host and streams stdout/stderr back. The patched zsh SHALL stream this output to its own stdout/stderr and exit with the remote command's exit code.

#### Scenario: gh command execution
- **WHEN** agent runs `gh pr list --repo rm-rf-etc/Willow`
- **THEN** patched zsh sends the command to gate-server
- **AND** gate-server checks permission (gh:pr:read — default ON)
- **AND** gate-server executes `gh.real pr list --repo rm-rf-etc/Willow` on host
- **AND** output is streamed back to the agent

### Requirement: Bash symlink
The container SHALL symlink `/bin/bash` to `/bin/zsh` (the patched version). Zsh SHALL be started as `zsh --emulate bash` for scripts invoked via `/bin/bash` shebang. All tools that invoke `bash -c "..."` or `/bin/bash` SHALL transparently use the patched zsh.

#### Scenario: OpenCode runs bash command
- **WHEN** OpenCode execs `/bin/bash -c "gh pr create --title fix"`
- **THEN** the patched zsh receives the command
- **AND** the pre-exec hook fires and checks with gate-server

#### Scenario: Bash compatibility limitations
- **GIVEN** bash-specific features like `$FUNCNAME`, `type -t`, `coproc`, POSIX-style arithmetic `$((x++))` edge cases, and associative array syntax differ in zsh
- **WHEN** a script uses these features via `/bin/bash` shebang
- **THEN** the behavior may differ from native bash
- **AND** known incompatibilities SHALL be documented in `/etc/zsh/bash-compat-notes.txt`
- **AND** if critical incompatibilities are discovered during testing, fall back to: bash retained as `/bin/bash` with DEBUG trap + extdebug for gating; zsh remains default interactive shell

### Requirement: zsh patch maintained as .patch file
The zsh modifications SHALL be maintained as a patch file against official zsh release tarballs. The patch SHALL add a gate hook point in `Src/exec.c` before the fork for external command execution. The patch SHALL be less than 50 lines of C code.

#### Scenario: Applying patch to new zsh release
- **WHEN** a new zsh release is published
- **THEN** the patch file can be applied to the release tarball
- **AND** the patched zsh builds successfully

### Requirement: No credentials in container
The patched zsh container SHALL NOT contain any GitHub credentials — no `GH_TOKEN`, no `GITHUB_TOKEN`, no SSH keys for GitHub, no `.config/gh/` credentials, no `github-app-token` script, no `github-push` script.

#### Scenario: Agent attempts direct GitHub access
- **WHEN** agent runs `/usr/bin/git push origin main` (bypassing function overrides)
- **THEN** git fails with authentication error because no credentials exist in the environment

#### Scenario: Agent attempts curl to GitHub
- **WHEN** agent runs `curl https://api.github.com/repos/rm-rf-etc/Willow/pulls`
- **THEN** curl succeeds for public repos (no auth needed)
- **AND** curl fails with 401/404 for private repos (no auth available)

### Requirement: Gate-server failure fallback
The patched zsh SHALL implement configurable fail-open vs fail-closed behavior when gate-server is unreachable.

#### Scenario: Gate-server unreachable
- **WHEN** gate-server socket at `/run/gate.sock` is missing or unresponsive
- **THEN** patched zsh retries connection with 5s timeout, up to 3 retries
- **AND** if all retries fail, behavior depends on config:
  - **fail-closed (default)**: GitHub-related commands (git push, gh, etc.) are rejected with exit code 69; local commands (ls, cat, etc.) are allowed
  - **fail-open**: all commands proceed without credentials; a warning is logged
- **AND** the fallback mode is configurable via `/etc/gate/fallback` config option

### Requirement: Container startup dependency on gate-server
The patched zsh SHALL handle the case where gate-server starts after the sandbox container.

#### Scenario: Sandbox starts before gate-server
- **WHEN** the sandbox container starts before gate-server is ready
- **THEN** patched zsh retries connection to `/run/gate.sock` with exponential backoff (base 100ms, 2x multiplier, jitter) for up to 30 seconds
- **AND** if gate-server remains unreachable after 30s, local commands execute without interception
- **AND** GitHub commands print a clear error message and exit with code 1
