use crate::types::{GateError, GitHubConfig};

/// Generate a GitHub App installation access token via JWT flow.
///
/// Spec: gate-server/spec.md > "GitHub App token generation"
/// Tasks: 3.3
/// Async — creates JWT, calls GitHub API for installation token.
pub async fn generate_installation_token(_config: &GitHubConfig) -> Result<String, GateError> {
    todo!("generate_installation_token: load private key, create RS256 JWT with app_id, POST /app/installations/{{id}}/access_tokens")
}

/// Get the currently cached token, refreshing if needed.
///
/// Spec: gate-server/spec.md > "GitHub App token generation"
/// Tasks: 3.3
/// Async — returns cached token if still valid, otherwise triggers refresh.
pub async fn get_cached_token() -> Result<String, GateError> {
    todo!("get_cached_token: check cached token TTL, return if valid, else call generate_installation_token")
}

/// Background task to refresh the token at the 50-minute mark.
///
/// Spec: gate-server/spec.md > "GitHub App token generation"
/// Tasks: 3.3
/// Async — periodic refresh loop, 5-minute grace period on failure.
pub async fn refresh_token_background(_config: GitHubConfig) -> Result<(), GateError> {
    todo!("refresh_token_background: spawn tokio interval at 50min mark, call generate_installation_token, update cache, 5min grace on failure")
}
