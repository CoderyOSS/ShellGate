use crate::derived_grants;
use crate::pipeline::{
    DeliberationContext, DeliberationStage, NotifyMessage, NotifyStrategy, StageVerdict,
};
use crate::types::GateError;

pub struct AllowListStage {
    sampling_rate: f64,
    db_path: String,
}

impl AllowListStage {
    pub fn new(sampling_rate: f64, db_path: String) -> Self {
        Self { sampling_rate, db_path }
    }
}

impl DeliberationStage for AllowListStage {
    fn name(&self) -> &str {
        "allow_list"
    }

    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        let match_result = derived_grants::find_matching_derived_grant(
            &conn,
            &ctx.command,
            &ctx.args,
            &ctx.cwd,
        )?;

        let Some(m) = match_result else {
            return Ok(StageVerdict::Pass);
        };

        let should_sample = rand_sample(self.sampling_rate);

        if m.notification == "advisory" {
            return Ok(StageVerdict::AllowAndNotify {
                message: NotifyMessage {
                    strategy: NotifyStrategy::Advisory,
                    command: ctx.command.clone(),
                    args: ctx.args.clone(),
                    cwd: ctx.cwd.clone(),
                    reason: m.grant.reason.clone(),
                    agenda_id: Some(m.grant.agenda_id.clone()),
                    grant_id: Some(m.grant.id.clone()),
                    confidence: m.grant.confidence,
                },
            });
        }

        if should_sample {
            return Ok(StageVerdict::AllowAndPass {
                note: format!(
                    "sampled for LLM review (rate={})",
                    self.sampling_rate
                ),
            });
        }

        Ok(StageVerdict::Allow)
    }
}

fn rand_sample(rate: f64) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    ((nanos as f64) / u32::MAX as f64) < rate
}
