use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParrotConfig {
    #[serde(default)]
    pub transcription: TranscriptionConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default = "default_true")]
    pub timestamps: bool,
}

fn default_model() -> String { "whisper-medium".to_string() }
fn default_true() -> bool { true }

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self { model: default_model(), language: None, timestamps: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model_llm")]
    pub model: String,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
}

fn default_provider() -> String { "ollama".to_string() }
fn default_model_llm() -> String { "qwen3:14b".to_string() }
fn default_ollama_url() -> String { "http://localhost:11434".to_string() }

impl Default for LlmConfig {
    fn default() -> Self {
        Self { provider: default_provider(), model: default_model_llm(), ollama_url: default_ollama_url() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_dir")]
    pub dir: String,
}

fn default_output_dir() -> String {
    format!("{}/.parrot/output", std::env::var("HOME").unwrap_or_default())
}

impl Default for OutputConfig {
    fn default() -> Self { Self { dir: default_output_dir() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for McpConfig {
    fn default() -> Self { Self { servers: std::collections::HashMap::new() } }
}

impl Default for ParrotConfig {
    fn default() -> Self {
        Self {
            transcription: TranscriptionConfig::default(),
            llm: LlmConfig::default(),
            output: OutputConfig::default(),
            mcp: McpConfig::default(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(format!("{}/.parrot", home))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn models_dir() -> PathBuf {
    config_dir().join("models")
}

pub fn output_dir() -> PathBuf {
    PathBuf::from(load().output.dir.clone())
}

pub fn ensure_dirs() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::create_dir_all(models_dir())?;
    let dir = output_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(())
}

pub fn load() -> ParrotConfig {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ParrotConfig::default()
    }
}

pub fn save(cfg: &ParrotConfig) -> anyhow::Result<()> {
    ensure_dirs()?;
    let content = serde_json::to_string_pretty(cfg)?;
    std::fs::write(config_path(), content)?;
    Ok(())
}