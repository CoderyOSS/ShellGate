use crate::pipeline::{
    DeliberationContext, DeliberationStage, NotifyMessage, NotifyStrategy, StageVerdict,
};
use crate::types::GateError;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HumanApprovalStage {
    pub timeout_secs: u64,
    pub pending: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<crate::types::GateResponse>>>>,
}

impl DeliberationStage for HumanApprovalStage {
    fn name(&self) -> &str {
        "human"
    }

    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
        let approval_id = uuid::Uuid::new_v4().to_string();

        Ok(StageVerdict::BlockAndNotify {
            message: NotifyMessage {
                strategy: NotifyStrategy::Intercept,
                command: ctx.command.clone(),
                args: ctx.args.clone(),
                cwd: ctx.cwd.clone(),
                reason: Some("no grant or LLM approval for this command".into()),
                agenda_id: None,
                grant_id: None,
                confidence: None,
            },
        })
    }
}
