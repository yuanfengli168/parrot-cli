use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

use crate::config;

// Built-in model aliases -> HuggingFace URLs
fn model_aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("whisper-large-v3", "https://huggingface.co/openai/whisper-large-v3/resolve/main/model.bin");
    m.insert("whisper-medium", "https://huggingface.co/openai/whisper-medium/resolve/main/model.bin");
    m.insert("whisper-small", "https://huggingface.co/openai/whisper-small/resolve/main/model.bin");
    m.insert("whisper-tiny", "https://huggingface.co/openai/whisper-tiny/resolve/main/model.bin");
    m
}

fn model_sizes() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("whisper-large-v3", "~3GB, needs ~8GB RAM");
    m.insert("whisper-medium", "~1.5GB, needs ~4GB RAM");
    m.insert("whisper-small", "~500MB, needs ~2GB RAM");
    m.insert("whisper-tiny", "~75MB, needs ~1GB RAM");
    m
}

pub async fn list() -> Result<()> {
    let aliases = model_aliases();
    let sizes = model_sizes();
    let models_dir = config::models_dir();

    println!("🦜 Whisper Models\n");
    println!("{:<20} {:<12} {}", "Alias", "Installed", "Info");
    println!("{}", "-".repeat(55));

    for (alias, _url) in &aliases {
        let installed = if models_dir.join(alias).exists() { "✅" } else { "⬜" };
        let info = sizes.get(alias).unwrap_or(&"");
        println!("{:<20} {:<12} {}", alias, installed, info);
    }

    println!("\nUse `parrot-cli model pull <alias>` to download.");

    // Also show config default
    let cfg = config::load();
    println!("\nCurrent default: {}", cfg.transcription.model);
    Ok(())
}

pub async fn pull(name: &str) -> Result<()> {
    let aliases = model_aliases();
    let url = if name.starts_with("https://") {
        name.to_string()
    } else {
        aliases.get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown model alias: {}. Available: {}", name, aliases.keys().cloned().collect::<Vec<_>>().join(", ")))?
            .to_string()
    };

    let models_dir = config::models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let model_path = models_dir.join(name);
    if model_path.exists() {
        println!("Model {} already downloaded.", name);
        return Ok(());
    }

    println!("⬇️  Downloading model {}...", name);
    let client = Client::new();
    let mut resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to download model: HTTP {}", resp.status());
    }

    let total = resp.content_length();
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&model_path)?;
    use std::io::Write;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if let Some(total) = total {
            let pct = (downloaded as f64 / total as f64) * 100.0;
            print!("\r  Downloaded: {:.1}% ({:.1}MB / {:.1}MB)", pct, downloaded as f64 / 1_048_576.0, total as f64 / 1_048_576.0);
        } else {
            print!("\r  Downloaded: {:.1}MB", downloaded as f64 / 1_048_576.0);
        }
        std::io::stdout().flush()?;
    }

    println!("\n✅ Model {} saved to {}", name, model_path.display());
    Ok(())
}

pub async fn remove(name: &str) -> Result<()> {
    let model_path = config::models_dir().join(name);
    if model_path.exists() {
        std::fs::remove_file(&model_path)?;
        println!("🗑️  Model {} removed.", name);
    } else {
        println!("Model {} not found locally.", name);
    }
    Ok(())
}

pub async fn set_default(name: &str) -> Result<()> {
    let aliases = model_aliases();
    if !aliases.contains_key(name) && !name.starts_with("https://") {
        anyhow::bail!("Unknown model alias: {}. Available: {}", name, aliases.keys().cloned().collect::<Vec<_>>().join(", "));
    }

    let mut cfg = config::load();
    cfg.transcription.model = name.to_string();
    config::save(&cfg)?;
    println!("✅ Default model set to: {}", name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_aliases_contain_all_known_models() {
        let aliases = model_aliases();
        assert!(aliases.contains_key("whisper-large-v3"));
        assert!(aliases.contains_key("whisper-medium"));
        assert!(aliases.contains_key("whisper-small"));
        assert!(aliases.contains_key("whisper-tiny"));
        assert_eq!(aliases.len(), 4);
    }

    #[test]
    fn model_aliases_point_to_hf_urls() {
        let aliases = model_aliases();
        for (name, url) in &aliases {
            assert!(url.starts_with("https://huggingface.co/"), "{} URL should be from HuggingFace", name);
            assert!(url.ends_with("/model.bin"), "{} URL should end with model.bin", name);
            assert!(url.contains(name), "{} URL should contain model name", name);
        }
    }

    #[test]
    fn model_sizes_contain_all_models() {
        let sizes = model_sizes();
        assert_eq!(sizes.len(), 4);
        assert!(sizes.contains_key("whisper-large-v3"));
        assert!(sizes.get("whisper-large-v3").unwrap().contains("3GB"));
        assert!(sizes.get("whisper-tiny").unwrap().contains("75MB"));
    }

    #[test]
    fn model_aliases_urls_are_valid() {
        let aliases = model_aliases();
        // Verify specific URLs
        assert_eq!(*aliases.get("whisper-large-v3").unwrap(),
            "https://huggingface.co/openai/whisper-large-v3/resolve/main/model.bin");
        assert_eq!(*aliases.get("whisper-medium").unwrap(),
            "https://huggingface.co/openai/whisper-medium/resolve/main/model.bin");
        assert_eq!(*aliases.get("whisper-small").unwrap(),
            "https://huggingface.co/openai/whisper-small/resolve/main/model.bin");
        assert_eq!(*aliases.get("whisper-tiny").unwrap(),
            "https://huggingface.co/openai/whisper-tiny/resolve/main/model.bin");
    }
}