//! Integration and unit tests for Argus synthesis pipeline, Map-Reduce chunking,
//! Mock LLM provider, and Obsidian vault writer.

use argus_lib::{
    llm::{parse_llm_json, LlmClient, ModelTag},
    mock,
    synthesis::{extract_concepts_map_reduce, pair_timeline, TimelineRow},
    vault::writer,
    ArgusResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MockLlmClient {
    map_call_count: Arc<AtomicUsize>,
    reduce_call_count: Arc<AtomicUsize>,
}

impl MockLlmClient {
    fn new() -> Self {
        Self {
            map_call_count: Arc::new(AtomicUsize::new(0)),
            reduce_call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, _model: &str, _system: Option<&str>, _prompt: &str) -> ArgusResult<String> {
        Ok("Mock completion".into())
    }

    async fn complete_json(&self, _model: &str, system: &str, _prompt: &str) -> ArgusResult<String> {
        if system.contains("analyze a short segment") {
            // Map phase response
            self.map_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(r#"{
                "window_label": "Window 1/2",
                "key_activities": ["Refactored authentication middleware", "Ran cargo test"],
                "key_decisions": ["Switched to PASETO tokens"],
                "action_items": ["Add integration tests for expired tokens"],
                "open_questions": ["How to handle clock skew across distributed clusters?"]
            }"#.into())
        } else {
            // Reduce phase response
            self.reduce_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(r#"{
                "suggested_title": "Auth Architecture & Vector Search",
                "action_items": [
                    "Add integration tests for expired tokens",
                    "Update documentation"
                ],
                "open_questions": [
                    "How to handle clock skew across distributed clusters?"
                ],
                "concepts": [
                    {
                        "label": "PASETO Authentication Middleware",
                        "summary": "Migrated from standard JWT to PASETO tokens with robust expiration handling in Rust.",
                        "evidence": ["src/auth/middleware.rs"],
                        "action_items": ["Add integration tests for expired tokens"],
                        "open_questions": ["How to handle clock skew across distributed clusters?"],
                        "tags": ["auth", "rust", "security"]
                    },
                    {
                        "label": "sqlite-vec Vector Indexing",
                        "summary": "Integrated sqlite-vec virtual tables for semantic similarity searching across Obsidian notes.",
                        "evidence": ["sqlite-vec vec0 tables"],
                        "action_items": ["Verify HNSW index scaling"],
                        "open_questions": [],
                        "tags": ["sqlite", "vectors", "search"]
                    }
                ]
            }"#.into())
        }
    }

    async fn embed(&self, _model: &str, _text: &str) -> ArgusResult<Vec<f32>> {
        // Return 768-dim mock vector
        Ok(vec![0.1f32; 768])
    }

    async fn ping(&self) -> ArgusResult<Vec<ModelTag>> {
        Ok(vec![
            ModelTag {
                name: "llama3.2:latest".into(),
                size: Some(2_000_000_000),
            },
            ModelTag {
                name: "nomic-embed-text:latest".into(),
                size: Some(300_000_000),
            },
        ])
    }
}

#[test]
fn test_json_sanitization() {
    // Test markdown json block with extra chatter
    let markdown_raw = "```json\n{\n  \"label\": \"Auth Refactor\",\n  \"status\": \"done\"\n}\n```";
    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Dummy {
        label: String,
        status: String,
    }
    let parsed: Dummy = parse_llm_json(markdown_raw).expect("parsed sanitized JSON");
    assert_eq!(
        parsed,
        Dummy {
            label: "Auth Refactor".into(),
            status: "done".into()
        }
    );
}

#[test]
fn test_mock_raw_db_generation_and_pairing() {
    let tmp_dir = std::env::temp_dir().join(format!("argus_test_{}", uuid::Uuid::new_v4().simple()));
    let raw_db = tmp_dir.join("raw.db");

    // Generate 30 mins of synthetic data
    mock::generate_mock_raw_db(&raw_db, 30).expect("generated mock raw db");
    assert!(raw_db.exists());

    let timeline = pair_timeline(&raw_db).expect("paired timeline");
    assert!(!timeline.is_empty());
    assert!(timeline.len() > 30, "expected multiple OCR & audio rows");

    // Clean up
    let _ = std::fs::remove_dir_all(tmp_dir);
}

#[tokio::test]
async fn test_map_reduce_synthesis_pipeline() {
    let mock_llm = MockLlmClient::new();
    let map_counter = mock_llm.map_call_count.clone();
    let reduce_counter = mock_llm.reduce_call_count.clone();

    // Create a 30-minute synthetic timeline (rows from t=0s to t=1800s)
    let mut timeline = Vec::new();
    for t in (0..1800).step_by(30) {
        timeline.push(TimelineRow {
            t,
            screen_text: format!("Editing file at timestamp {t}s"),
            audio_text: format!("Thinking about architecture at timestamp {t}s"),
        });
    }

    let result = extract_concepts_map_reduce(&mock_llm, "mock-model", &timeline)
        .await
        .expect("extract concepts map-reduce succeeded");

    assert_eq!(
        result.suggested_title.as_deref(),
        Some("Auth Architecture & Vector Search")
    );
    assert_eq!(result.concepts.len(), 2);
    assert_eq!(result.concepts[0].label, "PASETO Authentication Middleware");
    assert_eq!(result.action_items.len(), 2);

    // Verify Map was called multiple times for 30m window chunks (> 10m each)
    let map_calls = map_counter.load(Ordering::SeqCst);
    let reduce_calls = reduce_counter.load(Ordering::SeqCst);
    assert!(map_calls >= 2, "expected >= 2 map window calls, got {map_calls}");
    assert_eq!(reduce_calls, 1, "expected exactly 1 reduce call");
}

#[test]
fn test_vault_writer_new_and_append() {
    let tmp_dir = std::env::temp_dir().join(format!("argus_vault_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let rel_path = PathBuf::from("Argus/TestNote.md");
    let session_id = "session_2026-08-31_test";

    // 1. Create new note
    let written = writer::write_new_note(
        &tmp_dir,
        &rel_path,
        "Test Note",
        "Initial session summary content.",
        &["rust".into(), "testing".into()],
        session_id,
    )
    .expect("write_new_note");

    assert!(written.exists());
    let content = std::fs::read_to_string(&written).unwrap();
    assert!(content.contains("title: \"Test Note\""));
    assert!(content.contains("argus: true"));
    assert!(content.contains("# Test Note"));

    // 2. Append to existing note
    let appended = writer::append_to_note(
        &tmp_dir,
        &rel_path,
        Some("Next Steps"),
        "- Implement vector index upgrade",
    )
    .expect("append_to_note");

    let updated_content = std::fs::read_to_string(&appended).unwrap();
    assert!(updated_content.contains("## Next Steps"));
    assert!(updated_content.contains("- Implement vector index upgrade"));

    // Clean up
    let _ = std::fs::remove_dir_all(tmp_dir);
}
