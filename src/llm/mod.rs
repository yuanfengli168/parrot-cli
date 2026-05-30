use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn complete(prompt: &str) -> Result<String> {
    let cfg = config::load();

    match cfg.llm.provider.as_str() {
        "ollama" => ollama_complete(prompt, &cfg.llm.model, &cfg.llm.ollama_url).await,
        _ => anyhow::bail!("Unsupported LLM provider: {}", cfg.llm.provider),
    }
}

async fn ollama_complete(prompt: &str, model: &str, url: &str) -> Result<String> {
    let client = Client::new();
    let endpoint = format!("{}/api/generate", url.trim_end_matches('/'));

    println!("🤖 Asking {}...", model);

    let body = OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let resp = client.post(&endpoint)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama request failed ({}): {}", status, text);
    }

    let result: OllamaResponse = resp.json().await?;
    Ok(result.response)
}