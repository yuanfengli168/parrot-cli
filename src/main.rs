use clap::{Parser, Subcommand};

mod commands;
mod config;
mod llm;
mod mcp;
mod model;
mod transcribe;

#[derive(Parser)]
#[command(name = "parrot-cli", version, about = "🦜 Parrot — Local-first meeting assistant CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transcribe an audio/video file
    Transcribe {
        /// Input file (mp4, mp3, wav, etc.)
        file: String,
        /// Model to use (alias or path)
        #[arg(long)]
        model: Option<String>,
        /// Include timestamps
        #[arg(long)]
        timestamps: bool,
        /// No timestamps (clean text)
        #[arg(long)]
        no_timestamps: bool,
        /// Language hint (default: auto-detect)
        #[arg(long)]
        language: Option<String>,
        /// Output file path
        #[arg(long)]
        output: Option<String>,
    },
    /// Summarize a transcript
    Summarize {
        /// Input file (transcript or media file)
        file: String,
        /// Longer, more thorough summary
        #[arg(long)]
        detail: bool,
        /// Topic-segmented summary
        #[arg(long)]
        sections: bool,
    },
    /// Extract action items from a transcript
    Actions {
        /// Input file (transcript or media file)
        file: String,
        /// Attempt to attribute actions to speakers
        #[arg(long)]
        assignee: bool,
    },
    /// Extract decisions from a transcript
    Decisions {
        /// Input file (transcript or media file)
        file: String,
    },
    /// Draft a follow-up message
    Followup {
        /// Input file (transcript or media file)
        file: String,
        /// Format for channel (slack, email)
        #[arg(long)]
        channel: Option<String>,
    },
    /// Generate a structured document (for NotebookLM etc.)
    Doc {
        /// Input file (transcript or media file)
        file: String,
        /// Output format (notebooklm, markdown, obsidian)
        #[arg(long, default_value = "notebooklm")]
        format: String,
    },
    /// Transcribe + summarize + actions + decisions + doc all-in-one
    Digest {
        /// Input file (media file)
        file: String,
        /// Output format for doc
        #[arg(long, default_value = "notebooklm")]
        format: String,
    },
    /// Manage transcription models
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
    /// Manage MCP servers
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Export a document via MCP
    Export {
        /// Input file
        file: String,
        /// Export target (obsidian, notion, notebooklm, slack)
        #[arg(long)]
        to: String,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List installed and available models
    List,
    /// Download a model (by alias or URL)
    Pull {
        /// Model alias (e.g. whisper-medium) or HuggingFace URL
        name: String,
    },
    /// Remove an installed model
    Remove {
        /// Model alias to remove
        name: String,
    },
    /// Set default model
    Use {
        /// Model alias to set as default
        name: String,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// List configured and available MCP servers
    List,
    /// Add/configure an MCP server
    Add {
        /// Server name
        name: String,
    },
    /// Remove an MCP server
    Remove {
        /// Server name
        name: String,
    },
    /// Test connection to an MCP server
    Test {
        /// Server name
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Transcribe { file, model, timestamps, no_timestamps, language, output } => {
            commands::transcribe::run(&file, model.as_deref(), timestamps, no_timestamps, language.as_deref(), output.as_deref()).await
        }
        Commands::Summarize { file, detail, sections } => {
            commands::summarize::run(&file, detail, sections).await
        }
        Commands::Actions { file, assignee } => {
            commands::actions::run(&file, assignee).await
        }
        Commands::Decisions { file } => {
            commands::decisions::run(&file).await
        }
        Commands::Followup { file, channel } => {
            commands::followup::run(&file, channel.as_deref()).await
        }
        Commands::Doc { file, format } => {
            commands::doc::run(&file, &format).await
        }
        Commands::Digest { file, format } => {
            commands::digest::run(&file, &format).await
        }
        Commands::Model { command } => match command {
            ModelCommands::List => model::list().await,
            ModelCommands::Pull { name } => model::pull(&name).await,
            ModelCommands::Remove { name } => model::remove(&name).await,
            ModelCommands::Use { name } => model::set_default(&name).await,
        },
        Commands::Mcp { command } => match command {
            McpCommands::List => mcp::list().await,
            McpCommands::Add { name } => mcp::add(&name).await,
            McpCommands::Remove { name } => mcp::remove(&name).await,
            McpCommands::Test { name } => mcp::test(&name).await,
        },
        Commands::Export { file, to } => {
            commands::export::run(&file, &to).await
        }
    }
}