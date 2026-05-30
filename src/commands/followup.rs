use anyhow::Result;
use std::path::Path;

use crate::llm;

fn read_input(file: &str) -> Result<String> {
    let path = Path::new(file);
    if path.exists() {
        Ok(std::fs::read_to_string(path)?)
    } else {
        Ok(file.to_string())
    }
}

pub async fn run(file: &str, channel: Option<&str>) -> Result<()> {
    let transcript = read_input(file)?;
    let prompt = match channel {
        Some("slack") => format!("Draft a concise follow-up message for Slack based on this meeting transcript. Use Slack-friendly formatting (short paragraphs, bullet points, emoji headers):\n\n{}", transcript),
        Some("email") => format!("Draft a professional follow-up email based on this meeting transcript. Include subject line, key points, action items, and next steps:\n\n{}", transcript),
        Some(ch) => format!("Draft a follow-up message for {} based on this meeting transcript:\n\n{}", ch, transcript),
        None => format!("Draft a follow-up message based on this meeting transcript, including key points discussed, action items, and next steps:\n\n{}", transcript),
    };
    let result = llm::complete(&prompt).await?;
    println!("{}", result);
    Ok(())
}