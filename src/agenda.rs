use crate::types::{Agenda, GateError};

use chrono::Utc;

pub fn create_agenda(
    conn: &rusqlite::Connection,
    source: &str,
    description: &str,
    scope: Option<&str>,
    ttl_secs: Option<u64>,
) -> Result<Agenda, GateError> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let ttl = ttl_secs.unwrap_or(86400);
    let expires_at = Utc::now() + chrono::Duration::seconds(ttl as i64);
    let expires_str = expires_at.to_rfc3339();

    conn.execute(
        "INSERT INTO agendas (id, source, description, scope, status, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6)",
        rusqlite::params![&id, source, description, scope, &created_at, &expires_str],
    )?;

    Ok(Agenda {
        id,
        source: source.to_string(),
        description: description.to_string(),
        scope: scope.map(String::from),
        status: "active".to_string(),
        created_at,
        expires_at: expires_str,
    })
}

pub fn list_active_agendas(conn: &rusqlite::Connection) -> Result<Vec<Agenda>, GateError> {
    let now = Utc::now().to_rfc3339();
    expire_agendas(conn, &now)?;

    let mut stmt = conn.prepare(
        "SELECT id, source, description, scope, status, created_at, expires_at FROM agendas WHERE status = 'active' ORDER BY created_at DESC"
    )?;

    let agendas = stmt
        .query_map([], |row| {
            Ok(Agenda {
                id: row.get(0)?,
                source: row.get(1)?,
                description: row.get(2)?,
                scope: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                expires_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(agendas)
}

pub fn get_agenda(conn: &rusqlite::Connection, id: &str) -> Result<Option<Agenda>, GateError> {
    let mut stmt = conn.prepare(
        "SELECT id, source, description, scope, status, created_at, expires_at FROM agendas WHERE id = ?1"
    )?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(Agenda {
                id: row.get(0)?,
                source: row.get(1)?,
                description: row.get(2)?,
                scope: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                expires_at: row.get(6)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn expire_agendas(conn: &rusqlite::Connection, now: &str) -> Result<usize, GateError> {
    let count = conn.execute(
        "UPDATE agendas SET status = 'expired' WHERE status = 'active' AND expires_at < ?1",
        [now],
    )?;

    if count > 0 {
        conn.execute(
            "UPDATE derived_grants SET expires_at = 'expired' WHERE agenda_id IN (SELECT id FROM agendas WHERE status = 'expired')",
            [],
        )?;
    }

    Ok(count)
}

pub fn replace_agenda(
    conn: &rusqlite::Connection,
    source: &str,
    description: &str,
    scope: Option<&str>,
    ttl_secs: Option<u64>,
) -> Result<Agenda, GateError> {
    conn.execute(
        "UPDATE agendas SET status = 'replaced' WHERE source = ?1 AND status = 'active'",
        [source],
    )?;

    create_agenda(conn, source, description, scope, ttl_secs)
}
