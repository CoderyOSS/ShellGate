use gate_server::agenda;
use gate_server::bonsai;
use gate_server::config;
use gate_server::derived_grants::{self, NewDerivedGrant};
use gate_server::handler;
use gate_server::pipeline::{DeliberationContext, DeliberationStage, Pipeline, PipelineConfig, StageVerdict};
use gate_server::schema;
use gate_server::stages::allow_list::AllowListStage;
use gate_server::stages::catch_list::CatchListStage;
use gate_server::stages::human::HumanApprovalStage;
use gate_server::stages::llm::LlmStage;
use gate_server::types::{AppState, GateRequest};

use std::collections::HashMap;
use std::sync::Arc;

fn setup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    schema::run_migrations(&conn).expect("migrations");
    conn
}

fn make_ctx(command: &str, args: &[&str]) -> DeliberationContext {
    DeliberationContext {
        command: command.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        cwd: "/home/gem/projects/test".to_string(),
        pid: 1234,
        agendas: vec![],
        recent_history: vec![],
        config: test_config(),
    }
}

fn setup_temp_db() -> (tempfile::TempDir, String, rusqlite::Connection) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let db_path_str = db_path.to_string_lossy().to_string();
    let conn = rusqlite::Connection::open(&db_path).expect("open db");
    schema::run_migrations(&conn).expect("migrations");
    (dir, db_path_str, conn)
}

fn test_config() -> PipelineConfig {
    PipelineConfig::default()
}

fn seed_agenda_with_grants(
    conn: &rusqlite::Connection,
    grants: Vec<NewDerivedGrant>,
) -> String {
    let agenda = agenda::create_agenda(conn, "test", "test agenda", None, Some(3600))
        .expect("create agenda");
    derived_grants::create_derived_grants(conn, &agenda.id, &grants)
        .expect("create derived grants");
    agenda.id
}

fn make_app_state(db_path: &str) -> AppState {
    let config = gate_server::types::Config {
        gate: gate_server::types::GateConfig {
            socket_path: "/tmp/test.sock".into(),
            db_path: db_path.to_string(),
            audit_ttl_secs: 86400,
            rest_port: 3000,
            rest_host: "127.0.0.1".into(),
            pending_queue_max: 100,
            allowed_uids: vec![],
        },
        github: gate_server::types::GitHubConfig {
            app_id: 0,
            app_key_path: "/dev/null".into(),
            installation_id: 0,
        },
        telegram: gate_server::types::TelegramConfig {
            bot_token: "test".into(),
            chat_id: 0,
        },
        mcp: gate_server::types::McpConfig {
            fifo_path: "/tmp/test.fifo".into(),
        },
        web: gate_server::types::WebConfig {
            dist_path: "/dev/null".into(),
        },
        pipeline: test_config(),
    };
    AppState {
        db_path: db_path.to_string(),
        config,
        pending: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    }
}

#[test]
fn catch_list_matches_dangerous_command() {
    let stage = CatchListStage::new(&[
        "rm -rf *".into(),
        "auth:*".into(),
    ]).expect("build catch list");

    let ctx = make_ctx("rm", &["-rf", "/"]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    match result {
        StageVerdict::BlockAndNotify { .. } => {}
        other => panic!("expected BlockAndNotify, got {:?}", other),
    }
}

#[test]
fn catch_list_matches_auth_pattern() {
    let stage = CatchListStage::new(&["auth*".into()]).expect("build catch list");

    let ctx = make_ctx("auth", &["login", "--token", "abc123"]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    assert!(matches!(result, StageVerdict::BlockAndNotify { .. }));
}

#[test]
fn catch_list_passes_safe_command() {
    let stage = CatchListStage::new(&[
        "rm -rf *".into(),
        "auth:*".into(),
    ]).expect("build catch list");

    let ctx = make_ctx("git", &["status"]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    assert!(matches!(result, StageVerdict::Pass));
}

#[test]
fn catch_list_matches_command_name_only() {
    let stage = CatchListStage::new(&["dangerous".into()]).expect("build catch list");

    let ctx = make_ctx("dangerous", &[]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    assert!(matches!(result, StageVerdict::BlockAndNotify { .. }));
}

#[test]
fn catch_list_no_patterns_passes_everything() {
    let stage = CatchListStage::new(&[]).expect("build catch list");

    let ctx = make_ctx("rm", &["-rf", "/"]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    assert!(matches!(result, StageVerdict::Pass));
}

#[test]
fn allow_list_matches_derived_grant() {
    let (_dir, db_path, conn) = setup_temp_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "git".into(),
        args_pattern: "status*".into(),
        path_pattern: None,
        notification: "silent".into(),
        reason: Some("read-only git".into()),
        confidence: Some(0.95),
    }]);

    let stage = AllowListStage::new(0.0, db_path);
    let ctx = DeliberationContext {
        command: "git".into(),
        args: vec!["status".into(), "--short".into()],
        cwd: "/home/gem/projects/test".into(),
        pid: 1234,
        agendas: vec![],
        recent_history: vec![],
        config: test_config(),
    };

    let result = stage.evaluate(&ctx).expect("evaluate");
    assert!(matches!(result, StageVerdict::Allow));
}

#[test]
fn allow_list_advisory_grant_returns_allow_and_notify() {
    let (_dir, db_path, conn) = setup_temp_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "npm".into(),
        args_pattern: "install*".into(),
        path_pattern: None,
        notification: "advisory".into(),
        reason: Some("dep install".into()),
        confidence: Some(0.6),
    }]);

    let stage = AllowListStage::new(0.0, db_path);
    let ctx = DeliberationContext {
        command: "npm".into(),
        args: vec!["install".into(), "lodash".into()],
        cwd: "/home/gem/projects/test".into(),
        pid: 1234,
        agendas: vec![],
        recent_history: vec![],
        config: test_config(),
    };

    let result = stage.evaluate(&ctx).expect("evaluate");
    assert!(matches!(result, StageVerdict::AllowAndNotify { .. }));
}

