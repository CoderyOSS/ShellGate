use crate::agenda;
use crate::llm_client::LlmClient;
use crate::prompts;
use crate::stages::llm::generate_rules_for_agenda;
use crate::types::GateError;

pub struct BootstrapResult {
    pub agenda_id: String,
    pub questions: Vec<BootstrapQuestion>,
    pub rules_generated: usize,
}

pub struct BootstrapQuestion {
    pub question: String,
    pub q_type: String,
    pub options: Vec<String>,
}

pub fn run_interactive_bootstrap(
    model: &LlmClient,
    conn: &rusqlite::Connection,
    approved_command: &str,
    recent_commands: &[String],
) -> Result<Option<BootstrapResult>, GateError> {
    if !model.is_available() {
        return Ok(None);
    }

    let agendas = agenda::list_active_agendas(conn)?;
    if !agendas.is_empty() {
        return Ok(None);
    }

    let prompt = prompts::question_generation_prompt(approved_command, recent_commands);

    let model_for_thread = model.clone();
    let raw = match std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .expect("failed to create bootstrap runtime");
        rt.block_on(model_for_thread.infer(&prompt))
    }).join() {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            tracing::error!(error = %e, "bootstrap question generation failed");
            return Ok(None);
        }
        Err(_) => {
            tracing::error!("bootstrap thread panicked");
            return Ok(None);
        }
    };

    let parsed = match prompts::parse_questions(&raw) {
        Some(p) => p,
        None => {
            tracing::warn!(output = %raw, "failed to parse bootstrap questions");
            return Ok(None);
        }
    };

    let questions: Vec<BootstrapQuestion> = parsed
        .questions
        .into_iter()
        .map(|q| BootstrapQuestion {
            question: q.question,
            q_type: q.q_type,
            options: q.options,
        })
        .collect();

    Ok(Some(BootstrapResult {
        agenda_id: String::new(),
        questions,
        rules_generated: 0,
    }))
}

pub fn complete_bootstrap(
    conn: &rusqlite::Connection,
    model: &LlmClient,
    description: &str,
    scope: Option<&str>,
    ttl_secs: Option<u64>,
) -> Result<String, GateError> {
    let agenda = agenda::create_agenda(conn, "interactive", description, scope, ttl_secs)?;

    let rules = generate_rules_for_agenda(model, conn, &agenda.id, description, scope)?;

    tracing::info!(
        agenda_id = %agenda.id,
        rules = rules,
        "interactive bootstrap completed"
    );

    Ok(agenda.id)
}
