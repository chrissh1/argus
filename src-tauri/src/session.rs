//! Session lifecycle: `Idle → Active → Paused → Synthesizing → Complete | Interrupted`.
//!
//! Only one active session at a time. The currently-running session is held in
//! `state::AppState::current`. Persisted snapshots live in `app.db`.

use crate::{db::Db, paths, ArgusError, ArgusResult};
use chrono::{DateTime, Local, TimeZone, Utc};
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Idle,
    Active,
    Paused,
    Synthesizing,
    Complete,
    Interrupted,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Idle => "idle",
            Status::Active => "active",
            Status::Paused => "paused",
            Status::Synthesizing => "synthesizing",
            Status::Complete => "complete",
            Status::Interrupted => "interrupted",
        }
    }

    pub fn from_str(s: &str) -> ArgusResult<Self> {
        Ok(match s {
            "idle" => Status::Idle,
            "active" => Status::Active,
            "paused" => Status::Paused,
            "synthesizing" => Status::Synthesizing,
            "complete" => Status::Complete,
            "interrupted" => Status::Interrupted,
            other => return Err(ArgusError::InvalidState(format!("unknown status: {other}"))),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: String,
    pub display_name: Option<String>,
    pub status: Status,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_secs: i64,
    pub paused_secs: i64,
    pub vault_files_affected: Vec<VaultFileAffected>,
    pub action_items: Vec<String>,
    pub open_questions: Vec<String>,
    pub raw_db_path: Option<String>,
    pub raw_db_expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultFileAffected {
    pub path: String,
    pub action: VaultAction,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VaultAction {
    Appended,
    Created,
}

/// Generate a fresh session id like `session_2026-06-04_1430`.
pub fn new_session_id(now: DateTime<Local>) -> String {
    format!("session_{}", now.format("%Y-%m-%d_%H%M"))
}

pub fn create(db: &Db, retention_days: u32) -> ArgusResult<SessionRecord> {
    let now_local = Local::now();
    let id = unique_id(db, new_session_id(now_local))?;
    let started_at = Utc::now().timestamp();
    let raw_db_path = paths::session_raw_db(&id)?.to_string_lossy().to_string();
    let expires = if retention_days > 0 {
        Some(started_at + (retention_days as i64) * 86_400)
    } else {
        None
    };

    db.with_conn(|c| {
        c.execute(
            "INSERT INTO sessions
                (id, display_name, status, started_at, duration_secs, paused_secs,
                 raw_db_path, raw_db_expires_at)
             VALUES (?1, NULL, 'active', ?2, 0, 0, ?3, ?4)",
            params![id, started_at, raw_db_path, expires],
        )?;
        Ok(())
    })?;

    Ok(SessionRecord {
        id,
        display_name: None,
        status: Status::Active,
        started_at,
        ended_at: None,
        duration_secs: 0,
        paused_secs: 0,
        vault_files_affected: vec![],
        action_items: vec![],
        open_questions: vec![],
        raw_db_path: Some(raw_db_path),
        raw_db_expires_at: expires,
    })
}

fn unique_id(db: &Db, base: String) -> ArgusResult<String> {
    db.with_conn(|c| {
        let mut id = base.clone();
        let mut suffix = 1;
        loop {
            let exists: bool = c
                .query_row(
                    "SELECT 1 FROM sessions WHERE id = ?1",
                    params![id],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if !exists {
                return Ok(id);
            }
            id = format!("{base}_{suffix}");
            suffix += 1;
        }
    })
}

pub fn set_status(db: &Db, id: &str, status: Status) -> ArgusResult<()> {
    db.with_conn(|c| {
        c.execute(
            "UPDATE sessions SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        Ok(())
    })
}

pub fn mark_ended(db: &Db, id: &str, duration_secs: i64, paused_secs: i64) -> ArgusResult<()> {
    let now = Utc::now().timestamp();
    db.with_conn(|c| {
        c.execute(
            "UPDATE sessions
             SET status = 'synthesizing', ended_at = ?1,
                 duration_secs = ?2, paused_secs = ?3
             WHERE id = ?4",
            params![now, duration_secs, paused_secs, id],
        )?;
        Ok(())
    })
}

pub fn rename(db: &Db, id: &str, name: &str) -> ArgusResult<()> {
    db.with_conn(|c| {
        c.execute(
            "UPDATE sessions SET display_name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        Ok(())
    })
}

pub fn set_synthesis_results(
    db: &Db,
    id: &str,
    display_name: Option<&str>,
    action_items: &[String],
    open_questions: &[String],
    vault_files: &[VaultFileAffected],
) -> ArgusResult<()> {
    let ai = serde_json::to_string(action_items)?;
    let oq = serde_json::to_string(open_questions)?;
    let vf = serde_json::to_string(vault_files)?;
    db.with_conn(|c| {
        c.execute(
            "UPDATE sessions
             SET status = 'complete',
                 display_name = COALESCE(?1, display_name),
                 action_items = ?2,
                 open_questions = ?3,
                 vault_files_affected = ?4
             WHERE id = ?5",
            params![display_name, ai, oq, vf, id],
        )?;
        Ok(())
    })
}

pub fn list(db: &Db, limit: u32) -> ArgusResult<Vec<SessionRecord>> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, display_name, status, started_at, ended_at,
                    duration_secs, paused_secs, vault_files_affected,
                    action_items, open_questions, raw_db_path, raw_db_expires_at
             FROM sessions
             ORDER BY started_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })
}

pub fn get(db: &Db, id: &str) -> ArgusResult<SessionRecord> {
    db.with_conn(|c| {
        c.query_row(
            "SELECT id, display_name, status, started_at, ended_at,
                    duration_secs, paused_secs, vault_files_affected,
                    action_items, open_questions, raw_db_path, raw_db_expires_at
             FROM sessions WHERE id = ?1",
            params![id],
            from_row,
        )
        .map_err(Into::into)
    })
}

fn from_row(r: &Row) -> rusqlite::Result<SessionRecord> {
    let status_s: String = r.get(2)?;
    let status = Status::from_str(&status_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let vault_files: Vec<VaultFileAffected> = r
        .get::<_, Option<String>>(7)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let action_items: Vec<String> = r
        .get::<_, Option<String>>(8)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let open_questions: Vec<String> = r
        .get::<_, Option<String>>(9)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Ok(SessionRecord {
        id: r.get(0)?,
        display_name: r.get(1)?,
        status,
        started_at: r.get(3)?,
        ended_at: r.get(4)?,
        duration_secs: r.get(5)?,
        paused_secs: r.get(6)?,
        vault_files_affected: vault_files,
        action_items,
        open_questions,
        raw_db_path: r.get(10)?,
        raw_db_expires_at: r.get(11)?,
    })
}

/// Delete expired raw session DB files (TTL housekeeping). The `app.db`
/// session summary record is retained permanently.
pub fn vacuum_expired_raw_dbs(db: &Db) -> ArgusResult<u32> {
    let now = Utc::now().timestamp();
    let mut deleted = 0;
    let stale = db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, raw_db_path FROM sessions
             WHERE raw_db_path IS NOT NULL
               AND raw_db_expires_at IS NOT NULL
               AND raw_db_expires_at < ?1",
        )?;
        let rows = stmt
            .query_map(params![now], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })?;

    for (id, p) in stale {
        if let Some(parent) = std::path::Path::new(&p).parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
        db.with_conn(|c| {
            c.execute(
                "UPDATE sessions SET raw_db_path = NULL, raw_db_expires_at = NULL WHERE id = ?1",
                params![id],
            )?;
            Ok(())
        })?;
        deleted += 1;
    }
    Ok(deleted)
}

#[allow(dead_code)]
pub fn format_started_at(unix: i64) -> String {
    Local
        .timestamp_opt(unix, 0)
        .single()
        .map(|d| d.format("%a %b %-d, %Y · %-I:%M %p").to_string())
        .unwrap_or_default()
}