#[test]
fn allow_list_no_match_returns_pass() {
    let (_dir, db_path, conn) = setup_temp_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "git".into(),
        args_pattern: "status*".into(),
        path_pattern: None,
        notification: "silent".into(),
        reason: Some("read-only git".into()),
        confidence: Some(0.95),
    }]);

    let stage = AllowListStage::new(0.0, db_path);
    let ctx = make_ctx("curl", &["https://example.com"]);

    let result = stage.evaluate(&ctx).expect("evaluate");
    assert!(matches!(result, StageVerdict::Pass));
}

#[test]
fn allow_list_expired_agenda_ignored() {
    let (_dir, db_path, conn) = setup_temp_db();

    let agenda = agenda::create_agenda(&conn, "test", "expired agenda", None, Some(0))
        .expect("create agenda");

    derived_grants::create_derived_grants(&conn, &agenda.id, &[NewDerivedGrant {
        command_pattern: "git".into(),
        args_pattern: "*".into(),
        path_pattern: None,
        notification: "silent".into(),
        reason: Some("should not match".into()),
        confidence: Some(0.9),
    }]).expect("create grants");

    let now = chrono::Utc::now().to_rfc3339();
    agenda::expire_agendas(&conn, &now).expect("expire");

    let stage = AllowListStage::new(0.0, db_path);
    let ctx = make_ctx("git", &["status"]);

    let result = stage.evaluate(&ctx).expect("evaluate");
    assert!(matches!(result, StageVerdict::Pass));
}

#[test]
fn pipeline_allow_list_stops_before_catch_list() {
    let (_dir, db_path, conn) = setup_temp_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "git".into(),
        args_pattern: "*".into(),
        path_pattern: None,
        notification: "silent".into(),
        reason: Some("all git ok".into()),
        confidence: Some(0.99),
    }]);

    let allow = AllowListStage::new(0.0, db_path);
    let catch = CatchListStage::new(&["git *".into()]).expect("catch list");

    let pipeline = Pipeline::new(vec![
        Box::new(allow),
        Box::new(catch),
    ]);

    let ctx = make_ctx("git", &["push"]);
    let result = pipeline.run(&ctx);

    assert!(result.allowed);
    assert_eq!(result.stage_name, "allow_list");
}

#[test]
fn pipeline_catch_list_blocks_after_allow_passes() {
    let (_dir, db_path, _conn) = setup_temp_db();

    let allow = AllowListStage::new(0.0, db_path);
    let catch = CatchListStage::new(&["rm *".into()]).expect("catch list");

    let pipeline = Pipeline::new(vec![
        Box::new(allow),
        Box::new(catch),
    ]);

    let ctx = make_ctx("rm", &["-rf", "/tmp/test"]);
    let result = pipeline.run(&ctx);

    assert!(!result.allowed);
    assert_eq!(result.stage_name, "catch_list");
}

