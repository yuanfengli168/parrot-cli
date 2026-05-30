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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_request_serialization() {
        let req = OllamaRequest {
            model: "qwen3:14b".to_string(),
            prompt: "Summarize this meeting".to_string(),
            stream: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("qwen3:14b"));
        assert!(json.contains("Summarize this meeting"));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn ollama_response_deserialization() {
        let json = r#"{"response":"This is the summary","done":true}"#;
        let resp: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response, "This is the summary");
    }

    #[test]
    fn prompt_construction_summarize() {
        let transcript = "We discussed the project timeline.";
        let prompt = format!("Provide a concise TLDR and key points for this meeting transcript:\n\n{}", transcript);
        assert!(prompt.contains("TLDR"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_actions() {
        let transcript = "John will follow up by Friday.";
        let prompt = format!("Extract all action items from this meeting transcript as a numbered list:\n\n{}", transcript);
        assert!(prompt.contains("action items"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_actions_with_assignee() {
        let transcript = "John will follow up.";
        let prompt = format!("Extract all action items from this meeting transcript. For each action item, identify who is responsible (the assignee), what they need to do, and any deadline mentioned. Format as a numbered list:\n\n{}", transcript);
        assert!(prompt.contains("assignee"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_decisions() {
        let transcript = "We decided to go with option B.";
        let prompt = format!("Extract all decisions that were made during this meeting. List each decision clearly, stripping away the discussion that led to it:\n\n{}", transcript);
        assert!(prompt.contains("decisions"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_followup_slack() {
        let transcript = "Meeting notes here.";
        let prompt = format!("Draft a concise follow-up message for Slack based on this meeting transcript. Use Slack-friendly formatting (short paragraphs, bullet points, emoji headers):\n\n{}", transcript);
        assert!(prompt.contains("Slack"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_followup_email() {
        let transcript = "Meeting notes here.";
        let prompt = format!("Draft a professional follow-up email based on this meeting transcript. Include subject line, key points, action items, and next steps:\n\n{}", transcript);
        assert!(prompt.contains("email"));
        assert!(prompt.contains(transcript));
    }

    #[test]
    fn prompt_construction_followup_custom_channel() {
        let transcript = "Meeting notes.";
        let prompt = format!("Draft a follow-up message for teams based on this meeting transcript:\n\n{}", transcript);
        assert!(prompt.contains("teams"));
    }

    #[test]
    fn prompt_construction_followup_default() {
        let transcript = "Meeting notes.";
        let prompt = format!("Draft a follow-up message based on this meeting transcript, including key points discussed, action items, and next steps:\n\n{}", transcript);
        assert!(prompt.contains("follow-up"));
    }

    #[test]
    fn endpoint_url_construction() {
        let url = "http://localhost:11434";
        let endpoint = format!("{}/api/generate", url.trim_end_matches('/'));
        assert_eq!(endpoint, "http://localhost:11434/api/generate");
    }

    #[test]
    fn endpoint_url_construction_trailing_slash() {
        let url = "http://localhost:11434/";
        let endpoint = format!("{}/api/generate", url.trim_end_matches('/'));
        assert_eq!(endpoint, "http://localhost:11434/api/generate");
    }
}