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

pub async fn run(file: &str, assignee: bool) -> Result<()> {
    let transcript = read_input(file)?;
    let prompt = if assignee {
        format!("Extract all action items from this meeting transcript. For each action item, identify who is responsible (the assignee), what they need to do, and any deadline mentioned. Format as a numbered list:\n\n{}", transcript)
    } else {
        format!("Extract all action items from this meeting transcript as a numbered list:\n\n{}", transcript)
    };
    let result = llm::complete(&prompt).await?;
    println!("{}", result);
    Ok(())
}