use crate::types::{CommandClass, GateError};

/// Classify a command name + args into a structured action type.
///
/// Spec: gate-server/spec.md > "Command classification"
/// Tasks: 4.1
/// Pure function — no async, no DB access.
pub fn classify_command(command: &str, args: &[String]) -> CommandClass {
    todo!("classify_command: match command name (gh/git/curl), parse subcommand, determine action type (gh:pr:create, git:push, git:local, api:read, api:write, unknown)")
}

/// Extract repo from command args (--repo flag for gh, remote resolution for git, URL for clone).
///
/// Spec: gate-server/spec.md > "Command classification"
/// Tasks: 4.2
/// Pure function — parses args, may read .git/config for git remote.
pub fn extract_repo(command: &str, args: &[String], cwd: &str) -> Result<String, GateError> {
    todo!("extract_repo: for gh parse --repo flag, for git parse remote from .git/config or clone URL, for curl parse github.com URL")
}

/// Match a glob pattern against text using globset.
///
/// Spec: gate-permissions/spec.md > "Command pattern matching"
/// Tasks: 4.2
/// Pure function — glob pattern matching.
pub fn match_glob_pattern(pattern: &str, text: &str) -> bool {
    todo!("match_glob_pattern: compile globset::GlobMatcher from pattern, match against text")
}
