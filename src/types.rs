use std::collections::HashMap;

pub type GateError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Decision {
    Allow,
    AllowWithEnv(HashMap<String, String>),
    NeedApproval { approval_id: String },
    Reject { reason: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CommandClass {
    Gh { subcommand: String },
    Git { subcommand: String },
    GitLocal,
    ApiRead,
    ApiWrite,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub pid: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateResponse {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Grant {
    pub id: String,
    pub action: String,
    pub repo_pattern: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<u64>,
    pub use_count: u64,
    pub reason: String,
    pub created_by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub action: String,
    pub repo: String,
    pub status: ApprovalStatus,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: String,
    pub command: String,
    pub args: String,
    pub action: String,
    pub repo: String,
    pub granted_by: String,
    pub exit_code: Option<i32>,
    pub agent_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultPermission {
    pub action: String,
    pub state: PermissionState,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PermissionState {
    On,
    Off,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub gate: GateConfig,
    pub github: GitHubConfig,
    pub telegram: TelegramConfig,
    pub mcp: McpConfig,
    pub web: WebConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateConfig {
    pub socket_path: String,
    pub db_path: String,
    pub audit_ttl_secs: u64,
    pub rest_port: u16,
    pub rest_host: String,
    pub pending_queue_max: u32,
    pub allowed_uids: Vec<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubConfig {
    pub app_id: u64,
    pub app_key_path: String,
    pub installation_id: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpConfig {
    pub fifo_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebConfig {
    pub dist_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SseEvent {
    ApprovalNew { id: String },
    ApprovalResolved { id: String, status: String },
    GrantCreated { id: String },
    GrantExpired { id: String },
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

#[derive(Debug, Clone)]
pub struct OutputChunk {
    pub stream: OutputStream,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_path: String,
    pub config: Config,
    pub pending: std::sync::Arc<tokio::sync::RwLock<HashMap<String, tokio::sync::oneshot::Sender<GateResponse>>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditQueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub action: Option<String>,
    pub search: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateGrantRequest {
    pub action: String,
    pub repo_pattern: String,
    pub ttl_secs: u64,
    pub max_uses: Option<u64>,
    pub reason: String,
}
