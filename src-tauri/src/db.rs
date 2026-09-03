//! App database (`~/.argus/app.db`) — sessions + settings.
//!
//! We open a single connection in a mutex; SQLite is fast enough at our scale
//! that contention isn't a concern. All access is synchronous; long-running
//! work (synthesis, indexing) reads + clones what it needs and releases the lock.

use crate::{paths, ArgusResult};
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    display_name    TEXT,
    status          TEXT NOT NULL,
    started_at      INTEGER NOT NULL,
    ended_at        INTEGER,
    duration_secs   INTEGER NOT NULL DEFAULT 0,
    paused_secs     INTEGER NOT NULL DEFAULT 0,
    vault_files_affected TEXT,   -- JSON array
    action_items    TEXT,        -- JSON array
    open_questions  TEXT,        -- JSON array
    raw_db_path     TEXT,
    raw_db_expires_at INTEGER
);

CREATE INDEX IF NOT EXISTS sessions_started_at_idx ON sessions(started_at DESC);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

impl Db {
    pub fn open() -> ArgusResult<Self> {
        let path = paths::app_db_path()?;
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Self::seed_defaults(&conn)?;
        // Clean up legacy pre-seeded defaults so the user must explicitly configure their LLM
        let _ = conn.execute(
            "DELETE FROM settings WHERE (key = 'ollama_model' AND value = 'llama3.2') \
             OR (key = 'embed_model' AND value = 'nomic-embed-text') \
             OR (key = 'ollama_host' AND value = 'http://localhost:11434' AND NOT EXISTS (SELECT 1 FROM settings WHERE key = 'ollama_model'))",
            [],
        );
        Ok(Db { conn: Mutex::new(conn) })
    }

    fn seed_defaults(conn: &Connection) -> ArgusResult<()> {
        let defaults: &[(&str, &str)] = &[
            ("data_retention_days", "30"),
            ("similarity_threshold", "0.75"),
            ("min_session_seconds", "60"),
            ("exclusion_list", "[]"),
            ("warn_missing_vault", "true"),
        ];
        for (k, v) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn open_in_memory() -> ArgusResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Self::seed_defaults(&conn)?;
        Ok(Db { conn: Mutex::new(conn) })
    }

    pub fn with_conn<R>(&self, f: impl FnOnce(&Connection) -> ArgusResult<R>) -> ArgusResult<R> {
        let guard = self.conn.lock().unwrap();
        f(&guard)
    }
}