#[test]
fn pipeline_all_pass_falls_through() {
    let catch = CatchListStage::new(&[]).expect("empty catch list");

    let pipeline = Pipeline::new(vec![
        Box::new(catch),
    ]);

    let ctx = make_ctx("cargo", &["build"]);
    let result = pipeline.run(&ctx);

    assert!(!result.allowed);
    assert_eq!(result.stage_name, "pipeline_end");
}

#[test]
fn pipeline_human_stage_blocks() {
    let human = HumanApprovalStage {
        timeout_secs: 300,
        pending: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    let pipeline = Pipeline::new(vec![Box::new(human)]);

    let ctx = make_ctx("gh", &["pr", "create"]);
    let result = pipeline.run(&ctx);

    assert!(!result.allowed);
    assert_eq!(result.stage_name, "human");
}

#[test]
fn llm_stage_passes_when_model_unavailable() {
    let bonsai_config = gate_server::pipeline::BonsaiConfig {
        model_path: "/nonexistent/model.gguf".into(),
        model_size: "4b".into(),
        max_tokens: 128,
        temperature: 0.1,
    };

    let model = gate_server::bonsai::BonsaiModel::load(&bonsai_config).expect("load bonsai");
    assert!(!model.is_available());

    let stage = LlmStage::new(Arc::new(model), ":memory:".to_string());

    let ctx = make_ctx("cargo", &["test"]);
    let result = stage.evaluate(&ctx).expect("evaluate");

    assert!(matches!(result, StageVerdict::Pass));
}

#[test]
fn derived_grant_path_pattern_filtering() {
    let conn = setup_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "npm".into(),
        args_pattern: "test".into(),
        path_pattern: Some("/home/gem/projects/myapp/*".into()),
        notification: "silent".into(),
        reason: Some("tests in myapp only".into()),
        confidence: Some(0.9),
    }]);

    let match_result = derived_grants::find_matching_derived_grant(
        &conn,
        "npm",
        &["test".to_string()],
        "/home/gem/projects/myapp/src",
    ).expect("find grant");
    assert!(match_result.is_some());

    let no_match = derived_grants::find_matching_derived_grant(
        &conn,
        "npm",
        &["test".to_string()],
        "/home/gem/projects/other",
    ).expect("find grant");
    assert!(no_match.is_none());
}

#[test]
fn derived_grant_args_pattern_matching() {
    let conn = setup_db();

    seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
        command_pattern: "git".into(),
        args_pattern: "status*".into(),
        path_pattern: None,
        notification: "silent".into(),
        reason: Some("git status variants".into()),
        confidence: Some(0.9),
    }]);

    let match1 = derived_grants::find_matching_derived_grant(
        &conn, "git", &["status".to_string()], "/any",
    ).expect("find");
    assert!(match1.is_some());

    let match2 = derived_grants::find_matching_derived_grant(
        &conn, "git", &["status".to_string(), "--short".to_string()], "/any",
    ).expect("find");
    assert!(match2.is_some());

    let no_match = derived_grants::find_matching_derived_grant(
        &conn, "git", &["push".to_string()], "/any",
    ).expect("find");
    assert!(no_match.is_none());
}

#[test]
fn parse_deliberation_malformed_input() {
    assert!(gate_server::prompts::parse_deliberation("").is_none());
    assert!(gate_server::prompts::parse_deliberation("DECISION: ALLOW").is_none());
    assert!(gate_server::prompts::parse_deliberation("garbage").is_none());
}

#[test]
fn parse_deliberation_case_insensitive() {
    let raw = "DECISION: allow\nCONFIDENCE: 0.85\nREASON: fits agenda";
    let parsed = gate_server::prompts::parse_deliberation(raw).expect("parse");
    assert_eq!(parsed.decision, "ALLOW");
}

#[test]
fn parse_rules_empty_array() {
    let raw = "```json\n[]\n```";
    let rules = gate_server::prompts::parse_rules(raw).expect("parse");
    assert!(rules.is_empty());
}

#[test]
fn parse_rules_invalid_json() {
    assert!(gate_server::prompts::parse_rules("not json at all").is_none());
}

#[test]
fn parse_questions_valid() {
    let raw = r#"{"questions": [
        {"question": "What?", "type": "text", "options": []},
        {"question": "Which?", "type": "choice", "options": ["a", "b"]}
    ]}"#;
    let parsed = gate_server::prompts::parse_questions(raw).expect("parse");
    assert_eq!(parsed.questions.len(), 2);
    assert_eq!(parsed.questions[1].options.len(), 2);
}

