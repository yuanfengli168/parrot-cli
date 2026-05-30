use anyhow::Result;

pub async fn run(file: &str, target: &str) -> Result<()> {
    let cfg = crate::config::load();
    
    match target {
        "obsidian" => {
            let vault_path = cfg.mcp.servers.get("obsidian")
                .and_then(|v| v.get("vault_path"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Obsidian MCP not configured. Run: parrot-cli mcp add obsidian"))?;
            
            let content = std::fs::read_to_string(file)?;
            let filename = std::path::Path::new(file)
                .file_name().unwrap().to_str().unwrap();
            let dest = format!("{}/{}.md", vault_path.trim_end_matches('/'), filename.trim_end_matches(".txt").trim_end_matches(".md"));
            std::fs::write(&dest, &content)?;
            println!("✅ Exported to Obsidian: {}", dest);
        }
        "notion" => {
            anyhow::bail!("Notion export not yet implemented. MCP server integration coming soon.");
        }
        "notebooklm" => {
            println!("📝 For NotebookLM: Use the generated doc file and import it manually into NotebookLM.");
            println!("   The doc format is already optimized for NotebookLM import.");
        }
        "slack" => {
            anyhow::bail!("Slack export not yet implemented. MCP server integration coming soon.");
        }
        _ => {
            anyhow::bail!("Unknown export target: {}. Supported: obsidian, notion, notebooklm, slack", target);
        }
    }
    Ok(())
}