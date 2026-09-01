//! Ollama implementation of the LlmClient trait.

use super::{LlmClient, ModelTag};
use crate::{ArgusError, ArgusResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagsResponse {
    models: Vec<ModelTag>,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

impl OllamaClient {
    pub fn new(base: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .expect("reqwest client");
        OllamaClient {
            client,
            base: base.into(),
        }
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn ping(&self) -> ArgusResult<Vec<ModelTag>> {
        let url = format!("{}/api/tags", self.base.trim_end_matches('/'));
        let r = self.client.get(url).send().await?.error_for_status()?;
        let body: TagsResponse = r.json().await?;
        Ok(body.models)
    }

    async fn complete(&self, model: &str, system: Option<&str>, prompt: &str) -> ArgusResult<String> {
        let url = format!("{}/api/generate", self.base.trim_end_matches('/'));
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
            format: None,
            system,
            options: Some(GenerateOptions {
                temperature: Some(0.2),
                num_ctx: Some(8192),
            }),
        };
        let body: GenerateResponse = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(body.response)
    }

    async fn complete_json(&self, model: &str, system: &str, prompt: &str) -> ArgusResult<String> {
        let url = format!("{}/api/generate", self.base.trim_end_matches('/'));
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
            format: Some("json"),
            system: Some(system),
            options: Some(GenerateOptions {
                temperature: Some(0.1),
                num_ctx: Some(8192),
            }),
        };
        let res = self.client.post(url).json(&req).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ArgusError::Other(format!(
                "Ollama generate_json error ({status}): {text}"
            )));
        }
        let body: GenerateResponse = res.json().await?;
        Ok(body.response)
    }

    async fn embed(&self, model: &str, text: &str) -> ArgusResult<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base.trim_end_matches('/'));
        let req = EmbedRequest { model, prompt: text };
        let res = self.client.post(url).json(&req).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ArgusError::Other(format!(
                "Ollama embeddings error ({status}): {text}"
            )));
        }
        let body: EmbedResponse = res.json().await?;
        Ok(body.embedding)
    }
}
