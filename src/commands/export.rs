use anyhow::Result;
use std::path::Path;

#[allow(unused_variables)]
pub async fn run(file: &str, target: &str) -> Result<()> {
    let cfg = crate::config::load();
    
    match target {
        "obsidian" => {
            crate::mcp::obsidian::export(file, &cfg)
        }
        "notion" => {
            anyhow::bail!("Notion export not yet implemented. MCP server integration coming soon.");
        }
        "notebooklm" => {
            export_to_notebooklm(file)
        }
        "slack" => {
            anyhow::bail!("Slack export not yet implemented. MCP server integration coming soon.");
        }
        _ => {
            anyhow::bail!("Unknown export target: {}. Supported: obsidian, notion, notebooklm, slack", target);
        }
    }
}

fn export_to_notebooklm(file: &str) -> Result<()> {
    // Check prerequisites
    crate::mcp::notebooklm::check_prerequisites()?;

    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file);
    }

    let filename = path.file_name().unwrap_or_default().to_str().unwrap_or("untitled");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let notebook_title = format!("Meeting: {} - {}", today, filename);

    println!("📓 Creating NotebookLM notebook: {}", notebook_title);
    let notebook_id = crate::mcp::notebooklm::create_notebook(&notebook_title)?;
    println!("  ✅ Created notebook: {} (ID: {})", notebook_title, notebook_id);

    // Determine how to add the source
    let file_str = path.to_str().unwrap();
    
    // Check if the file is a URL (starts with http)
    if file.starts_with("http://") || file.starts_with("https://") {
        println!("  📎 Adding URL source...");
        crate::mcp::notebooklm::add_url_source(&notebook_id, file)?;
    } else if file.to_lowercase().ends_with(".pdf") {
        // For PDFs, use --file flag
        println!("  📄 Adding PDF source...");
        crate::mcp::notebooklm::add_file_source(&notebook_id, file_str)?;
    } else {
        // For text files, read content and add as text
        println!("  📝 Adding text source...");
        let content = std::fs::read_to_string(file)?;
        crate::mcp::notebooklm::add_text_source(&notebook_id, &content)?;
    }

    println!("\n✅ Exported to NotebookLM!");
    println!("   Notebook: {}", notebook_title);
    println!("   ID: {}", notebook_id);
    Ok(())
}

