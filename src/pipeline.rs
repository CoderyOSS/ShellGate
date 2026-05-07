use std::collections::HashMap;

use crate::types::{AuditEntry, Config, GateError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StageVerdict {
    Allow,
    AllowAndNotify { message: NotifyMessage },
    Block { reason: String },
    BlockAndNotify { message: NotifyMessage },
    Pass,
    AllowAndPass { note: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotifyMessage {
    pub strategy: NotifyStrategy,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub reason: Option<String>,
    pub agenda_id: Option<String>,
    pub grant_id: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotifyStrategy {
    Intercept,
    Advisory,
    Request,
}

#[derive(Debug, Clone)]
pub struct DeliberationContext {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub pid: u32,
    pub agendas: Vec<AgendaSummary>,
    pub recent_history: Vec<AuditEntry>,
    pub config: PipelineConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgendaSummary {
    pub id: String,
    pub description: String,
    pub scope: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineConfig {
    pub flows: HashMap<String, Vec<String>>,
    pub stages: StagesConfig,
    pub bonsai: BonsaiConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StagesConfig {
    pub allow_list: AllowListStageConfig,
    pub catch_list: CatchListStageConfig,
    pub llm: LlmStageConfig,
    pub human: HumanStageConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllowListStageConfig {
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatchListStageConfig {
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmStageConfig {
    pub confidence_allow: f64,
    pub confidence_block: f64,
    pub max_context_commands: usize,
    pub warning_signs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HumanStageConfig {
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BonsaiConfig {
    pub model_path: String,
    pub model_size: String,
    pub max_tokens: usize,
    pub temperature: f64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        let mut flows = HashMap::new();
        flows.insert(
            "command_check".into(),
            vec![
                "allow_list".into(),
                "catch_list".into(),
                "llm".into(),
                "human".into(),
            ],
        );
        flows.insert(
            "mcp_request".into(),
            vec!["allow_list".into(), "llm".into(), "human".into()],
        );
        flows.insert(
            "interactive_bootstrap".into(),
            vec!["llm_questions".into(), "human_answers".into()],
        );

        Self {
            flows,
            stages: StagesConfig {
                allow_list: AllowListStageConfig {
                    sampling_rate: 0.05,
                },
                catch_list: CatchListStageConfig {
                    patterns: vec!["auth:*".into(), "rm -rf /".into()],
                },
                llm: LlmStageConfig {
                    confidence_allow: 0.7,
                    confidence_block: 0.3,
                    max_context_commands: 50,
                    warning_signs: vec![
                        "pip install from non-PyPI".into(),
                        "curl | bash or sh".into(),
                        "wget to /tmp".into(),
                        "chmod 777".into(),
                        "npm install from git URL not registry".into(),
                        "commands touching paths outside project dir".into(),
                        "base64 decode + execute".into(),
                    ],
                },
                human: HumanStageConfig {
                    timeout_seconds: 1800,
                },
            },
            bonsai: BonsaiConfig {
                model_path: "/opt/gate/models/bonsai-4b.gguf".into(),
                model_size: "4b".into(),
                max_tokens: 1024,
                temperature: 0.1,
            },
        }
    }
}

impl PipelineConfig {
    pub fn get_flow(&self, name: &str) -> Option<&Vec<String>> {
        self.flows.get(name)
    }
}

pub trait DeliberationStage: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError>;
}

pub struct Pipeline {
    stages: Vec<Box<dyn DeliberationStage>>,
}

impl Pipeline {
    pub fn new(stages: Vec<Box<dyn DeliberationStage>>) -> Self {
        Self { stages }
    }

    pub fn run(&self, ctx: &DeliberationContext) -> PipelineResult {
        let mut allowed = false;
        let mut notifications = Vec::new();

        for stage in &self.stages {
            match stage.evaluate(ctx) {
                Ok(StageVerdict::Allow) => {
                    return PipelineResult {
                        allowed: true,
                        notifications,
                        stage_name: stage.name().to_string(),
                        block_reason: None,
                    };
                }
                Ok(StageVerdict::AllowAndNotify { message }) => {
                    notifications.push(message);
                    return PipelineResult {
                        allowed: true,
                        notifications,
                        stage_name: stage.name().to_string(),
                        block_reason: None,
                    };
                }
                Ok(StageVerdict::Block { reason }) => {
                    return PipelineResult {
                        allowed: false,
                        notifications,
                        stage_name: stage.name().to_string(),
                        block_reason: Some(reason),
                    };
                }
                Ok(StageVerdict::BlockAndNotify { message }) => {
                    let reason = message.reason.clone();
                    notifications.push(message);
                    return PipelineResult {
                        allowed: false,
                        notifications,
                        stage_name: stage.name().to_string(),
                        block_reason: reason,
                    };
                }
                Ok(StageVerdict::Pass) => continue,
                Ok(StageVerdict::AllowAndPass { note }) => {
                    allowed = true;
                    tracing::debug!(stage = stage.name(), note = %note, "allow+pass");
                    continue;
                }
                Err(e) => {
                    tracing::error!(stage = stage.name(), error = %e, "stage error");
                    continue;
                }
            }
        }

        PipelineResult {
            allowed,
            notifications,
            stage_name: "pipeline_end".to_string(),
            block_reason: if allowed { None } else { Some("no stage permitted this command".into()) },
        }
    }
}

#[derive(Debug)]
pub struct PipelineResult {
    pub allowed: bool,
    pub notifications: Vec<NotifyMessage>,
    pub stage_name: String,
    pub block_reason: Option<String>,
}
