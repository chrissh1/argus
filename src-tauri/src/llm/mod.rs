//! Extensible LLM provider abstraction and JSON parsing utilities.

pub mod ollama;

use crate::{ArgusError, ArgusResult};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelTag {
    pub name: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// General text completion
    async fn complete(&self, model: &str, system: Option<&str>, prompt: &str) -> ArgusResult<String>;

    /// JSON-mode structured completion
    async fn complete_json(&self, model: &str, system: &str, prompt: &str) -> ArgusResult<String>;

    /// Vector embedding generation
    async fn embed(&self, model: &str, text: &str) -> ArgusResult<Vec<f32>>;

    /// Health check and available models list
    async fn ping(&self) -> ArgusResult<Vec<ModelTag>>;
}

/// Helper function to clean markdown fences and malformed JSON common in local LLMs.
pub fn clean_json(raw: &str) -> &str {
    let mut trimmed = raw.trim();

    // Strip starting markdown code block if present
    if trimmed.starts_with("```json") {
        trimmed = trimmed.trim_start_matches("```json").trim();
    } else if trimmed.starts_with("```") {
        trimmed = trimmed.trim_start_matches("```").trim();
    }

    // Strip ending markdown fence if present
    if trimmed.ends_with("```") {
        trimmed = trimmed.trim_end_matches("```").trim();
    }

    // Find the first '{' or '[' and last '}' or ']'
    let first_brace = trimmed.find('{');
    let first_bracket = trimmed.find('[');
    let start_idx = match (first_brace, first_bracket) {
        (Some(b), Some(k)) => Some(b.min(k)),
        (Some(b), None) => Some(b),
        (None, Some(k)) => Some(k),
        (None, None) => None,
    };

    let last_brace = trimmed.rfind('}');
    let last_bracket = trimmed.rfind(']');
    let end_idx = match (last_brace, last_bracket) {
        (Some(b), Some(k)) => Some(b.max(k)),
        (Some(b), None) => Some(b),
        (None, Some(k)) => Some(k),
        (None, None) => None,
    };

    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        if start <= end && end < trimmed.len() {
            return &trimmed[start..=end];
        }
    }

    trimmed
}

/// Parse string into typed struct, attempting sanitization first.
pub fn parse_llm_json<T: DeserializeOwned>(raw: &str) -> ArgusResult<T> {
    let cleaned = clean_json(raw);
    serde_json::from_str::<T>(cleaned).map_err(|e| {
        ArgusError::Other(format!(
            "Failed to parse LLM JSON response: {e}. Raw content: '{raw}'"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Sample {
        title: String,
        count: u32,
    }

    #[test]
    fn test_clean_json_markdown_fence() {
        let input = "```json\n{\n  \"title\": \"Refactoring\",\n  \"count\": 3\n}\n```";
        let parsed: Sample = parse_llm_json(input).unwrap();
        assert_eq!(
            parsed,
            Sample {
                title: "Refactoring".into(),
                count: 3
            }
        );
    }

    #[test]
    fn test_clean_json_with_chatter() {
        let input = "Here is your JSON result:\n\n{\"title\": \"Notes\", \"count\": 5}\nHope this helps!";
        let parsed: Sample = parse_llm_json(input).unwrap();
        assert_eq!(
            parsed,
            Sample {
                title: "Notes".into(),
                count: 5
            }
        );
    }
}
