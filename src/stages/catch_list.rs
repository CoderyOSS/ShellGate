use crate::pipeline::{DeliberationContext, DeliberationStage, NotifyMessage, NotifyStrategy, StageVerdict};
use crate::types::GateError;

use globset::{GlobBuilder, GlobMatcher};
use std::sync::Arc;

pub struct CatchListStage {
    patterns: Vec<(String, GlobMatcher)>,
}

impl CatchListStage {
    pub fn new(patterns: &[String]) -> Result<Self, globset::Error> {
        let compiled: Vec<(String, GlobMatcher)> = patterns
            .iter()
            .filter_map(|p| {
                GlobBuilder::new(p)
                    .literal_separator(false)
                    .build()
                    .ok()
                    .map(|g| (p.clone(), g.compile_matcher()))
            })
            .collect();
        Ok(Self { patterns: compiled })
    }
}

impl DeliberationStage for CatchListStage {
    fn name(&self) -> &str {
        "catch_list"
    }

    fn evaluate(&self, ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
        let full_cmd = if ctx.args.is_empty() {
            ctx.command.clone()
        } else {
            format!("{} {}", ctx.command, ctx.args.join(" "))
        };

        for (pattern, matcher) in &self.patterns {
            if matcher.is_match(&full_cmd) || matcher.is_match(&ctx.command) {
                return Ok(StageVerdict::BlockAndNotify {
                    message: NotifyMessage {
                        strategy: NotifyStrategy::Intercept,
                        command: ctx.command.clone(),
                        args: ctx.args.clone(),
                        cwd: ctx.cwd.clone(),
                        reason: Some(format!("matched catch pattern: {}", pattern)),
                        agenda_id: None,
                        grant_id: None,
                        confidence: None,
                    },
                });
            }
        }

        Ok(StageVerdict::Pass)
    }
}
