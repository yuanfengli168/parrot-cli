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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_input_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("transcript.txt");
        std::fs::write(&file, "meeting content here").unwrap();
        let result = read_input(file.to_str().unwrap()).unwrap();
        assert_eq!(result, "meeting content here");
    }

    #[test]
    fn read_input_raw_text() {
        let result = read_input("just raw text").unwrap();
        assert_eq!(result, "just raw text");
    }

    #[test]
    fn summarize_prompt_default() {
        let transcript = "We discussed Q3 targets.";
        let prompt = format!("Provide a concise TLDR and key points for this meeting transcript:\n\n{}", transcript);
        assert!(prompt.contains("TLDR"));
        assert!(prompt.contains("key points"));
    }

    #[test]
    fn summarize_prompt_detail() {
        let transcript = "We discussed Q3 targets.";
        let prompt = format!("Provide a detailed, thorough summary of this meeting transcript:\n\n{}", transcript);
        assert!(prompt.contains("detailed"));
        assert!(prompt.contains("thorough"));
    }

    #[test]
    fn summarize_prompt_sections() {
        let transcript = "We discussed Q3 targets.";
        let prompt = format!("Summarize this meeting transcript organized by topic sections:\n\n{}", transcript);
        assert!(prompt.contains("topic sections"));
    }
}