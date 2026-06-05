//! Thin HTTP client for the local Ollama server.
//!
//! Endpoints used:
//!   * `GET  /api/tags`        — list installed models, also serves as a ping
//!   * `POST /api/generate`    — single-shot completion (we use streaming=false)
//!   * `POST /api/embeddings`  — vector embeddings via `nomic-embed-text`

use crate::ArgusResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Ollama {
    client: Client,
    base: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelTag {
    pub name: String,
    #[serde(default)]
    pub size: Option<u64>,
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

impl Ollama {
    pub fn new(base: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .expect("reqwest client");
        Ollama { client, base: base.into() }
    }

    pub async fn ping(&self) -> ArgusResult<Vec<ModelTag>> {
        let url = format!("{}/api/tags", self.base.trim_end_matches('/'));
        let r = self.client.get(url).send().await?.error_for_status()?;
        let body: TagsResponse = r.json().await?;
        Ok(body.models)
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> ArgusResult<String> {
        self.generate_inner(model, prompt, None, None).await
    }

    pub async fn generate_json(
        &self,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> ArgusResult<String> {
        self.generate_inner(model, prompt, Some(system), Some("json")).await
    }

    async fn generate_inner(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
        format: Option<&str>,
    ) -> ArgusResult<String> {
        let url = format!("{}/api/generate", self.base.trim_end_matches('/'));
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
            format,
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

    pub async fn embed(&self, model: &str, text: &str) -> ArgusResult<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base.trim_end_matches('/'));
        let req = EmbedRequest { model, prompt: text };
        let body: EmbedResponse = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(body.embedding)
    }
}
