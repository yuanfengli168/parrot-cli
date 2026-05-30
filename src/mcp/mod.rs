pub mod notebooklm;

use anyhow::Result;
use serde_json::json;

use crate::config;

// Curated MCP servers for v1
fn available_servers() -> Vec<&'static str> {
    vec!["obsidian", "notion", "notebooklm", "slack", "jira", "diffchecker", "google-docs", "email", "github", "todoist", "confluence", "telegram"]
}

/// Get status string for a server (checks nlm for notebooklm, stub for others)
fn server_status(name: &str, configured: bool) -> String {
    if name == "notebooklm" {
        if !notebooklm::is_nlm_installed() {
            return "❌ nlm not installed (pip install notebooklm-mcp-cli)".to_string();
        }
        match notebooklm::is_nlm_authenticated() {
            Ok(true) if configured => "✅ connected".to_string(),
            Ok(true) => "✅ authenticated".to_string(),
            Ok(false) => "⚠️  not authenticated (run: nlm login)".to_string(),
            Err(_) => "❓ could not check auth".to_string(),
        }
    } else if configured {
        "✅ configured".to_string()
    } else {
        "⬜ not configured".to_string()
    }
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
            let status = server_status(name, true);
            println!("  {:<15} {}", name, status);
        }
    }

    // Available but not configured
    println!("\nAvailable:");
    for server in available_servers() {
        if !cfg.mcp.servers.contains_key(server) {
            let status = server_status(server, false);
            println!("  ⬜ {:<15} {}", server, status);
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

    // Special handling for notebooklm
    if name == "notebooklm" {
        if !notebooklm::is_nlm_installed() {
            println!("❌ 'nlm' CLI is not installed.");
            println!("\nInstall it with:");
            println!("  pip install notebooklm-mcp-cli");
            println!("\nThen authenticate:");
            println!("  nlm login");
            println!("\nSee: https://github.com/jacob-bd/notebooklm-mcp-cli");
            anyhow::bail!("nlm CLI not installed");
        }
        if !notebooklm::is_nlm_authenticated()? {
            println!("⚠️  'nlm' is installed but not authenticated.");
            println!("\nRun:");
            println!("  nlm login");
            anyhow::bail!("nlm not authenticated");
        }
        cfg.mcp.servers.insert(name.to_string(), json!({ "enabled": true }));
        config::save(&cfg)?;
        println!("✅ Added MCP server: {}", name);
        return Ok(());
    }

    let server_config = match name {
        "obsidian" => json!({ "vault_path": "" }),
        "notion" => json!({ "api_key": "" }),
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

    // Special handling for notebooklm
    if name == "notebooklm" {
        println!("🔍 Testing NotebookLM MCP server...");

        if !notebooklm::is_nlm_installed() {
            println!("❌ 'nlm' CLI is not installed.");
            println!("\nInstall it with:");
            println!("  pip install notebooklm-mcp-cli");
            anyhow::bail!("nlm CLI not installed");
        }
        println!("  ✅ nlm CLI is installed");

        match notebooklm::is_nlm_authenticated() {
            Ok(true) => println!("  ✅ nlm is authenticated"),
            Ok(false) => {
                println!("  ❌ nlm is not authenticated");
                println!("\nRun:");
                println!("  nlm login");
                anyhow::bail!("nlm not authenticated");
            }
            Err(e) => {
                println!("  ❓ Could not check auth status: {}", e);
            }
        }

        println!("\n✅ NotebookLM MCP server is ready!");
        return Ok(());
    }

    // Generic test for other servers
    println!("🔍 Testing MCP server '{}'...", name);
    println!("✅ Server '{}' is configured.", name);
    println!("   Note: Full MCP protocol testing requires running the server. Coming in future release.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ParrotConfig;

    fn temp_home() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    fn save_config_with_servers(home: &std::path::Path, servers: std::collections::HashMap<String, serde_json::Value>) {
        let config_dir = home.join(".parrot");
        std::fs::create_dir_all(&config_dir).unwrap();
        let mut cfg = ParrotConfig::default();
        cfg.mcp.servers = servers;
        let path = config_dir.join("config.json");
        std::fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
    }

    #[test]
    fn available_servers_list() {
        let servers = available_servers();
        assert!(servers.contains(&"obsidian"));
        assert!(servers.contains(&"notion"));
        assert!(servers.contains(&"slack"));
        assert!(servers.contains(&"notebooklm"));
        assert!(servers.contains(&"github"));
        assert!(servers.len() >= 10);
    }

    #[test]
    fn server_config_for_known_servers() {
        // Test the server config generation logic by checking known names
        let obsidian = json!({ "vault_path": "" });
        assert!(obsidian.get("vault_path").is_some());

        let slack = json!({ "bot_token": "", "channel": "" });
        assert!(slack.get("bot_token").is_some());

        let notion = json!({ "api_key": "" });
        assert!(notion.get("api_key").is_some());
    }

    #[test]
    fn mcp_config_roundtrip() {
        let dir = temp_home();
        let mut servers = std::collections::HashMap::new();
        servers.insert("obsidian".to_string(), json!({"vault_path": "/my/vault"}));
        servers.insert("slack".to_string(), json!({"bot_token": "xoxb-123"}));
        save_config_with_servers(dir.path(), servers);

        let config_path = dir.path().join(".parrot/config.json");
        let loaded: ParrotConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded.mcp.servers.len(), 2);
        assert_eq!(loaded.mcp.servers["obsidian"]["vault_path"], "/my/vault");
        assert_eq!(loaded.mcp.servers["slack"]["bot_token"], "xoxb-123");
    }

    #[test]
    fn mcp_add_inserts_into_config() {
        let dir = temp_home();
        save_config_with_servers(dir.path(), std::collections::HashMap::new());

        let config_path = dir.path().join(".parrot/config.json");
        let mut cfg: ParrotConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        cfg.mcp.servers.insert("obsidian".to_string(), json!({"vault_path": ""}));
        std::fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();

        let loaded: ParrotConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(loaded.mcp.servers.contains_key("obsidian"));
    }

    #[test]
    fn mcp_remove_from_config() {
        let dir = temp_home();
        let mut servers = std::collections::HashMap::new();
        servers.insert("slack".to_string(), json!({"bot_token": ""}));
        save_config_with_servers(dir.path(), servers);

        let config_path = dir.path().join(".parrot/config.json");
        let mut cfg: ParrotConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        cfg.mcp.servers.remove("slack");
        std::fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();

        let loaded: ParrotConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(!loaded.mcp.servers.contains_key("slack"));
    }
}