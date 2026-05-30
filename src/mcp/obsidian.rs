use anyhow::Result;
use serde_json::json;
use std::path::Path;

use crate::config;

/// Interactive setup for Obsidian MCP server
pub fn add(cfg: &mut config::ParrotConfig, advanced: bool, vault_path: Option<&str>) -> Result<()> {
    println!("🦜 Setting up Obsidian integration");
    println!();

    let vault_path = match vault_path {
        Some(p) => p.to_string(),
        None => {
            // Interactive prompt
            println!("Enter the path to your Obsidian vault:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if vault_path.is_empty() {
        anyhow::bail!("Vault path cannot be empty.");
    }

    // Validate path exists
    let vault = Path::new(&vault_path);
    if !vault.exists() {
        anyhow::bail!("Path does not exist: {}", vault_path);
    }

    // Validate it's a directory
    if !vault.is_dir() {
        anyhow::bail!("Path is not a directory: {}", vault_path);
    }

    // Validate .obsidian folder exists
    let obsidian_dir = vault.join(".obsidian");
    if !obsidian_dir.exists() {
        anyhow::bail!(
            "Path does not appear to be an Obsidian vault (no .obsidian folder found): {}\n\
             Please verify the path points to your Obsidian vault root.",
            vault_path
        );
    }

    // Save to config
    let server_config = json!({
        "vault_path": vault_path,
        "advanced": advanced
    });

    cfg.mcp.servers.insert("obsidian".to_string(), server_config);
    config::save(cfg)?;

    println!();
    println!("✅ Obsidian MCP server configured!");
    println!("   Vault path: {}", vault_path);

    if advanced {
        println!();
        println!("📋 Advanced mode enabled — for search/tag management capabilities,");
        println!("   install the Obsidian MCP server:");
        println!();
        println!("   npx -y obsidian-mcp {}", vault_path);
        println!();
        println!("   This enables: search notes, manage tags, backlink queries, and more.");
        println!("   Without it, parrot-cli will use direct file writes (which still work for exports).");
    }

    Ok(())
}

/// Test Obsidian MCP server configuration
pub fn test(cfg: &config::ParrotConfig) -> Result<()> {
    println!("🔍 Testing Obsidian MCP server...");

    let server_config = cfg.mcp.servers.get("obsidian")
        .ok_or_else(|| anyhow::anyhow!("Obsidian not configured. Run: parrot-cli mcp add obsidian"))?;

    let vault_path = server_config.get("vault_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("vault_path not set in Obsidian config. Run: parrot-cli mcp add obsidian"))?;

    // Check vault path exists
    let vault = Path::new(vault_path);
    if !vault.exists() {
        println!("  ❌ Vault path does not exist: {}", vault_path);
        anyhow::bail!("Vault path not accessible");
    }
    println!("  ✅ Vault path exists: {}", vault_path);

    // Check it's a directory
    if !vault.is_dir() {
        println!("  ❌ Vault path is not a directory");
        anyhow::bail!("Vault path is not a directory");
    }
    println!("  ✅ Vault path is a directory");

    // Check .obsidian folder
    let obsidian_dir = vault.join(".obsidian");
    if obsidian_dir.exists() {
        println!("  ✅ .obsidian folder found");
    } else {
        println!("  ⚠️  .obsidian folder not found (path may not be a real Obsidian vault)");
    }

    // Check Meetings directory
    let meetings_dir = vault.join("Meetings");
    if meetings_dir.exists() {
        println!("  ✅ Meetings directory exists");
    } else {
        println!("  ℹ️  Meetings directory will be created on first export");
    }

    // Check advanced mode
    let is_advanced = server_config.get("advanced")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_advanced {
        println!("  ℹ️  Advanced mode enabled — checking obsidian-mcp...");
        // Check if npx is available
        let npm_check = std::process::Command::new("npx")
            .arg("--version")
            .output();
        match npm_check {
            Ok(output) if output.status.success() => {
                println!("  ✅ npx is available");
                // Try to check if obsidian-mcp is available
                let mcp_check = std::process::Command::new("npx")
                    .args(["-y", "obsidian-mcp", "--help"])
                    .output();
                match mcp_check {
                    Ok(output) if output.status.success() => {
                        println!("  ✅ obsidian-mcp is available");
                    }
                    _ => {
                        println!("  ⚠️  obsidian-mcp not found. Install with: npx -y obsidian-mcp {}", vault_path);
                    }
                }
            }
            _ => {
                println!("  ⚠️  npx not found. Install Node.js to use obsidian-mcp");
            }
        }
    }

    println!();
    println!("✅ Obsidian MCP server is ready!");
    Ok(())
}

/// Get status string for Obsidian in `mcp list`
pub fn status(server_config: Option<&serde_json::Value>) -> String {
    match server_config {
        Some(config) => {
            let vault_path = config.get("vault_path")
                .and_then(|v| v.as_str())
                .unwrap_or("(not set)");

            if vault_path.is_empty() || vault_path == "(not set)" {
                return "⚠️  vault_path not set".to_string();
            }

            let path = Path::new(vault_path);
            if path.exists() && path.join(".obsidian").exists() {
                let advanced = config.get("advanced")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if advanced {
                    format!("✅ configured (advanced) → {}", vault_path)
                } else {
                    format!("✅ configured → {}", vault_path)
                }
            } else if path.exists() {
                format!("⚠️  path exists but no .obsidian folder → {}", vault_path)
            } else {
                format!("❌ vault path not found → {}", vault_path)
            }
        }
        None => "⬜ not configured".to_string(),
    }
}

/// Export a file to Obsidian vault
pub fn export(file: &str, cfg: &config::ParrotConfig) -> Result<()> {
    let server_config = cfg.mcp.servers.get("obsidian")
        .ok_or_else(|| anyhow::anyhow!("Obsidian MCP not configured. Run: parrot-cli mcp add obsidian"))?;

    let vault_path = server_config.get("vault_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("vault_path not set in Obsidian config. Run: parrot-cli mcp add obsidian"))?;

    let vault = Path::new(vault_path);
    if !vault.exists() {
        anyhow::bail!("Vault path does not exist: {}", vault_path);
    }

    // Read input file
    let input_path = Path::new(file);
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", file);
    }
    let content = std::fs::read_to_string(input_path)?;

    // Get filename without extension
    let filename = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let source_name = input_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Get today's date
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Create Meetings directory if it doesn't exist
    let meetings_dir = vault.join("Meetings");
    std::fs::create_dir_all(&meetings_dir)?;

    // Build the Obsidian note
    let note = build_obsidian_note(&today, filename, source_name, &content);

    // Write the file
    let output_filename = format!("{}-{}.md", today, filename);
    let output_path = meetings_dir.join(&output_filename);
    std::fs::write(&output_path, &note)?;

    println!("✅ Exported to Obsidian: {}", output_path.display());
    Ok(())
}

/// Build an Obsidian-flavored note with frontmatter
fn build_obsidian_note(date: &str, title: &str, source: &str, content: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");

    // Parse content sections if they exist (markdown with ## headers)
    let (tldr, transcript, action_items, decisions) = parse_sections(content);

    let mut note = String::new();

    // YAML frontmatter
    note.push_str("---\n");
    note.push_str(&format!("date: {}\n", date));
    note.push_str("type: meeting\n");
    note.push_str("tags:\n");
    note.push_str("  - meeting\n");
    note.push_str("  - action-item\n");
    note.push_str(&format!("source: {}\n", source));
    note.push_str(&format!("generated-by: parrot-cli {}\n", version));
    note.push_str("---\n\n");

    // Title
    note.push_str(&format!("# Meeting: {} - {}\n\n", date, title));

    // TLDR
    note.push_str("## TLDR\n\n");
    if tldr.is_empty() {
        // If no TLDR section found, use the first few lines of content as summary
        let summary_lines: Vec<&str> = content.lines().take(5).collect();
        note.push_str(&summary_lines.join("\n"));
    } else {
        note.push_str(&tldr);
    }
    note.push_str("\n\n");

    // Transcript
    note.push_str("## Transcript\n\n");
    note.push_str(&transcript);
    note.push_str("\n\n");

    // Action Items
    if !action_items.is_empty() {
        note.push_str("## Action Items\n\n");
        note.push_str(&action_items);
        note.push_str("\n\n");
    }

    // Decisions
    if !decisions.is_empty() {
        note.push_str("## Decisions\n\n");
        note.push_str(&decisions);
        note.push_str("\n\n");
    }

    note
}

/// Parse markdown content into sections based on ## headers
fn parse_sections(content: &str) -> (String, String, String, String) {
    let mut tldr = String::new();
    let mut transcript = String::new();
    let mut action_items = String::new();
    let mut decisions = String::new();

    let mut current_section = &mut transcript; // default: everything goes to transcript

    for line in content.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("## ") {
            let header = trimmed.trim_start_matches("## ").trim();
            current_section = match header {
                "tldr" | "summary" | "tl;dr" => &mut tldr,
                "action items" | "actions" | "action-items" => &mut action_items,
                "decisions" | "decision" => &mut decisions,
                "transcript" => &mut transcript,
                _ => &mut transcript, // unknown sections go to transcript
            };
            continue; // skip the header line itself
        }
        current_section.push_str(line);
        current_section.push('\n');
    }

    // If no sections were parsed, treat entire content as transcript
    if tldr.is_empty() && transcript.is_empty() && action_items.is_empty() && decisions.is_empty() {
        transcript = content.to_string();
    }

    (tldr, transcript, action_items, decisions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ParrotConfig;

    fn temp_home() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    fn save_config(home: &std::path::Path, cfg: &ParrotConfig) {
        let config_dir = home.join(".parrot");
        std::fs::create_dir_all(&config_dir).unwrap();
        let path = config_dir.join("config.json");
        std::fs::write(&path, serde_json::to_string_pretty(cfg).unwrap()).unwrap();
    }

    #[test]
    fn parse_sections_with_headers() {
        let content = r#"## TLDR
This is the summary.

## Transcript
Speaker 1: Hello
Speaker 2: Hi

## Action Items
- Do thing 1
- Do thing 2

## Decisions
- Decided to go ahead
"#;
        let (tldr, transcript, action_items, decisions) = parse_sections(content);
        assert!(tldr.contains("This is the summary"));
        assert!(transcript.contains("Speaker 1: Hello"));
        assert!(action_items.contains("Do thing 1"));
        assert!(decisions.contains("Decided to go ahead"));
    }

    #[test]
    fn parse_sections_no_headers() {
        let content = "Just some plain text\nwith no headers";
        let (tldr, transcript, action_items, decisions) = parse_sections(content);
        assert!(tldr.is_empty());
        assert!(transcript.contains("Just some plain text"));
        assert!(action_items.is_empty());
        assert!(decisions.is_empty());
    }

    #[test]
    fn parse_sections_partial_headers() {
        let content = "## TLDR\nQuick summary\n\n## Transcript\nFull content here";
        let (tldr, transcript, action_items, decisions) = parse_sections(content);
        assert!(tldr.contains("Quick summary"));
        assert!(transcript.contains("Full content here"));
        assert!(action_items.is_empty());
        assert!(decisions.is_empty());
    }

    #[test]
    fn build_obsidian_note_basic() {
        let note = build_obsidian_note("2026-05-30", "team-standup", "recording.mp4", "Meeting content here");
        assert!(note.starts_with("---\n"));
        assert!(note.contains("date: 2026-05-30"));
        assert!(note.contains("type: meeting"));
        assert!(note.contains("tags:"));
        assert!(note.contains("  - meeting"));
        assert!(note.contains("source: recording.mp4"));
        assert!(note.contains("generated-by: parrot-cli"));
        assert!(note.contains("# Meeting: 2026-05-30 - team-standup"));
        assert!(note.contains("## TLDR"));
        assert!(note.contains("## Transcript"));
        assert!(note.contains("Meeting content here"));
    }

    #[test]
    fn build_obsidian_note_with_sections() {
        let content = "## TLDR\nSummary here\n\n## Transcript\nTranscript here\n\n## Action Items\n- Task 1\n\n## Decisions\n- Decided X";
        let note = build_obsidian_note("2026-05-30", "meeting", "notes.md", content);
        assert!(note.contains("Summary here"));
        assert!(note.contains("Transcript here"));
        assert!(note.contains("## Action Items"));
        assert!(note.contains("Task 1"));
        assert!(note.contains("## Decisions"));
        assert!(note.contains("Decided X"));
    }

    #[test]
    fn obsidian_status_configured() {
        let config = json!({"vault_path": "/tmp/testvault", "advanced": false});
        let status = status(Some(&config));
        // /tmp/testvault probably doesn't have .obsidian, so it'll show path not found or no .obsidian
        // Just check it doesn't say "not configured"
        assert!(!status.contains("not configured"));
    }

    #[test]
    fn obsidian_status_not_configured() {
        let status = status(None);
        assert!(status.contains("not configured"));
    }

    #[test]
    fn export_creates_obsidian_note() {
        let dir = temp_home();

        // Create a fake vault with .obsidian
        let vault_path = dir.path().join("TestVault");
        std::fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

        // Create input file
        let input_file = dir.path().join("meeting-notes.md");
        std::fs::write(&input_file, "## TLDR\nImportant meeting\n\n## Transcript\nWe discussed things").unwrap();

        // Set up config
        let mut cfg = ParrotConfig::default();
        cfg.mcp.servers.insert("obsidian".to_string(), json!({
            "vault_path": vault_path.to_str().unwrap(),
            "advanced": false
        }));

        // Run export
        export(input_file.to_str().unwrap(), &cfg).unwrap();

        // Check file was created
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let output_path = vault_path.join("Meetings").join(format!("{}-meeting-notes.md", today));
        assert!(output_path.exists(), "Output file should exist at {:?}", output_path);

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.starts_with("---\n"), "Should have YAML frontmatter");
        assert!(content.contains("date:"), "Should have date");
        assert!(content.contains("type: meeting"), "Should have type");
        assert!(content.contains("source: meeting-notes.md"), "Should have source");
        assert!(content.contains("## TLDR"), "Should have TLDR section");
        assert!(content.contains("Important meeting"), "Should have TLDR content");
        assert!(content.contains("## Transcript"), "Should have Transcript section");
    }

    #[test]
    fn export_missing_vault_fails() {
        let dir = temp_home();

        let input_file = dir.path().join("notes.md");
        std::fs::write(&input_file, "content").unwrap();

        let mut cfg = ParrotConfig::default();
        cfg.mcp.servers.insert("obsidian".to_string(), json!({
            "vault_path": "/nonexistent/path/vault",
            "advanced": false
        }));

        let result = export(input_file.to_str().unwrap(), &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn export_missing_input_file_fails() {
        let dir = temp_home();
        let vault_path = dir.path().join("TestVault");
        std::fs::create_dir_all(vault_path.join(".obsidian")).unwrap();

        let mut cfg = ParrotConfig::default();
        cfg.mcp.servers.insert("obsidian".to_string(), json!({
            "vault_path": vault_path.to_str().unwrap(),
            "advanced": false
        }));

        let result = export("/nonexistent/file.md", &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn export_without_obsidian_config_fails() {
        let cfg = ParrotConfig::default();
        let result = export("somefile.md", &cfg);
        assert!(result.is_err());
    }
}