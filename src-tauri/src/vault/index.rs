//! sqlite-vec vault vector index.
//!
//! Schema:
//!   chunks(id, note_path, chunk_text, content_hash, mtime, vec_rowid)
//!   vec_chunks(rowid, embedding) — virtual table, vec0
//!
//! On query: cosine top-k via `vec_chunks MATCH ?` join.

use crate::{llm::LlmClient, paths, vault::chunk, ArgusResult};
use rusqlite::{ffi, params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::{Mutex, Once};
use walkdir::WalkDir;

static VEC_INIT: Once = Once::new();

/// Register the sqlite-vec extension as a SQLite auto-extension so every
/// `Connection` opened afterwards has the `vec0` virtual table available.
fn ensure_vec_loaded() {
    VEC_INIT.call_once(|| unsafe {
        ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    });
}

pub struct VaultIndex {
    conn: Mutex<Connection>,
    /// Dimension of embeddings — fixed at construction; nomic-embed-text = 768.
    dim: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    pub configured: bool,
    pub note_count: u32,
    pub chunk_count: u32,
    pub last_indexed_at: Option<i64>,
    pub indexing: bool,
    pub progress_current: u32,
    pub progress_total: u32,
}

#[derive(Debug, Clone)]
pub struct CandidateChunk {
    pub note_path: String,
    pub chunk_text: String,
    pub similarity: f32,
}

impl VaultIndex {
    pub fn open(dim: usize) -> ArgusResult<Self> {
        ensure_vec_loaded();
        let path = paths::vault_index_db()?;
        let conn = Connection::open(path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                note_path   TEXT NOT NULL,
                chunk_idx   INTEGER NOT NULL,
                chunk_text  TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                mtime       INTEGER NOT NULL,
                UNIQUE(note_path, chunk_idx)
            );
            CREATE INDEX IF NOT EXISTS chunks_note_path_idx ON chunks(note_path);

            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;

        // Create the vec0 virtual table dimensioned to our model.
        let create = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(embedding float[{}])",
            dim
        );
        conn.execute(&create, [])?;

        Ok(VaultIndex { conn: Mutex::new(conn), dim })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn status(&self, vault_path: Option<&str>) -> ArgusResult<IndexStatus> {
        let conn = self.conn.lock().unwrap();
        let chunk_count: u32 =
            conn.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))?;
        let note_count: u32 = conn.query_row(
            "SELECT COUNT(DISTINCT note_path) FROM chunks",
            [],
            |r| r.get(0),
        )?;
        let last_indexed_at: Option<i64> = conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM meta WHERE key='last_indexed_at'",
                [],
                |r| r.get::<_, Option<i64>>(0),
            )
            .ok()
            .flatten();
        Ok(IndexStatus {
            configured: vault_path.is_some(),
            note_count,
            chunk_count,
            last_indexed_at,
            indexing: false,
            progress_current: 0,
            progress_total: 0,
        })
    }

    /// Walk the vault, embed every chunk whose hash/mtime differs. Caller is
    /// responsible for reporting progress via the supplied callback.
    pub async fn reindex(
        &self,
        vault_path: &Path,
        llm: &dyn LlmClient,
        embed_model: &str,
        mut progress: impl FnMut(u32, u32) + Send,
    ) -> ArgusResult<()> {
        let files: Vec<_> = WalkDir::new(vault_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .collect();

        let total = files.len() as u32;
        progress(0, total);

        for (idx, entry) in files.iter().enumerate() {
            self.reindex_file(entry.path(), llm, embed_model).await?;
            progress(idx as u32 + 1, total);
        }

        self.set_meta("last_indexed_at", &chrono::Utc::now().timestamp().to_string())?;
        Ok(())
    }

    pub async fn reindex_file(
        &self,
        path: &Path,
        llm: &dyn LlmClient,
        embed_model: &str,
    ) -> ArgusResult<()> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => return Ok(()), // file may have been removed mid-walk
        };
        let mtime = path
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let hash = hex::encode(Sha256::digest(&bytes));
        let note_path = path.to_string_lossy().to_string();

        // Skip if every chunk for this file already matches the current hash.
        let unchanged = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM chunks WHERE note_path = ?1 AND content_hash = ?2",
            )?;
            let count: i64 = stmt.query_row(params![note_path, hash], |r| r.get(0))?;
            count > 0
        };
        if unchanged {
            return Ok(());
        }

        let text = String::from_utf8_lossy(&bytes).to_string();
        let chunks = chunk::chunk_text(&text);

        // Drop previous chunks for this file.
        self.purge_file(&note_path)?;

        for (i, c) in chunks.iter().enumerate() {
            let embedding = llm.embed(embed_model, c).await?;
            if embedding.len() != self.dim {
                tracing::warn!(
                    expected = self.dim,
                    got = embedding.len(),
                    "embedding dim mismatch; skipping chunk"
                );
                continue;
            }
            self.insert_chunk(&note_path, i as i64, c, &hash, mtime, &embedding)?;
        }
        Ok(())
    }

    pub fn purge_file(&self, note_path: &str) -> ArgusResult<()> {
        let conn = self.conn.lock().unwrap();
        // Cascade: rowid in `chunks` must equal rowid in `vec_chunks` to keep them aligned.
        let rowids: Vec<i64> = conn
            .prepare("SELECT id FROM chunks WHERE note_path = ?1")?
            .query_map(params![note_path], |r| r.get::<_, i64>(0))?
            .collect::<Result<_, _>>()?;
        for r in &rowids {
            conn.execute("DELETE FROM vec_chunks WHERE rowid = ?1", params![r])?;
        }
        conn.execute("DELETE FROM chunks WHERE note_path = ?1", params![note_path])?;
        Ok(())
    }

    fn insert_chunk(
        &self,
        note_path: &str,
        chunk_idx: i64,
        chunk_text: &str,
        hash: &str,
        mtime: i64,
        embedding: &[f32],
    ) -> ArgusResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chunks(note_path, chunk_idx, chunk_text, content_hash, mtime)
             VALUES (?1,?2,?3,?4,?5)",
            params![note_path, chunk_idx, chunk_text, hash, mtime],
        )?;
        let rowid = conn.last_insert_rowid();
        let bytes = bytemuck_cast(embedding);
        conn.execute(
            "INSERT INTO vec_chunks(rowid, embedding) VALUES (?1, ?2)",
            params![rowid, bytes],
        )?;
        Ok(())
    }

    pub fn topk(&self, query_emb: &[f32], k: usize) -> ArgusResult<Vec<CandidateChunk>> {
        if query_emb.len() != self.dim {
            return Ok(vec![]);
        }
        let bytes = bytemuck_cast(query_emb);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT c.note_path, c.chunk_text, v.distance
             FROM vec_chunks v
             JOIN chunks c ON c.id = v.rowid
             WHERE v.embedding MATCH ?1 AND k = ?2
             ORDER BY v.distance",
        )?;
        let rows = stmt
            .query_map(params![bytes, k as i64], |r| {
                let distance: f32 = r.get(2)?;
                Ok(CandidateChunk {
                    note_path: r.get(0)?,
                    chunk_text: r.get(1)?,
                    // sqlite-vec returns L2 distance by default; convert to a
                    // pseudo-cosine in [0,1] for thresholding.
                    similarity: 1.0 / (1.0 + distance),
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    fn set_meta(&self, key: &str, value: &str) -> ArgusResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO meta(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn bytemuck_cast(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for f in v {
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}
