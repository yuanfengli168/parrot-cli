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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_input_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("transcript.txt");
        std::fs::write(&file, "John will send the report.").unwrap();
        let result = read_input(file.to_str().unwrap()).unwrap();
        assert_eq!(result, "John will send the report.");
    }

    #[test]
    fn actions_prompt_default() {
        let transcript = "John will follow up by Friday.";
        let prompt = format!("Extract all action items from this meeting transcript as a numbered list:\n\n{}", transcript);
        assert!(prompt.contains("action items"));
        assert!(prompt.contains("numbered list"));
    }

    #[test]
    fn actions_prompt_with_assignee() {
        let transcript = "John will follow up.";
        let prompt = format!("Extract all action items from this meeting transcript. For each action item, identify who is responsible (the assignee), what they need to do, and any deadline mentioned. Format as a numbered list:\n\n{}", transcript);
        assert!(prompt.contains("assignee"));
        assert!(prompt.contains("deadline"));
    }
}