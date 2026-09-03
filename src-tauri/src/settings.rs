//! Typed accessors over the `settings` key/value table.

use crate::{db::Db, ArgusError, ArgusResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub vault_path: Option<String>,
    pub ollama_host: Option<String>,
    pub ollama_model: Option<String>,
    pub embed_model: Option<String>,
    pub data_retention_days: u32,
    pub similarity_threshold: f32,
    pub min_session_seconds: u32,
    pub exclusion_list: Vec<ExclusionEntry>,
    pub warn_missing_vault: bool,
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
            .filter(|s| !s.trim().is_empty())
            .cloned(),
        ollama_model: map
            .get("ollama_model")
            .filter(|s| !s.trim().is_empty())
            .cloned(),
        embed_model: map
            .get("embed_model")
            .filter(|s| !s.trim().is_empty())
            .cloned(),
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
        warn_missing_vault: map
            .get("warn_missing_vault")
            .map(|s| s != "false" && s != "0")
            .unwrap_or(true),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warn_missing_vault_default() {
        let db = Db::open_in_memory().unwrap();
        let s = get_all(&db).unwrap();
        assert!(s.warn_missing_vault, "warn_missing_vault should default to true");
    }

    #[test]
    fn test_warn_missing_vault_toggle() {
        let db = Db::open_in_memory().unwrap();
        set_string(&db, "warn_missing_vault", "false").unwrap();
        let s = get_all(&db).unwrap();
        assert!(!s.warn_missing_vault, "warn_missing_vault should be false after updating");

        set_string(&db, "warn_missing_vault", "true").unwrap();
        let s2 = get_all(&db).unwrap();
        assert!(s2.warn_missing_vault, "warn_missing_vault should be true after updating back");
    }

    #[test]
    fn test_llm_fields_no_defaults() {
        let db = Db::open_in_memory().unwrap();
        let s = get_all(&db).unwrap();
        assert!(s.ollama_host.is_none(), "ollama_host should be None by default");
        assert!(s.ollama_model.is_none(), "ollama_model should be None by default");
        assert!(s.embed_model.is_none(), "embed_model should be None by default");
    }
}
