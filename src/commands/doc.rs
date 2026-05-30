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