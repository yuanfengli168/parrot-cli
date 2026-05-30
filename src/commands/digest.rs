use anyhow::Result;

use crate::commands;
use crate::config;

pub async fn run(file: &str, format: &str) -> Result<()> {
    config::ensure_dirs()?;

    println!("🦜 Running full digest pipeline...\n");

    // Step 1: Transcribe
    println!("=== Step 1: Transcribing ===");
    commands::transcribe::run(file, None, true, false, None, None).await?;

    // Find transcript output
    let stem = std::path::Path::new(file)
        .file_stem().unwrap().to_str().unwrap().to_string();
    let transcript_path = config::output_dir().join(format!("{}.txt", stem));

    if !transcript_path.exists() {
        anyhow::bail!("Transcript file not found at {:?}", transcript_path);
    }

    let transcript = std::fs::read_to_string(&transcript_path)?;

    // Step 2: Summarize
    println!("\n=== Step 2: Summarizing ===");
    let summary = crate::llm::complete(&format!(
        "Provide a concise TLDR and key points for this meeting transcript:\n\n{}", transcript
    )).await?;
    println!("{}", summary);

    // Step 3: Actions
    println!("\n=== Step 3: Extracting Action Items ===");
    let actions = crate::llm::complete(&format!(
        "Extract all action items from this meeting transcript as a numbered list:\n\n{}", transcript
    )).await?;
    println!("{}", actions);

    // Step 4: Decisions
    println!("\n=== Step 4: Extracting Decisions ===");
    let decisions = crate::llm::complete(&format!(
        "Extract all decisions made during this meeting:\n\n{}", transcript
    )).await?;
    println!("{}", decisions);

    // Step 5: Generate doc
    println!("\n=== Step 5: Generating Document ===");
    let doc = match format {
        "notebooklm" => format_notebooklm(&transcript, &summary, &actions, &decisions),
        "obsidian" => format_obsidian(&transcript, &summary, &actions, &decisions),
        _ => format_markdown(&transcript, &summary, &actions, &decisions),
    };

    // Save doc
    let doc_path = config::output_dir().join(format!("{}-digest.md", stem));
    std::fs::write(&doc_path, &doc)?;
    println!("\n✅ Digest saved to {}", doc_path.display());

    Ok(())
}

fn format_notebooklm(t: &str, s: &str, a: &str, d: &str) -> String {
    format!("# Meeting Transcript — NotebookLM Import\n\n## TLDR\n\n{}\n\n## Action Items\n\n{}\n\n## Decisions\n\n{}\n\n## Full Transcript\n\n{}", s, a, d, t)
}
fn format_obsidian(t: &str, s: &str, a: &str, d: &str) -> String {
    format!("---\ntags: [meeting, digest]\n---\n\n# Meeting Digest\n\n## TLDR\n> {}\n\n## Action Items\n{}\n\n## Decisions\n{}\n\n## Transcript\n{}", s, a, d, t)
}
fn format_markdown(t: &str, s: &str, a: &str, d: &str) -> String {
    format!("# Meeting Digest\n\n## TLDR\n\n{}\n\n## Action Items\n\n{}\n\n## Decisions\n\n{}\n\n## Full Transcript\n\n{}", s, a, d, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_notebooklm_format() {
        let result = format_notebooklm("transcript", "summary", "actions", "decisions");
        assert!(result.contains("# Meeting Transcript — NotebookLM Import"));
        assert!(result.contains("summary"));
        assert!(result.contains("actions"));
        assert!(result.contains("decisions"));
        assert!(result.contains("transcript"));
    }

    #[test]
    fn digest_obsidian_format() {
        let result = format_obsidian("transcript", "summary", "actions", "decisions");
        assert!(result.contains("tags: [meeting, digest]"));
        assert!(result.contains("# Meeting Digest"));
        assert!(result.contains("> summary"));
    }

    #[test]
    fn digest_markdown_format() {
        let result = format_markdown("transcript", "summary", "actions", "decisions");
        assert!(result.contains("# Meeting Digest"));
        assert!(result.contains("## TLDR"));
        assert!(result.contains("summary"));
    }
}