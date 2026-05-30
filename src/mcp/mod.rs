use anyhow::Result;
use serde_json::json;

use crate::config;

// Curated MCP servers for v1
fn available_servers() -> Vec<&'static str> {
    vec!["obsidian", "notion", "notebooklm", "slack", "jira", "diffchecker", "google-docs", "email", "github", "todoist", "confluence", "telegram"]
}

pub async fn list() -> Result<()> {
    let cfg = config::load();

    println!("🦜 MCP Servers\n");

    // Configured servers
    println!("Installed:");
    if cfg.mcp.servers.is_empty() {
        println!("  (none configured)");
    } else {
        for (name, _config) in &cfg.mcp.servers {
            println!("  ✅ {:<15} configured", name);
        }
    }

    // Available but not configured
    println!("\nAvailable:");
    for server in available_servers() {
        if !cfg.mcp.servers.contains_key(server) {
            println!("  ⬜ {}", server);
        }
    }

    println!("\nUse `parrot-cli mcp add <name>` to configure a server.");
    Ok(())
}

pub async fn add(name: &str) -> Result<()> {
    let mut cfg = config::load();

    if cfg.mcp.servers.contains_key(name) {
        println!("MCP server '{}' is already configured.", name);
        return Ok(());
    }

    let server_config = match name {
        "obsidian" => json!({ "vault_path": "" }),
        "notion" => json!({ "api_key": "" }),
        "notebooklm" => json!({ "enabled": true }),
        "slack" => json!({ "bot_token": "", "channel": "" }),
        "jira" => json!({ "host": "", "api_token": "" }),
        "google-docs" => json!({ "credentials": "" }),
        "email" => json!({ "smtp_host": "", "from": "" }),
        "github" => json!({ "token": "" }),
        "todoist" => json!({ "api_token": "" }),
        "confluence" => json!({ "host": "", "api_token": "" }),
        "telegram" => json!({ "bot_token": "", "chat_id": "" }),
        "diffchecker" => json!({ "api_key": "" }),
        _ => json!({ "enabled": true }),
    };

    cfg.mcp.servers.insert(name.to_string(), server_config);
    config::save(&cfg)?;

    println!("✅ Added MCP server: {}", name);
    println!("   Edit ~/.parrot/config.json to configure credentials.");
    Ok(())
}

pub async fn remove(name: &str) -> Result<()> {
    let mut cfg = config::load();

    if cfg.mcp.servers.remove(name).is_some() {
        config::save(&cfg)?;
        println!("🗑️  Removed MCP server: {}", name);
    } else {
        println!("MCP server '{}' not found in configuration.", name);
    }
    Ok(())
}

pub async fn test(name: &str) -> Result<()> {
    let cfg = config::load();

    if !cfg.mcp.servers.contains_key(name) {
        anyhow::bail!("MCP server '{}' not configured. Run: parrot-cli mcp add {}", name, name);
    }

    // For now, just verify config exists
    // Full MCP protocol testing would require spawning the server process
    println!("🔍 Testing MCP server '{}'...", name);
    println!("✅ Server '{}' is configured.", name);
    println!("   Note: Full MCP protocol testing requires running the server. Coming in future release.");
    Ok(())
}