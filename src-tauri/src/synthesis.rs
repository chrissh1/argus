//! Post-session synthesis pipeline.
//!
//! Steps (each emits a Tauri event for live progress):
//!   1. temporal_pairing — read OCR frames + audio transcripts from raw.db,
//!      align on timestamp into (t, screen_text, audio_text) tuples
//!   2. concept_extraction — chunked JSON-mode prompt to Ollama
//!   3. vault_retrieval — embed each concept, top-k against vault index
//!   4. content_generation — per concept, decide append-vs-new + body
//!   5. vault_write — atomic writes via vault::writer
//!   6. summary — update session row, emit synthesis-complete

use crate::{
    events,
    ollama::Ollama,
    paths,
    session::{self, VaultAction, VaultFileAffected},
    settings,
    state::AppState,
    vault::{index::VaultIndex, writer},
    ArgusResult,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRow {
    pub t: i64,
    pub screen_text: String,
    pub audio_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedConcept {
    pub label: String,
    pub summary: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractionPayload {
    #[serde(default)]
    concepts: Vec<ExtractedConcept>,
    #[serde(default)]
    suggested_title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppendDecision {
    #[serde(default)]
    decision: String,                 // "append" | "new"
    #[serde(default)]
    section_heading: Option<String>,  // when append
    #[serde(default)]
    new_path: Option<String>,         // when new (relative to vault root)
    #[serde(default)]
    body: String,                     // markdown to write
    #[serde(default)]
    summary: Option<String>,          // one-liner for the post-synthesis summary
}

pub async fn synthesize(
    app: AppHandle,
    state: Arc<AppState>,
    session_id: String,
) -> ArgusResult<()> {
    events::emit_step(&app, &session_id, "starting", "Preparing synthesis…", 0, 0);

    let settings = settings::get_all(&state.db)?;
    let ollama = Ollama::new(&settings.ollama_host);
    let raw_db_path = paths::session_raw_db(&session_id)?;

    // Step 1 — temporal pairing
    events::emit_step(&app, &session_id, "pairing", "Aligning OCR + audio…", 1, 6);
    let timeline = pair_timeline(&raw_db_path)?;
    tracing::info!(rows = timeline.len(), "timeline assembled");

    // Step 2 — concept extraction
    events::emit_step(
        &app,
        &session_id,
        "extracting",
        &format!("Extracting concepts from {} frames…", timeline.len()),
        2,
        6,
    );
    let extraction = extract_concepts(&ollama, &settings.ollama_model, &timeline).await?;

    if extraction.concepts.is_empty() {
        tracing::warn!("no concepts extracted; finalizing empty session");
        session::set_synthesis_results(
            &state.db,
            &session_id,
            extraction.suggested_title.as_deref(),
            &[],
            &[],
            &[],
        )?;
        events::emit_complete(&app, &session_id);
        return Ok(());
    }

    // Step 3 — vault candidate retrieval
    events::emit_step(&app, &session_id, "retrieving", "Querying vault…", 3, 6);
    let candidates_per_concept =
        retrieve_candidates(&state.vault_index, &ollama, &settings, &extraction.concepts).await?;

    // Step 4 — content generation
    events::emit_step(
        &app,
        &session_id,
        "generating",
        "Drafting notes…",
        4,
        6,
    );
    let mut decisions = Vec::with_capacity(extraction.concepts.len());
    for (concept, candidates) in extraction.concepts.iter().zip(candidates_per_concept.iter()) {
        let d = decide_append_or_new(
            &ollama,
            &settings.ollama_model,
            concept,
            candidates,
        )
        .await?;
        decisions.push((concept.clone(), d));
    }

    // Step 5 — write to vault
    events::emit_step(&app, &session_id, "writing", "Writing to vault…", 5, 6);
    let vault_path: PathBuf = settings::require_vault_path(&state.db)?.into();
    let mut files_affected: Vec<VaultFileAffected> = vec![];
    let mut action_items = Vec::<String>::new();
    let mut open_questions = Vec::<String>::new();

    for (concept, decision) in &decisions {
        action_items.extend(concept.action_items.iter().cloned());
        open_questions.extend(concept.open_questions.iter().cloned());

        match decision.decision.as_str() {
            "append" => {
                let target = candidates_per_concept
                    .iter()
                    .flatten()
                    .next()
                    .map(|c| Path::new(&c.note_path).to_path_buf());
                if let Some(target) = target {
                    let rel = target
                        .strip_prefix(&vault_path)
                        .unwrap_or(&target)
                        .to_path_buf();
                    let written = writer::append_to_note(
                        &vault_path,
                        &rel,
                        decision.section_heading.as_deref(),
                        &decision.body,
                    )?;
                    files_affected.push(VaultFileAffected {
                        path: written.to_string_lossy().to_string(),
                        action: VaultAction::Appended,
                        summary: decision.summary.clone(),
                    });
                }
            }
            _ => {
                let new_path = decision
                    .new_path
                    .clone()
                    .unwrap_or_else(|| format!("Argus/{}.md", sanitize_filename(&concept.label)));
                let rel = Path::new(&new_path).to_path_buf();
                let written = writer::write_new_note(
                    &vault_path,
                    &rel,
                    &concept.label,
                    &decision.body,
                    &concept.tags,
                    &session_id,
                )?;
                files_affected.push(VaultFileAffected {
                    path: written.to_string_lossy().to_string(),
                    action: VaultAction::Created,
                    summary: decision.summary.clone(),
                });
            }
        }
    }

    // Step 6 — finalize
    events::emit_step(&app, &session_id, "finalizing", "Saving summary…", 6, 6);
    session::set_synthesis_results(
        &state.db,
        &session_id,
        extraction.suggested_title.as_deref(),
        &dedup(action_items),
        &dedup(open_questions),
        &files_affected,
    )?;
    events::emit_complete(&app, &session_id);
    Ok(())
}

fn dedup(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

// --- Step 1 ---------------------------------------------------------------

/// Run the first SQL string that doesn't error; ignore rows that fail to deserialize.
/// Used to tolerate small Screenpipe schema variations between versions.
fn read_two_col(conn: &Connection, queries: &[&str]) -> Vec<(i64, String)> {
    for q in queries {
        let Ok(mut stmt) = conn.prepare(q) else { continue };
        let Ok(iter) = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) else {
            continue;
        };
        return iter.filter_map(Result::ok).collect();
    }
    vec![]
}

fn pair_timeline(raw_db_path: &Path) -> ArgusResult<Vec<TimelineRow>> {
    if !raw_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = Connection::open(raw_db_path)?;

    let ocr = read_two_col(&conn, &[
        "SELECT timestamp, text FROM ocr_frames ORDER BY timestamp",
        "SELECT timestamp, COALESCE(text,'') FROM ocr_text ORDER BY timestamp",
    ]);
    let audio = read_two_col(&conn, &[
        "SELECT timestamp, transcript FROM audio_chunks ORDER BY timestamp",
        "SELECT timestamp, COALESCE(transcription,'') FROM audio_transcriptions ORDER BY timestamp",
    ]);

    // Walk through audio; for each chunk, gather OCR text within ±2s.
    let window = 2;
    let mut rows = Vec::with_capacity(audio.len().max(ocr.len()));
    let mut ocr_idx = 0;
    for (t, audio_text) in &audio {
        let lo = t - window;
        let hi = t + window;
        let mut screen = String::new();
        while ocr_idx < ocr.len() && ocr[ocr_idx].0 < lo {
            ocr_idx += 1;
        }
        let mut peek = ocr_idx;
        while peek < ocr.len() && ocr[peek].0 <= hi {
            if !screen.is_empty() {
                screen.push(' ');
            }
            screen.push_str(&ocr[peek].1);
            peek += 1;
        }
        rows.push(TimelineRow {
            t: *t,
            screen_text: screen,
            audio_text: audio_text.clone(),
        });
    }
    // If no audio at all, fall back to OCR-only rows so synthesis still happens.
    if rows.is_empty() {
        for (t, txt) in ocr {
            rows.push(TimelineRow { t, screen_text: txt, audio_text: String::new() });
        }
    }
    Ok(rows)
}

// --- Step 2 ---------------------------------------------------------------

const EXTRACT_SYSTEM: &str = "You are Argus, an analyst that reads a transcript of a single deep-work \
session (paired screen OCR + voice transcript) and distills it into structured concepts for an \
Obsidian knowledge vault. Output strict JSON only.";

const EXTRACT_INSTRUCTIONS: &str = r#"Return JSON of shape:
{
  "suggested_title": "short descriptive session title",
  "concepts": [
    {
      "label": "concept name suitable as a note title",
      "summary": "2-4 sentence summary of what was learned or decided",
      "evidence": ["short verbatim snippets that support this concept"],
      "action_items": ["concrete next steps mentioned"],
      "open_questions": ["explicit unresolved questions"],
      "tags": ["lowercase-kebab tags"]
    }
  ]
}
Rules:
- Be terse and technical.
- Skip filler chatter.
- Prefer 3-6 concepts. Never zero unless the transcript is empty.
"#;

async fn extract_concepts(
    ollama: &Ollama,
    model: &str,
    timeline: &[TimelineRow],
) -> ArgusResult<ExtractionPayload> {
    if timeline.is_empty() {
        return Ok(ExtractionPayload { concepts: vec![], suggested_title: None });
    }
    let mut prompt = String::with_capacity(4096);
    prompt.push_str(EXTRACT_INSTRUCTIONS);
    prompt.push_str("\n\nTRANSCRIPT:\n");
    let mut chars = 0usize;
    let cap = 16_000; // soft cap to keep within an 8k-token ctx window
    for row in timeline {
        let line = format!(
            "[{}s] SCREEN: {}\n      AUDIO:  {}\n",
            row.t,
            row.screen_text.chars().take(280).collect::<String>(),
            row.audio_text.chars().take(280).collect::<String>(),
        );
        if chars + line.len() > cap {
            break;
        }
        chars += line.len();
        prompt.push_str(&line);
    }

    let raw = ollama.generate_json(model, EXTRACT_SYSTEM, &prompt).await?;
    let parsed: ExtractionPayload = serde_json::from_str(&raw).unwrap_or_else(|e| {
        tracing::warn!(?e, body = %raw, "extraction JSON parse failed");
        ExtractionPayload { concepts: vec![], suggested_title: None }
    });
    Ok(parsed)
}

// --- Step 3 ---------------------------------------------------------------

async fn retrieve_candidates(
    index: &VaultIndex,
    ollama: &Ollama,
    settings: &settings::Settings,
    concepts: &[ExtractedConcept],
) -> ArgusResult<Vec<Vec<crate::vault::index::CandidateChunk>>> {
    let mut all = Vec::with_capacity(concepts.len());
    for c in concepts {
        let query = format!("{}\n\n{}", c.label, c.summary);
        let emb = match ollama.embed(&settings.embed_model, &query).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(?e, "embed failed; treating as no candidates");
                all.push(vec![]);
                continue;
            }
        };
        let mut cands = index.topk(&emb, 5)?;
        cands.retain(|c| c.similarity >= settings.similarity_threshold);
        all.push(cands);
    }
    Ok(all)
}

// --- Step 4 ---------------------------------------------------------------

const DECIDE_SYSTEM: &str = "You decide whether to append new content into the most-related existing \
Obsidian note or create a brand new note. Output strict JSON only.";

async fn decide_append_or_new(
    ollama: &Ollama,
    model: &str,
    concept: &ExtractedConcept,
    candidates: &[crate::vault::index::CandidateChunk],
) -> ArgusResult<AppendDecision> {
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "CONCEPT:\nlabel: {}\nsummary: {}\ntags: {:?}\naction_items: {:?}\nopen_questions: {:?}\n\n",
        concept.label, concept.summary, concept.tags, concept.action_items, concept.open_questions
    ));
    prompt.push_str("CANDIDATES (most similar existing notes):\n");
    for (i, c) in candidates.iter().enumerate() {
        prompt.push_str(&format!(
            "[{i}] {} (similarity {:.2})\n---\n{}\n---\n",
            c.note_path,
            c.similarity,
            c.chunk_text.chars().take(1200).collect::<String>()
        ));
    }
    if candidates.is_empty() {
        prompt.push_str("(no candidates above threshold)\n");
    }
    prompt.push_str(
        r#"
Return JSON:
{
  "decision": "append" | "new",
  "section_heading": "H2/H3 heading to append under, if append",
  "new_path": "relative/path/Note.md, if new (default to Argus/<label>.md)",
  "body": "markdown body to write; use [[wikilinks]] freely",
  "summary": "one-line summary of what was written"
}
"#,
    );
    let raw = ollama.generate_json(model, DECIDE_SYSTEM, &prompt).await?;
    let parsed: AppendDecision = serde_json::from_str(&raw).unwrap_or_else(|_| AppendDecision {
        decision: "new".into(),
        section_heading: None,
        new_path: None,
        body: format!("{}\n\n_Auto-imported by Argus._", concept.summary),
        summary: Some(concept.summary.clone()),
    });
    Ok(parsed)
}