#[test]
fn parse_questions_invalid() {
    assert!(gate_server::prompts::parse_questions("no json here").is_none());
}

#[tokio::test]
async fn handler_allows_command_with_derived_grant() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let db_path_str = db_path.to_string_lossy().to_string();

    {
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        schema::run_migrations(&conn).expect("migrations");
        seed_agenda_with_grants(&conn, vec![NewDerivedGrant {
            command_pattern: "git".into(),
            args_pattern: "status".into(),
            path_pattern: None,
            notification: "silent".into(),
            reason: Some("read-only git".into()),
            confidence: Some(0.95),
        }]);
    }

    let state = make_app_state(&db_path_str);

    let request = GateRequest {
        command: "git".into(),
        args: vec!["status".into()],
        cwd: "/home/gem/projects/test".into(),
        pid: 1234,
    };

    let response = handler::handle_check_request(&request, &state, None)
        .await
        .expect("handle");

    assert_eq!(response.action, "allow");
}

#[tokio::test]
async fn handler_rejects_command_matching_catch_list() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let db_path_str = db_path.to_string_lossy().to_string();

    {
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        schema::run_migrations(&conn).expect("migrations");
    }

    let state = make_app_state(&db_path_str);

    let request = GateRequest {
        command: "rm".into(),
        args: vec!["-rf".into(), "/".into()],
        cwd: "/home/gem/projects/test".into(),
        pid: 1234,
    };

    let response = handler::handle_check_request(&request, &state, None)
        .await
        .expect("handle");

    assert_eq!(response.action, "reject");
}

#[tokio::test]
async fn handler_rejects_unknown_command() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let db_path_str = db_path.to_string_lossy().to_string();

    {
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        schema::run_migrations(&conn).expect("migrations");
    }

    let mut config = test_config();
    config.stages.catch_list.patterns = vec!["auth:*".into()];

    let mut state = make_app_state(&db_path_str);
    state.config.pipeline = config;

    let request = GateRequest {
        command: "curl".into(),
        args: vec!["https://example.com".into()],
        cwd: "/home/gem/projects/test".into(),
        pid: 1234,
    };

    let response = handler::handle_check_request(&request, &state, None)
        .await
        .expect("handle");

    assert_eq!(response.action, "reject");
}

#[test]
fn agenda_crud_lifecycle() {
    let conn = setup_db();

    let a = agenda::create_agenda(&conn, "test", "build feature X", Some("src/**/*.rs"), None)
        .expect("create");

    let listed = agenda::list_active_agendas(&conn).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, a.id);

    let fetched = agenda::get_agenda(&conn, &a.id).expect("get");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().description, "build feature X");
}

#[test]
fn agenda_replace_supersedes_active() {
    let conn = setup_db();

    agenda::create_agenda(&conn, "test", "old agenda", None, None).expect("create");
    let new = agenda::replace_agenda(&conn, "test", "new agenda", None, None).expect("replace");

    let active = agenda::list_active_agendas(&conn).expect("list");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, new.id);
    assert_eq!(active[0].description, "new agenda");
}

#[test]
fn schema_idempotent_migrations() {
    let conn = setup_db();
    schema::run_migrations(&conn).expect("re-run migrations");
    schema::run_migrations(&conn).expect("third run");

    let version: String = conn
        .query_row("SELECT value FROM config WHERE key='schema_version'", [], |row| row.get(0))
        .expect("version");
    assert_eq!(version, "2");
}

#[test]
fn config_load_toml() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config_path = dir.path().join("config.toml");

    std::fs::write(&config_path, r#"
[gate]
socket_path = "/run/gate.sock"
db_path = "/opt/gate/db.sqlite"
audit_ttl_secs = 86400
rest_port = 3000
rest_host = "127.0.0.1"
pending_queue_max = 100
allowed_uids = [1000]

[github]
app_id = 12345
app_key_path = "/opt/gate/key.pem"
installation_id = 67890

[telegram]
bot_token = "123456:ABC"
chat_id = -1001234567890

[mcp]
fifo_path = "/tmp/gate-mcp.fifo"

[web]
dist_path = "/opt/gate/web/dist"
"#).expect("write config");

    let config = gate_server::config::load_config(config_path.to_str().unwrap()).expect("load config");
    assert_eq!(config.gate.rest_port, 3000);
    assert_eq!(config.github.app_id, 12345);
    assert_eq!(config.telegram.chat_id, -1001234567890);
}
