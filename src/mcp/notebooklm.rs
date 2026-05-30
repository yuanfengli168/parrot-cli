use anyhow::{bail, Result};
use std::process::Command;

/// Check if `nlm` CLI is installed
pub fn is_nlm_installed() -> bool {
    Command::new("which")
        .arg("nlm")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if `nlm` is authenticated via `nlm login --check`
pub fn is_nlm_authenticated() -> Result<bool> {
    let output = Command::new("nlm")
        .args(["login", "--check"])
        .output()?;

    Ok(output.status.success())
}

/// Create a notebook via `nlm notebook create "<title>"` and return the notebook ID
pub fn create_notebook(title: &str) -> Result<String> {
    let output = Command::new("nlm")
        .args(["notebook", "create", title])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create notebook: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_notebook_id(&stdout)
}

/// Add a text source to a notebook via `nlm source add <notebook-id> --text "<content>"`
pub fn add_text_source(notebook_id: &str, text: &str) -> Result<()> {
    let output = Command::new("nlm")
        .args(["source", "add", notebook_id, "--text", text])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add text source: {}", stderr.trim());
    }
    Ok(())
}

/// Add a URL source to a notebook via `nlm source add <notebook-id> --url "<url>"`
pub fn add_url_source(notebook_id: &str, url: &str) -> Result<()> {
    let output = Command::new("nlm")
        .args(["source", "add", notebook_id, "--url", url])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add URL source: {}", stderr.trim());
    }
    Ok(())
}

/// Add a file source to a notebook via `nlm source add <notebook-id> --file "<path>"`
pub fn add_file_source(notebook_id: &str, path: &str) -> Result<()> {
    let output = Command::new("nlm")
        .args(["source", "add", notebook_id, "--file", path])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add file source: {}", stderr.trim());
    }
    Ok(())
}

/// Parse notebook ID from `nlm notebook create` output
fn parse_notebook_id(output: &str) -> Result<String> {
    // Output format may be like:
    // "Created notebook: My Notebook (id: abc123)" or just the ID on a line
    for line in output.lines() {
        let line = line.trim();
        // Try to match patterns like "id: <id>" or "(id: <id>)"
        if let Some(idx) = line.rfind("id:") {
            let id_part = line[idx + 3..].trim();
            let id = id_part.trim_end_matches(')').trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }
        // Try to match "Created notebook <name> <id>" patterns
        if let Some(idx) = line.rfind("notebook ") {
            let rest = line[idx + 9..].trim();
            if !rest.is_empty() && rest.len() < 100 {
                return Ok(rest.to_string());
            }
        }
        // If the line looks like a plain ID (alphanumeric, short)
        if !line.is_empty() && line.len() < 50 && line.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Ok(line.to_string());
        }
    }
    bail!("Could not parse notebook ID from nlm output:\n{}", output);
}

/// Check prerequisites and return a helpful error if nlm is missing
pub fn check_prerequisites() -> Result<()> {
    if !is_nlm_installed() {
        bail!(
            "❌ 'nlm' CLI is not installed.\n\n\
             Install it with:\n  pip install notebooklm-mcp-cli\n\n\
             Then authenticate:\n  nlm login\n\n\
             See: https://github.com/jacob-bd/notebooklm-mcp-cli"
        );
    }
    if !is_nlm_authenticated()? {
        bail!(
            "❌ 'nlm' is installed but not authenticated.\n\n\
             Run:\n  nlm login\n\n\
             This will open a browser for Google authentication."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_notebook_id_with_id_prefix() {
        let output = "Created notebook: Meeting Test (id: abc123def456)";
        let id = parse_notebook_id(output).unwrap();
        assert_eq!(id, "abc123def456");
    }

    #[test]
    fn parse_notebook_id_plain_line() {
        let output = "abc123def456";
        let id = parse_notebook_id(output).unwrap();
        assert_eq!(id, "abc123def456");
    }

    #[test]
    fn parse_notebook_id_with_notebook_keyword() {
        let output = "Created notebook my-meeting-notebook abcd1234";
        let id = parse_notebook_id(output).unwrap();
        assert!(id.contains("abcd1234"));
    }

    #[test]
    fn parse_notebook_id_empty_fails() {
        assert!(parse_notebook_id("").is_err());
    }

    #[test]
    fn parse_notebook_id_garbage_fails() {
        assert!(parse_notebook_id("no id here that looks like anything useful at all really").is_err());
    }
}