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

pub async fn run(file: &str) -> Result<()> {
    let transcript = read_input(file)?;
    let prompt = format!("Extract all decisions that were made during this meeting. List each decision clearly, stripping away the discussion that led to it:\n\n{}", transcript);
    let result = llm::complete(&prompt).await?;
    println!("{}", result);
    Ok(())
}