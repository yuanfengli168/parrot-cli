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

pub async fn run(file: &str, format: &str) -> Result<()> {
    let transcript = read_input(file)?;

    // Generate TLDR
    let tldr_prompt = format!("Provide a concise TLDR (3-5 sentences) of this meeting transcript:\n\n{}", transcript);
    let tldr = llm::complete(&tldr_prompt).await?;

    // Generate action items
    let actions_prompt = format!("Extract all action items from this meeting transcript as a numbered list:\n\n{}", transcript);
    let actions = llm::complete(&actions_prompt).await?;

    // Generate decisions
    let decisions_prompt = format!("Extract all decisions made during this meeting:\n\n{}", transcript);
    let decisions = llm::complete(&decisions_prompt).await?;

    let doc = match format {
        "notebooklm" => format_notebooklm(&transcript, &tldr, &actions, &decisions),
        "obsidian" => format_obsidian(&transcript, &tldr, &actions, &decisions),
        "markdown" | _ => format_markdown(&transcript, &tldr, &actions, &decisions),
    };

    println!("{}", doc);
    Ok(())
}

fn format_notebooklm(transcript: &str, tldr: &str, actions: &str, decisions: &str) -> String {
    format!(
        "# Meeting Transcript — NotebookLM Import\n\n\
         ## TLDR\n\n{}\n\n\
         ## Action Items\n\n{}\n\n\
         ## Decisions\n\n{}\n\n\
         ## Full Transcript\n\n{}",
        tldr, actions, decisions, transcript
    )
}

fn format_obsidian(transcript: &str, tldr: &str, actions: &str, decisions: &str) -> String {
    format!(
        "---\ntags: [meeting, transcript]\n---\n\n\
         # Meeting Notes\n\n\
         ## TLDR\n> {}\n\n\
         ## Action Items\n{}\n\n\
         ## Decisions\n{}\n\n\
         ## Transcript\n{}",
        tldr, actions, decisions, transcript
    )
}

fn format_markdown(transcript: &str, tldr: &str, actions: &str, decisions: &str) -> String {
    format!(
        "# Meeting Summary\n\n\
         ## TLDR\n\n{}\n\n\
         ## Action Items\n\n{}\n\n\
         ## Decisions\n\n{}\n\n\
         ## Full Transcript\n\n{}",
        tldr, actions, decisions, transcript
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notebooklm_format() {
        let result = format_notebooklm("transcript text", "tldr here", "actions here", "decisions here");
        assert!(result.contains("# Meeting Transcript — NotebookLM Import"));
        assert!(result.contains("## TLDR\n\ntldr here"));
        assert!(result.contains("## Action Items\n\nactions here"));
        assert!(result.contains("## Decisions\n\ndecisions here"));
        assert!(result.contains("## Full Transcript\n\ntranscript text"));
    }

    #[test]
    fn obsidian_format() {
        let result = format_obsidian("transcript text", "tldr", "actions", "decisions");
        assert!(result.contains("---\ntags: [meeting, transcript]\n---"));
        assert!(result.contains("# Meeting Notes"));
        assert!(result.contains("> tldr"));
        assert!(result.contains("## Action Items\nactions"));
        assert!(result.contains("## Decisions\ndecisions"));
    }

    #[test]
    fn markdown_format() {
        let result = format_markdown("transcript text", "tldr", "actions", "decisions");
        assert!(result.contains("# Meeting Summary"));
        assert!(result.contains("## TLDR\n\ntldr"));
        assert!(result.contains("## Action Items\n\nactions"));
        assert!(result.contains("## Decisions\n\ndecisions"));
        assert!(result.contains("## Full Transcript\n\ntranscript text"));
    }

    #[test]
    fn read_input_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello from file").unwrap();
        let result = read_input(file.to_str().unwrap()).unwrap();
        assert_eq!(result, "hello from file");
    }

    #[test]
    fn read_input_as_raw_text() {
        // Non-existent paths are treated as raw text
        let result = read_input("just some raw text that isn't a file").unwrap();
        assert_eq!(result, "just some raw text that isn't a file");
    }
}