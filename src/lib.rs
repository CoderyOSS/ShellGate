pub mod agenda;
pub mod bootstrap;
pub mod bonsai;
pub mod derived_grants;
pub mod pipeline;
pub mod prompts;
pub mod stages;
pub mod types;

#[cfg(test)]
mod tests {
    use super::pipeline::*;
    use super::types::GateError;

    struct MockAllowStage;
    impl DeliberationStage for MockAllowStage {
        fn name(&self) -> &str { "mock_allow" }
        fn evaluate(&self, _ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
            Ok(StageVerdict::Allow)
        }
    }

    struct MockBlockStage;
    impl DeliberationStage for MockBlockStage {
        fn name(&self) -> &str { "mock_block" }
        fn evaluate(&self, _ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
            Ok(StageVerdict::Block { reason: "blocked by mock".into() })
        }
    }

    struct MockPassStage;
    impl DeliberationStage for MockPassStage {
        fn name(&self) -> &str { "mock_pass" }
        fn evaluate(&self, _ctx: &DeliberationContext) -> Result<StageVerdict, GateError> {
            Ok(StageVerdict::Pass)
        }
    }

    fn test_ctx() -> DeliberationContext {
        DeliberationContext {
            command: "npm".into(),
            args: vec!["install".into()],
            cwd: "/home/gem/projects/test".into(),
            pid: 1234,
            agendas: vec![],
            recent_history: vec![],
            config: PipelineConfig::default(),
        }
    }

    #[test]
    fn pipeline_allow_stops_early() {
        let pipeline = Pipeline::new(vec![
            Box::new(MockAllowStage),
            Box::new(MockBlockStage),
        ]);
        let result = pipeline.run(&test_ctx());
        assert!(result.allowed);
        assert_eq!(result.stage_name, "mock_allow");
    }

    #[test]
    fn pipeline_block_stops_early() {
        let pipeline = Pipeline::new(vec![
            Box::new(MockPassStage),
            Box::new(MockBlockStage),
            Box::new(MockAllowStage),
        ]);
        let result = pipeline.run(&test_ctx());
        assert!(!result.allowed);
        assert_eq!(result.stage_name, "mock_block");
    }

    #[test]
    fn pipeline_all_pass_falls_through() {
        let pipeline = Pipeline::new(vec![
            Box::new(MockPassStage),
            Box::new(MockPassStage),
        ]);
        let result = pipeline.run(&test_ctx());
        assert!(!result.allowed);
        assert_eq!(result.stage_name, "pipeline_end");
    }

    #[test]
    fn catch_list_matches() {
        let stage = super::stages::catch_list::CatchListStage::new(&[
            "rm -rf *".into(),
            "auth:*".into(),
        ]).unwrap();

        let mut ctx = test_ctx();
        ctx.command = "rm".into();
        ctx.args = vec!["-rf".into(), "/".into()];
        let result = stage.evaluate(&ctx).unwrap();

        match result {
            StageVerdict::BlockAndNotify { .. } => {}
            _ => panic!("expected BlockAndNotify, got {:?}", result),
        }
    }

    #[test]
    fn catch_list_no_match() {
        let stage = super::stages::catch_list::CatchListStage::new(&[
            "rm -rf *".into(),
        ]).unwrap();

        let result = stage.evaluate(&test_ctx()).unwrap();
        assert!(matches!(result, StageVerdict::Pass));
    }

    #[test]
    fn parse_deliberation_allow() {
        let raw = "DECISION: ALLOW\nCONFIDENCE: 0.85\nREASON: command fits the agenda";
        let parsed = super::prompts::parse_deliberation(raw).unwrap();
        assert_eq!(parsed.decision, "ALLOW");
        assert!((parsed.confidence - 0.85).abs() < 0.01);
        assert_eq!(parsed.reason, "command fits the agenda");
    }

    #[test]
    fn parse_deliberation_block() {
        let raw = "DECISION: BLOCK\nCONFIDENCE: 0.92\nREASON: suspicious curl to unknown host";
        let parsed = super::prompts::parse_deliberation(raw).unwrap();
        assert_eq!(parsed.decision, "BLOCK");
    }

    #[test]
    fn parse_rules() {
        let raw = r#"```json
[
    {"command_pattern": "git", "args_pattern": "status *", "path_pattern": null, "notification": "silent", "reason": "read-only", "confidence": 0.95},
    {"command_pattern": "npm", "args_pattern": "install", "path_pattern": null, "notification": "advisory", "reason": "dep install", "confidence": 0.6}
]
```"#;
        let rules = super::prompts::parse_rules(raw).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].command_pattern, "git");
        assert_eq!(rules[1].notification, "advisory");
    }

    #[test]
    fn parse_questions() {
        let raw = r#"{"questions": [{"question": "What are you building?", "type": "text", "options": []}]}"#;
        let parsed = super::prompts::parse_questions(raw).unwrap();
        assert_eq!(parsed.questions.len(), 1);
        assert_eq!(parsed.questions[0].question, "What are you building?");
    }

    #[test]
    fn pipeline_config_defaults() {
        let config = PipelineConfig::default();
        let flow = config.get_flow("command_check").unwrap();
        assert_eq!(flow, &vec!["allow_list", "catch_list", "llm", "human"]);
    }
}
