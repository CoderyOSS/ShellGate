use crate::bonsai::BonsaiModel;
use crate::derived_grants::{self, NewDerivedGrant};
use crate::pipeline::{
    AgendaSummary, DeliberationContext, DeliberationStage, NotifyMessage, NotifyStrategy,
    StageVerdict,
};
use crate::prompts;
use crate::types::GateError;

use std::sync::Arc;

pub struct LlmStage {
    model: Arc<BonsaiModel>,
    db_path: String,
}

impl LlmStage {
    pub fn new(model: Arc<BonsaiModel>, db_path: String) -> Self {
        Self { model, db_path }
    }
}

impl DeliberationStage for LlmStage {
    fn name(&self) -> &str {
        "llm"
    }

    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
        if !self.model.is_available() {
            tracing::debug!("bonsai model not available, passing to next stage");
            return Ok(StageVerdict::Pass);
        }

        let recent_commands: Vec<String> = ctx
            .recent_history
            .iter()
            .take(ctx.config.stages.llm.max_context_commands)
            .map(|e| format!("{} {}", e.command, e.args))
            .collect();

        let prompt = prompts::inline_deliberation_prompt(
            &ctx.command,
            &ctx.args,
            &ctx.cwd,
            &ctx.agendas,
            &recent_commands,
            &ctx.config.stages.llm.warning_signs,
        );

        let raw = match self.model.infer(&prompt) {
            Ok(output) => output,
            Err(e) => {
                tracing::error!(error = %e, "bonsai inference failed");
                return Ok(StageVerdict::Pass);
            }
        };

        let parsed = match prompts::parse_deliberation(&raw) {
            Some(p) => p,
            None => {
                tracing::warn!(output = %raw, "failed to parse LLM deliberation output");
                return Ok(StageVerdict::Pass);
            }
        };

        tracing::info!(
            decision = %parsed.decision,
            confidence = parsed.confidence,
            reason = %parsed.reason,
            "LLM deliberation result"
        );

        let conf = parsed.confidence;
        let thresholds = &ctx.config.stages.llm;

        match parsed.decision.as_str() {
            "ALLOW" if conf >= thresholds.confidence_allow => {
                Ok(StageVerdict::AllowAndNotify {
                    message: NotifyMessage {
                        strategy: NotifyStrategy::Advisory,
                        command: ctx.command.clone(),
                        args: ctx.args.clone(),
                        cwd: ctx.cwd.clone(),
                        reason: Some(parsed.reason),
                        agenda_id: None,
                        grant_id: None,
                        confidence: Some(conf),
                    },
                })
            }
            "BLOCK" if conf >= thresholds.confidence_block => {
                Ok(StageVerdict::BlockAndNotify {
                    message: NotifyMessage {
                        strategy: NotifyStrategy::Intercept,
                        command: ctx.command.clone(),
                        args: ctx.args.clone(),
                        cwd: ctx.cwd.clone(),
                        reason: Some(parsed.reason),
                        agenda_id: None,
                        grant_id: None,
                        confidence: Some(conf),
                    },
                })
            }
            _ => {
                tracing::info!(
                    confidence = conf,
                    "LLM uncertain, passing to human"
                );
                Ok(StageVerdict::Pass)
            }
        }
    }
}

pub fn generate_rules_for_agenda(
    model: &BonsaiModel,
    conn: &rusqlite::Connection,
    agenda_id: &str,
    description: &str,
    scope: Option<&str>,
) -> Result<usize, GateError> {
    if !model.is_available() {
        return Ok(0);
    }

    let prompt = prompts::batch_rule_prompt(description, scope);
    let raw = model.infer(&prompt)?;

    let rules = match prompts::parse_rules(&raw) {
        Some(r) => r,
        None => {
            tracing::warn!(output = %raw, "failed to parse batch rule output");
            return Ok(0);
        }
    };

    let new_grants: Vec<NewDerivedGrant> = rules
        .into_iter()
        .map(|r| NewDerivedGrant {
            command_pattern: r.command_pattern,
            args_pattern: r.args_pattern,
            path_pattern: r.path_pattern,
            notification: r.notification,
            reason: r.reason,
            confidence: r.confidence,
        })
        .collect();

    let count = new_grants.len();
    derived_grants::create_derived_grants(conn, agenda_id, &new_grants)?;

    tracing::info!(agenda_id = %agenda_id, rules = count, "generated derived grants from agenda");
    Ok(count)
}
