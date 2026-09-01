//! Mock session data generator for backend development and testing.

use crate::{
    db::Db,
    paths,
    session::{SessionRecord, Status},
    ArgusResult,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;

/// Generate realistic synthetic deep-work session data into a session's raw.db.
pub fn generate_mock_raw_db(db_path: &Path, duration_mins: u32) -> ArgusResult<()> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if db_path.exists() {
        let _ = std::fs::remove_file(db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ocr_frames (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            app_name TEXT,
            window_title TEXT,
            text TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS audio_chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            transcript TEXT NOT NULL,
            duration_ms INTEGER NOT NULL
        );",
    )?;

    let mut stmt_ocr = conn.prepare(
        "INSERT INTO ocr_frames (timestamp, app_name, window_title, text) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut stmt_audio = conn.prepare(
        "INSERT INTO audio_chunks (timestamp, transcript, duration_ms) VALUES (?1, ?2, ?3)",
    )?;

    // Simulate multi-phase deep work session (e.g., Auth Refactoring & SQLite Vec integration)
    let total_secs = (duration_mins * 60) as i64;
    let mut current_sec = 0i64;

    // Phase 1: Planning & Architecture (0 to 10 mins)
    while current_sec < 600 && current_sec < total_secs {
        stmt_ocr.execute(params![
            current_sec,
            "Obsidian",
            "Architecture Notes - Auth & Vector DB",
            "Planning migration from JWT to PASETO tokens. Need to verify sqlite-vec integration with rusqlite 0.32. Open question: token expiration policy in distributed clusters."
        ])?;

        if current_sec % 45 == 0 {
            stmt_audio.execute(params![
                current_sec,
                "I'm reviewing the authentication design. We need to make sure the token expiration handles clock skew cleanly.",
                12000
            ])?;
        }
        current_sec += 15;
    }

    // Phase 2: Implementation & Coding in Editor (10 to 20 mins)
    while current_sec < 1200 && current_sec < total_secs {
        stmt_ocr.execute(params![
            current_sec,
            "VS Code",
            "src/auth/middleware.rs - argus",
            "pub async fn verify_token(header: &str) -> Result<Claims, AuthError> { let token = parse_bearer(header)?; claims.validate_expiration()?; Ok(claims) }"
        ])?;

        if current_sec % 60 == 0 {
            stmt_audio.execute(params![
                current_sec,
                "Just finished writing the token verification helper in middleware.rs. Next step is writing unit tests for expired tokens.",
                15000
            ])?;
        }
        current_sec += 15;
    }

    // Phase 3: Testing & Debugging in Terminal (20 to 30 mins)
    while current_sec < 1800 && current_sec < total_secs {
        stmt_ocr.execute(params![
            current_sec,
            "Ghostty",
            "cargo test --test auth_integration",
            "running 4 tests\ntest tests::test_valid_token ... ok\ntest tests::test_expired_token ... ok\ntest tests::test_invalid_signature ... ok\ntest result: ok. 4 passed; 0 failed"
        ])?;

        if current_sec % 50 == 0 {
            stmt_audio.execute(params![
                current_sec,
                "All unit tests pass. Action item: update documentation and create pull request for auth middleware.",
                10000
            ])?;
        }
        current_sec += 15;
    }

    // Phase 4: Extended work if > 30 mins
    while current_sec < total_secs {
        stmt_ocr.execute(params![
            current_sec,
            "Safari",
            "sqlite-vec Documentation",
            "sqlite-vec allows fast vector similarity search using vec0 virtual tables. Supports L2 distance and Cosine metric."
        ])?;
        if current_sec % 60 == 0 {
            stmt_audio.execute(params![
                current_sec,
                "Checking sqlite-vec documentation for HNSW index configurations.",
                10000
            ])?;
        }
        current_sec += 15;
    }

    Ok(())
}

/// Create a full mock session in `app.db` and populate its `raw.db`.
pub fn create_mock_session(db: &Db, duration_mins: u32) -> ArgusResult<SessionRecord> {
    let now = Utc::now().timestamp();
    let duration_secs = (duration_mins * 60) as i64;
    let id = format!("mock_session_{}", Utc::now().format("%Y-%m-%d_%H%M%S"));
    let raw_db_path = paths::session_raw_db(&id)?.to_string_lossy().to_string();

    generate_mock_raw_db(Path::new(&raw_db_path), duration_mins)?;

    db.with_conn(|c| {
        c.execute(
            "INSERT INTO sessions
                (id, display_name, status, started_at, ended_at, duration_secs, paused_secs,
                 raw_db_path, raw_db_expires_at)
             VALUES (?1, ?2, 'synthesizing', ?3, ?4, ?5, 0, ?6, NULL)",
            params![
                id,
                "Mock: Auth & Vector Search Architecture",
                now - duration_secs,
                now,
                duration_secs,
                raw_db_path
            ],
        )?;
        Ok(())
    })?;

    Ok(SessionRecord {
        id,
        display_name: Some("Mock: Auth & Vector Search Architecture".into()),
        status: Status::Synthesizing,
        started_at: now - duration_secs,
        ended_at: Some(now),
        duration_secs,
        paused_secs: 0,
        vault_files_affected: vec![],
        action_items: vec![],
        open_questions: vec![],
        raw_db_path: Some(raw_db_path),
        raw_db_expires_at: None,
    })
}
