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

pub async fn run(file: &str, detail: bool, sections: bool) -> Result<()> {
    let transcript = read_input(file)?;
    let prompt = if detail {
        format!("Provide a detailed, thorough summary of this meeting transcript:\n\n{}", transcript)
    } else if sections {
        format!("Summarize this meeting transcript organized by topic sections:\n\n{}", transcript)
    } else {
        format!("Provide a concise TLDR and key points for this meeting transcript:\n\n{}", transcript)
    };
    let result = llm::complete(&prompt).await?;
    println!("{}", result);
    Ok(())
}