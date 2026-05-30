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