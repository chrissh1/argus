//! Post-session Map-Reduce synthesis pipeline.
//!
//! Steps (each emits a Tauri event for live progress):
//!   1. temporal_pairing — read OCR frames + audio transcripts from raw.db,
//!      align on timestamp into (t, screen_text, audio_text) tuples.
//!   2. map_reduce_extraction — slice long sessions into chronological windows,
//!      map each window to micro-summaries, then reduce into consolidated concepts,
//!      action items, and open questions without context truncation.
//!   3. vault_retrieval — embed each concept, top-k against vault index via sqlite-vec.
//!   4. content_generation — per concept, decide append-vs-new + Markdown body.
//!   5. vault_write — atomic writes via vault::writer.
//!   6. summary — update session row in app.db, emit synthesis-complete event.

use crate::{
    events,
    llm::{self, ollama::OllamaClient, LlmClient},
    paths,
    session::{self, VaultAction, VaultFileAffected},
    settings,
    state::AppState,
    vault::{index::VaultIndex, writer},
    ArgusError, ArgusResult,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionPayload {
    #[serde(default)]
    pub concepts: Vec<ExtractedConcept>,
    #[serde(default)]
    pub suggested_title: Option<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowSummary {
    #[serde(default)]
    pub window_label: String,
    #[serde(default)]
    pub key_activities: Vec<String>,
    #[serde(default)]
    pub key_decisions: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AppendDecision {
    #[serde(default)]
    decision: String, // "append" | "new"
    #[serde(default)]
    section_heading: Option<String>, // when append
    #[serde(default)]
    new_path: Option<String>, // when new (relative to vault root)
    #[serde(default)]
    body: String, // markdown to write
    #[serde(default)]
    summary: Option<String>, // one-liner for the post-synthesis summary
}

pub async fn synthesize(
    app: AppHandle,
    state: Arc<AppState>,
    session_id: String,
) -> ArgusResult<()> {
    events::emit_step(&app, &session_id, "starting", "Preparing synthesis…", 0, 6);

    let settings = settings::get_all(&state.db)?;
    let host = settings
        .ollama_host
        .as_deref()
        .ok_or_else(|| ArgusError::Other("Cannot synthesize: Ollama host not configured".into()))?;
    let model = settings
        .ollama_model
        .as_deref()
        .ok_or_else(|| ArgusError::Other("Cannot synthesize: Inference model not configured".into()))?;

    let llm_client: Arc<dyn LlmClient> = Arc::new(OllamaClient::new(host));
    let raw_db_path = paths::session_raw_db(&session_id)?;

    // Step 1 — temporal pairing
    events::emit_step(&app, &session_id, "pairing", "Aligning OCR + audio…", 1, 6);
    let timeline = pair_timeline(&raw_db_path)?;
    tracing::info!(rows = timeline.len(), "timeline assembled");

    // Step 2 — map-reduce concept extraction
    events::emit_step(
        &app,
        &session_id,
        "extracting",
        &format!("Distilling {} frames via Map-Reduce…", timeline.len()),
        2,
        6,
    );
    let extraction = extract_concepts_map_reduce(
        llm_client.as_ref(),
        model,
        &timeline,
    )
    .await?;

    if extraction.concepts.is_empty() {
        tracing::warn!("no concepts extracted; finalizing session");
        session::set_synthesis_results(
            &state.db,
            &session_id,
            extraction.suggested_title.as_deref(),
            &extraction.action_items,
            &extraction.open_questions,
            &[],
        )?;
        events::emit_complete(&app, &session_id);
        return Ok(());
    }

    // Step 3 — vault candidate retrieval
    events::emit_step(&app, &session_id, "retrieving", "Querying vault index…", 3, 6);
    let candidates_per_concept = retrieve_candidates(
        &state.vault_index,
        llm_client.as_ref(),
        &settings,
        &extraction.concepts,
    )
    .await?;

    // Step 4 — content generation
    events::emit_step(&app, &session_id, "generating", "Drafting Obsidian notes…", 4, 6);
    let mut decisions = Vec::with_capacity(extraction.concepts.len());
    for (concept, candidates) in extraction.concepts.iter().zip(candidates_per_concept.iter()) {
        let d = decide_append_or_new(
            llm_client.as_ref(),
            model,
            concept,
            candidates,
        )
        .await?;
        decisions.push((concept.clone(), d));
    }

    // Step 5 — write to vault
    events::emit_step(&app, &session_id, "writing", "Writing notes to vault…", 5, 6);
    let vault_path: PathBuf = match settings::require_vault_path(&state.db) {
        Ok(p) => p.into(),
        Err(_) => {
            tracing::warn!("no vault path configured; saving summary only");
            paths::argus_root()?.join("notes")
        }
    };

    let mut files_affected: Vec<VaultFileAffected> = vec![];
    let mut all_action_items = extraction.action_items.clone();
    let mut all_open_questions = extraction.open_questions.clone();

    for (concept, decision) in &decisions {
        all_action_items.extend(concept.action_items.iter().cloned());
        all_open_questions.extend(concept.open_questions.iter().cloned());

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
                    let note_summary = decision
                        .summary
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .or_else(|| Some(concept.summary.clone()));
                    files_affected.push(VaultFileAffected {
                        path: written.to_string_lossy().to_string(),
                        action: VaultAction::Appended,
                        summary: note_summary,
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
                let note_summary = decision
                    .summary
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .or_else(|| Some(concept.summary.clone()));
                files_affected.push(VaultFileAffected {
                    path: written.to_string_lossy().to_string(),
                    action: VaultAction::Created,
                    summary: note_summary,
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
        &dedup(all_action_items),
        &dedup(all_open_questions),
        &files_affected,
    )?;
    events::emit_complete(&app, &session_id);
    Ok(())
}

fn dedup(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v.retain(|s| !s.trim().is_empty());
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

// --- Step 1: Temporal Pairing ---------------------------------------------

fn read_two_col(conn: &Connection, queries: &[&str]) -> Vec<(i64, String)> {
    for q in queries {
        let Ok(mut stmt) = conn.prepare(q) else { continue };
        let Ok(iter) = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        else {
            continue;
        };
        return iter.filter_map(Result::ok).collect();
    }
    vec![]
}

pub fn pair_timeline(raw_db_path: &Path) -> ArgusResult<Vec<TimelineRow>> {
    if !raw_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = Connection::open(raw_db_path)?;

    let ocr = read_two_col(
        &conn,
        &[
            "SELECT timestamp, text FROM ocr_frames ORDER BY timestamp",
            "SELECT timestamp, COALESCE(text,'') FROM ocr_text ORDER BY timestamp",
        ],
    );
    let audio = read_two_col(
        &conn,
        &[
            "SELECT timestamp, transcript FROM audio_chunks ORDER BY timestamp",
            "SELECT timestamp, COALESCE(transcription,'') FROM audio_transcriptions ORDER BY timestamp",
        ],
    );

    let window = 2i64;
    let mut rows = Vec::with_capacity(audio.len() + ocr.len());
    let mut i = 0;
    let mut j = 0;

    while i < ocr.len() || j < audio.len() {
        if i < ocr.len() && j < audio.len() {
            let ocr_t = ocr[i].0;
            let audio_t = audio[j].0;
            if (ocr_t - audio_t).abs() <= window {
                rows.push(TimelineRow {
                    t: ocr_t,
                    screen_text: ocr[i].1.clone(),
                    audio_text: audio[j].1.clone(),
                });
                i += 1;
                j += 1;
            } else if ocr_t < audio_t {
                rows.push(TimelineRow {
                    t: ocr_t,
                    screen_text: ocr[i].1.clone(),
                    audio_text: String::new(),
                });
                i += 1;
            } else {
                rows.push(TimelineRow {
                    t: audio_t,
                    screen_text: String::new(),
                    audio_text: audio[j].1.clone(),
                });
                j += 1;
            }
        } else if i < ocr.len() {
            rows.push(TimelineRow {
                t: ocr[i].0,
                screen_text: ocr[i].1.clone(),
                audio_text: String::new(),
            });
            i += 1;
        } else {
            rows.push(TimelineRow {
                t: audio[j].0,
                screen_text: String::new(),
                audio_text: audio[j].1.clone(),
            });
            j += 1;
        }
    }
    Ok(rows)
}

// --- Step 2: Map-Reduce Concept Extraction --------------------------------

const MAP_SYSTEM: &str = "You analyze a short segment of deep-work screen OCR and audio. \
Extract concise key activities, decisions, action items, and open questions. Output strict JSON only.";

const MAP_INSTRUCTIONS: &str = r#"Return JSON:
{
  "window_label": "e.g. Minutes 0-10",
  "key_activities": ["activity 1", "activity 2"],
  "key_decisions": ["architectural or technical decision made"],
  "action_items": ["concrete next steps mentioned"],
  "open_questions": ["explicit unresolved questions"]
}
"#;

const REDUCE_SYSTEM: &str = "You are Argus, an analyst synthesizing structured knowledge from multiple \
deep-work segment summaries into cohesive concepts and Obsidian notes. Output strict JSON only.";

const REDUCE_INSTRUCTIONS: &str = r#"Return JSON:
{
  "suggested_title": "short descriptive session title",
  "action_items": ["consolidated unique next steps"],
  "open_questions": ["consolidated unresolved questions"],
  "concepts": [
    {
      "label": "concept name suitable as an Obsidian note title",
      "summary": "2-4 sentence technical summary of what was learned, designed, or refactored",
      "evidence": ["key code references, tools, or decisions"],
      "action_items": ["specific action items for this concept"],
      "open_questions": ["specific open questions for this concept"],
      "tags": ["lowercase-kebab-tags"]
    }
  ]
}
Rules:
- Prefer 2-5 high-signal concepts.
- Be terse, precise, and technical.
"#;

pub async fn extract_concepts_map_reduce(
    client: &dyn LlmClient,
    model: &str,
    timeline: &[TimelineRow],
) -> ArgusResult<ExtractionPayload> {
    if timeline.is_empty() {
        return Ok(ExtractionPayload::default());
    }

    // Determine window slicing strategy:
    // If timeline spans > 10 minutes (600s) or has > 35 rows, use Map-Reduce chunking.
    let duration_secs = timeline.last().map(|r| r.t).unwrap_or(0) - timeline.first().map(|r| r.t).unwrap_or(0);
    let is_long_session = duration_secs > 600 || timeline.len() > 35;

    let window_summaries = if is_long_session {
        // Slice into ~10 minute windows
        let window_size_secs = 600;
        let mut windows: Vec<Vec<TimelineRow>> = Vec::new();
        let mut current_window: Vec<TimelineRow> = Vec::new();
        let mut current_window_start = timeline[0].t;

        for row in timeline {
            if row.t - current_window_start > window_size_secs && !current_window.is_empty() {
                windows.push(current_window);
                current_window = Vec::new();
                current_window_start = row.t;
            }
            current_window.push(row.clone());
        }
        if !current_window.is_empty() {
            windows.push(current_window);
        }

        tracing::info!(total_windows = windows.len(), "running map phase over session windows");
        let mut summaries = Vec::with_capacity(windows.len());
        for (i, win) in windows.iter().enumerate() {
            let win_label = format!("Window {}/{} (T+{}m)", i + 1, windows.len(), (win[0].t / 60));
            let summary = map_window(client, model, win, &win_label).await?;
            summaries.push(summary);
        }
        summaries
    } else {
        // Single window for short session
        let summary = map_window(client, model, timeline, "Session Overview").await?;
        vec![summary]
    };

    // Reduce Phase: Consolidate window summaries into full concepts
    reduce_windows(client, model, &window_summaries).await
}

async fn map_window(
    client: &dyn LlmClient,
    model: &str,
    rows: &[TimelineRow],
    window_label: &str,
) -> ArgusResult<WindowSummary> {
    let mut prompt = String::with_capacity(2048);
    prompt.push_str(MAP_INSTRUCTIONS);
    prompt.push_str(&format!("\n\nWINDOW: {window_label}\nTRANSCRIPT ROWS:\n"));

    for row in rows {
        let line = format!(
            "[{}s] SCREEN: {}\n      AUDIO:  {}\n",
            row.t,
            row.screen_text.chars().take(240).collect::<String>(),
            row.audio_text.chars().take(240).collect::<String>(),
        );
        prompt.push_str(&line);
    }

    let raw = client.complete_json(model, MAP_SYSTEM, &prompt).await?;
    let parsed: WindowSummary = llm::parse_llm_json(&raw).unwrap_or_else(|e| {
        tracing::warn!(?e, body = %raw, "window map JSON parse fallback");
        WindowSummary {
            window_label: window_label.to_string(),
            key_activities: vec!["Working on session tasks".into()],
            key_decisions: vec![],
            action_items: vec![],
            open_questions: vec![],
        }
    });
    Ok(parsed)
}

async fn reduce_windows(
    client: &dyn LlmClient,
    model: &str,
    summaries: &[WindowSummary],
) -> ArgusResult<ExtractionPayload> {
    let mut prompt = String::with_capacity(4096);
    prompt.push_str(REDUCE_INSTRUCTIONS);
    prompt.push_str("\n\nCHRONOLOGICAL WINDOW SUMMARIES:\n");

    for s in summaries {
        prompt.push_str(&format!(
            "### {}\n- Activities: {}\n- Decisions: {}\n- Action Items: {}\n- Open Questions: {}\n\n",
            s.window_label,
            s.key_activities.join("; "),
            s.key_decisions.join("; "),
            s.action_items.join("; "),
            s.open_questions.join("; ")
        ));
    }

    let raw = client.complete_json(model, REDUCE_SYSTEM, &prompt).await?;
    let parsed: ExtractionPayload = llm::parse_llm_json(&raw).unwrap_or_else(|e| {
        tracing::warn!(?e, body = %raw, "reduce JSON parse fallback");
        ExtractionPayload {
            suggested_title: Some("Deep Work Session".into()),
            concepts: summaries
                .iter()
                .flat_map(|s| s.key_activities.iter())
                .take(3)
                .map(|act| ExtractedConcept {
                    label: act.clone(),
                    summary: format!("Work completed on: {act}"),
                    evidence: vec![],
                    action_items: vec![],
                    open_questions: vec![],
                    tags: vec!["argus".into()],
                })
                .collect(),
            action_items: summaries.iter().flat_map(|s| s.action_items.clone()).collect(),
            open_questions: summaries.iter().flat_map(|s| s.open_questions.clone()).collect(),
        }
    });

    Ok(parsed)
}

// --- Step 3: Vault Candidate Retrieval -----------------------------------

async fn retrieve_candidates(
    index: &VaultIndex,
    client: &dyn LlmClient,
    settings: &settings::Settings,
    concepts: &[ExtractedConcept],
) -> ArgusResult<Vec<Vec<crate::vault::index::CandidateChunk>>> {
    let mut all = Vec::with_capacity(concepts.len());
    for c in concepts {
        let query = format!("{}\n\n{}", c.label, c.summary);
        let emb = match settings.embed_model.as_deref() {
            Some(embed_m) => match client.embed(embed_m, &query).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(?e, "embed failed; treating as no candidates");
                    all.push(vec![]);
                    continue;
                }
            },
            None => {
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

// --- Step 4: Content Generation & Decision --------------------------------

const DECIDE_SYSTEM: &str = "You decide whether to append new content into the most-related existing \
Obsidian note or create a brand new note. Output strict JSON only.";

async fn decide_append_or_new(
    client: &dyn LlmClient,
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
    let raw = client.complete_json(model, DECIDE_SYSTEM, &prompt).await?;
    let parsed: AppendDecision = llm::parse_llm_json(&raw).unwrap_or_else(|_| AppendDecision {
        decision: "new".into(),
        section_heading: None,
        new_path: None,
        body: format!("{}\n\n_Auto-imported by Argus._", concept.summary),
        summary: Some(concept.summary.clone()),
    });
    Ok(parsed)
}
