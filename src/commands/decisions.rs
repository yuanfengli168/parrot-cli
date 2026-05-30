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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_input_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("transcript.txt");
        std::fs::write(&file, "We decided to launch in Q3.").unwrap();
        let result = read_input(file.to_str().unwrap()).unwrap();
        assert_eq!(result, "We decided to launch in Q3.");
    }

    #[test]
    fn decisions_prompt() {
        let transcript = "We decided to go with Option B.";
        let prompt = format!("Extract all decisions that were made during this meeting. List each decision clearly, stripping away the discussion that led to it:\n\n{}", transcript);
        assert!(prompt.contains("decisions"));
        assert!(prompt.contains("stripping away"));
    }
}