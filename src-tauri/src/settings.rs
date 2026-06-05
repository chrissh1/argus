//! Typed accessors over the `settings` key/value table.

use crate::{db::Db, ArgusError, ArgusResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub vault_path: Option<String>,
    pub ollama_host: String,
    pub ollama_model: String,
    pub embed_model: String,
    pub data_retention_days: u32,
    pub similarity_threshold: f32,
    pub min_session_seconds: u32,
    pub exclusion_list: Vec<ExclusionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionEntry {
    pub name: String,
    pub bundle_id: String,
}

pub fn get_all(db: &Db) -> ArgusResult<Settings> {
    let map = read_map(db)?;
    Ok(Settings {
        vault_path: map.get("vault_path").cloned(),
        ollama_host: map
            .get("ollama_host")
            .cloned()
            .unwrap_or_else(|| "http://localhost:11434".into()),
        ollama_model: map.get("ollama_model").cloned().unwrap_or_else(|| "llama3.2".into()),
        embed_model: map
            .get("embed_model")
            .cloned()
            .unwrap_or_else(|| "nomic-embed-text".into()),
        data_retention_days: map
            .get("data_retention_days")
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
        similarity_threshold: map
            .get("similarity_threshold")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.75),
        min_session_seconds: map
            .get("min_session_seconds")
            .and_then(|s| s.parse().ok())
            .unwrap_or(60),
        exclusion_list: map
            .get("exclusion_list")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default(),
    })
}

pub fn get_string(db: &Db, key: &str) -> ArgusResult<Option<String>> {
    db.with_conn(|c| {
        let mut stmt = c.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get::<_, String>(0)?))
        } else {
            Ok(None)
        }
    })
}

pub fn set_string(db: &Db, key: &str, value: &str) -> ArgusResult<()> {
    db.with_conn(|c| {
        c.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    })
}

pub fn set_json<T: Serialize>(db: &Db, key: &str, value: &T) -> ArgusResult<()> {
    let s = serde_json::to_string(value)?;
    set_string(db, key, &s)
}

pub fn get_exclusion_list(db: &Db) -> ArgusResult<Vec<ExclusionEntry>> {
    Ok(get_string(db, "exclusion_list")?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default())
}

pub fn require_vault_path(db: &Db) -> ArgusResult<String> {
    get_string(db, "vault_path")?
        .ok_or(ArgusError::PathNotConfigured("vault_path"))
}

fn read_map(db: &Db) -> ArgusResult<BTreeMap<String, String>> {
    db.with_conn(|c| {
        let mut stmt = c.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut map = BTreeMap::new();
        for r in rows {
            let (k, v) = r?;
            map.insert(k, v);
        }
        Ok(map)
    })
}
