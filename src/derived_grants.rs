use crate::types::{DerivedGrant, GateError};
use crate::pipeline::AgendaSummary;

use globset::{Glob, GlobBuilder, GlobMatcher};

pub fn create_derived_grants(
    conn: &rusqlite::Connection,
    agenda_id: &str,
    grants: &[NewDerivedGrant],
) -> Result<Vec<DerivedGrant>, GateError> {
    let now = chrono::Utc::now().to_rfc3339();

    let agenda_expires: String = conn.query_row(
        "SELECT expires_at FROM agendas WHERE id = ?1",
        rusqlite::params![agenda_id],
        |row| row.get(0),
    )?;

    let mut results = Vec::new();
    for g in grants {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO derived_grants (id, agenda_id, command_pattern, args_pattern, path_pattern, notification, reason, confidence, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &id,
                agenda_id,
                &g.command_pattern,
                &g.args_pattern,
                &g.path_pattern,
                &g.notification,
                &g.reason,
                &g.confidence,
                &now,
                &agenda_expires,
            ],
        )?;

        results.push(DerivedGrant {
            id,
            agenda_id: agenda_id.to_string(),
            command_pattern: g.command_pattern.clone(),
            args_pattern: g.args_pattern.clone(),
            path_pattern: g.path_pattern.clone(),
            notification: g.notification.clone(),
            reason: g.reason.clone(),
            confidence: g.confidence,
            created_at: now.clone(),
            expires_at: agenda_expires.clone(),
        });
    }

    Ok(results)
}

#[derive(Debug, Clone)]
pub struct NewDerivedGrant {
    pub command_pattern: String,
    pub args_pattern: String,
    pub path_pattern: Option<String>,
    pub notification: String,
    pub reason: Option<String>,
    pub confidence: Option<f64>,
}

pub struct DerivedGrantMatch {
    pub grant: DerivedGrant,
    pub notification: String,
}

pub fn find_matching_derived_grant(
    conn: &rusqlite::Connection,
    command: &str,
    args: &[String],
    cwd: &str,
) -> Result<Option<DerivedGrantMatch>, GateError> {
    let now = chrono::Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT dg.id, dg.agenda_id, dg.command_pattern, dg.args_pattern, dg.path_pattern, dg.notification, dg.reason, dg.confidence, dg.created_at, dg.expires_at \
         FROM derived_grants dg \
         JOIN agendas a ON dg.agenda_id = a.id \
         WHERE a.status = 'active' AND dg.expires_at > ?1 \
         ORDER BY dg.confidence DESC NULLS LAST"
    )?;

    let args_str = args.join(" ");
    let grants: Vec<DerivedGrant> = stmt
        .query_map(rusqlite::params![now], |row| {
            Ok(DerivedGrant {
                id: row.get(0)?,
                agenda_id: row.get(1)?,
                command_pattern: row.get(2)?,
                args_pattern: row.get(3)?,
                path_pattern: row.get(4)?,
                notification: row.get(5)?,
                reason: row.get(6)?,
                confidence: row.get(7)?,
                created_at: row.get(8)?,
                expires_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for grant in grants {
        if let Ok(cmd_matcher) = build_matcher(&grant.command_pattern) {
            if !cmd_matcher.is_match(command) {
                continue;
            }
        } else {
            continue;
        }

        if let Ok(args_matcher) = build_matcher(&grant.args_pattern) {
            if !args_matcher.is_match(&args_str) {
                continue;
            }
        } else {
            continue;
        }

        if let Some(ref path_pattern) = grant.path_pattern {
            if let Ok(path_matcher) = build_matcher(path_pattern) {
                if !path_matcher.is_match(cwd) {
                    continue;
                }
            } else {
                continue;
            }
        }

        return Ok(Some(DerivedGrantMatch {
            notification: grant.notification.clone(),
            grant,
        }));
    }

    Ok(None)
}

pub fn delete_derived_grants_for_agenda(
    conn: &rusqlite::Connection,
    agenda_id: &str,
) -> Result<usize, GateError> {
    Ok(conn.execute(
        "DELETE FROM derived_grants WHERE agenda_id = ?1",
        rusqlite::params![agenda_id],
    )?)
}

pub fn list_derived_grants(
    conn: &rusqlite::Connection,
    agenda_id: Option<&str>,
) -> Result<Vec<DerivedGrant>, GateError> {
    let sql = match agenda_id {
        Some(_) => "SELECT id, agenda_id, command_pattern, args_pattern, path_pattern, notification, reason, confidence, created_at, expires_at FROM derived_grants WHERE agenda_id = ?1 ORDER BY created_at DESC",
        None => "SELECT id, agenda_id, command_pattern, args_pattern, path_pattern, notification, reason, confidence, created_at, expires_at FROM derived_grants ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;

    let grants: Vec<DerivedGrant> = if let Some(aid) = agenda_id {
        stmt.query_map(rusqlite::params![aid], |row| {
            Ok(DerivedGrant {
                id: row.get(0)?,
                agenda_id: row.get(1)?,
                command_pattern: row.get(2)?,
                args_pattern: row.get(3)?,
                path_pattern: row.get(4)?,
                notification: row.get(5)?,
                reason: row.get(6)?,
                confidence: row.get(7)?,
                created_at: row.get(8)?,
                expires_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], |row| {
            Ok(DerivedGrant {
                id: row.get(0)?,
                agenda_id: row.get(1)?,
                command_pattern: row.get(2)?,
                args_pattern: row.get(3)?,
                path_pattern: row.get(4)?,
                notification: row.get(5)?,
                reason: row.get(6)?,
                confidence: row.get(7)?,
                created_at: row.get(8)?,
                expires_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(grants)
}

fn build_matcher(pattern: &str) -> Result<GlobMatcher, globset::Error> {
    GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .map(|g| g.compile_matcher())
}

pub fn agendas_to_summaries(
    conn: &rusqlite::Connection,
) -> Result<Vec<AgendaSummary>, GateError> {
    let agendas = crate::agenda::list_active_agendas(conn)?;
    Ok(agendas
        .into_iter()
        .map(|a| AgendaSummary {
            id: a.id,
            description: a.description,
            scope: a.scope,
            source: a.source,
        })
        .collect())
}
