use crate::agenda;
use crate::bonsai::BonsaiModel;
use crate::stages::llm::generate_rules_for_agenda;
use crate::types::GateError;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct OpenSpecWatcher {
    projects_dir: PathBuf,
    db_path: String,
    model: Arc<BonsaiModel>,
}

impl OpenSpecWatcher {
    pub fn new(projects_dir: impl Into<PathBuf>, db_path: String, model: Arc<BonsaiModel>) -> Self {
        Self {
            projects_dir: projects_dir.into(),
            db_path,
            model,
        }
    }

    pub async fn run(&self) -> Result<(), GateError> {
        let watch_dir = self.projects_dir.join("openspec").join("changes");

        if !watch_dir.exists() {
            tracing::info!(dir = %watch_dir.display(), "openspec changes dir not found, watcher idle");
            return Ok(());
        }

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.scan_changes().await {
                tracing::error!(error = %e, "openspec watcher scan failed");
            }
        }
    }

    async fn scan_changes(&self) -> Result<(), GateError> {
        let changes_dir = self.projects_dir.join("openspec").join("changes");
        let conn = rusqlite::Connection::open(&self.db_path)?;

        let mut read_dir = match tokio::fs::read_dir(&changes_dir).await {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };

        while let Some(entry) = read_dir.next_entry().await? {
            let proposal_path = entry.path().join("proposal.md");
            if !proposal_path.exists() {
                continue;
            }

            let change_name = entry.file_name().to_string_lossy().to_string();
            let source = format!("openspec:{}", change_name);

            let existing = conn.query_row(
                "SELECT id FROM agendas WHERE source = ?1 AND status = 'active'",
                [&source],
                |row| row.get::<_, String>(0),
            );

            if existing.is_ok() {
                continue;
            }

            let content = tokio::fs::read_to_string(&proposal_path).await?;
            let (description, scope) = parse_proposal(&content);

            let agenda = agenda::create_agenda(&conn, &source, &description, scope.as_deref(), None)?;

            tracing::info!(
                change = %change_name,
                agenda_id = %agenda.id,
                "created agenda from openspec proposal"
            );

            let _ = generate_rules_for_agenda(
                &self.model,
                &conn,
                &agenda.id,
                &description,
                scope.as_deref(),
            );
        }

        Ok(())
    }
}

fn parse_proposal(content: &str) -> (String, Option<String>) {
    let mut description = String::new();
    let mut in_what_changes = false;

    for line in content.lines() {
        if line.starts_with("## What Changes") {
            in_what_changes = true;
            continue;
        }
        if line.starts_with("## ") {
            in_what_changes = false;
            continue;
        }
        if line.starts_with("## Why") {
            continue;
        }

        if in_what_changes && !line.trim().is_empty() && !line.starts_with('-') {
            if description.is_empty() {
                description = line.trim().to_string();
                break;
            }
        }
    }

    if description.is_empty() {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('-') {
                description = trimmed.to_string();
                break;
            }
        }
    }

    (description, None)
}
