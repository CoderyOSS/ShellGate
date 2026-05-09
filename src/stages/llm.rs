use crate::derived_grants::{self, NewDerivedGrant};
use crate::llm_client::LlmClient;
use crate::pipeline::{
    DeliberationContext, DeliberationStage, NotifyMessage, NotifyStrategy,
    StageVerdict,
};
use crate::prompts;
use crate::types::GateError;

use std::sync::Arc;

pub struct LlmStage {
    model: Arc<LlmClient>,
    #[allow(dead_code)]
    db_path: String,
}

impl LlmStage {
    pub fn new(model: Arc<LlmClient>, db_path: String) -> Self {
        Self { model, db_path }
    }
}

impl DeliberationStage for LlmStage {
    fn name(&self) -> &str {
        "llm"
    }

    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
        if !self.model.is_available() {
            tracing::debug!("LLM client not available, passing to next stage");
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

        let prompt_clone = prompt;
        let model_clone = self.model.clone();
        let raw = match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new()
                .expect("failed to create LLM runtime");
            rt.block_on(model_clone.infer(&prompt_clone))
        }).join() {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                tracing::error!(error = %e, "LLM API call failed");
                return Ok(StageVerdict::Pass);
            }
            Err(_) => {
                tracing::error!("LLM thread panicked");
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
    model: &LlmClient,
    conn: &rusqlite::Connection,
    agenda_id: &str,
    description: &str,
    scope: Option<&str>,
) -> Result<usize, GateError> {
    if !model.is_available() {
        return Ok(0);
    }

    let prompt = prompts::batch_rule_prompt(description, scope);

    let model_for_thread = model.clone();
    let raw = match std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .expect("failed to create rule generation runtime");
        rt.block_on(model_for_thread.infer(&prompt))
    }).join() {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            tracing::error!(error = %e, "LLM API call failed in rule generation");
            return Ok(0);
        }
        Err(_) => {
            tracing::error!("rule generation thread panicked");
            return Ok(0);
        }
    };

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
