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

#[cfg(test)]
mod tests {
    use clap::Parser;
    use super::*;

    #[test]
    fn parse_transcribe_basic() {
        let cli = Cli::try_parse_from(["parrot-cli", "transcribe", "meeting.mp4"]).unwrap();
        match cli.command {
            Commands::Transcribe { file, model, timestamps, no_timestamps, language, output } => {
                assert_eq!(file, "meeting.mp4");
                assert!(model.is_none());
                assert!(!timestamps);
                assert!(!no_timestamps);
                assert!(language.is_none());
                assert!(output.is_none());
            }
            _ => panic!("Expected Transcribe command"),
        }
    }

    #[test]
    fn parse_transcribe_with_options() {
        let cli = Cli::try_parse_from([
            "parrot-cli", "transcribe", "meeting.mp4",
            "--model", "whisper-large-v3",
            "--timestamps",
            "--language", "en",
            "--output", "/tmp/out.txt"
        ]).unwrap();
        match cli.command {
            Commands::Transcribe { file, model, timestamps, no_timestamps, language, output } => {
                assert_eq!(file, "meeting.mp4");
                assert_eq!(model.unwrap(), "whisper-large-v3");
                assert!(timestamps);
                assert!(!no_timestamps);
                assert_eq!(language.unwrap(), "en");
                assert_eq!(output.unwrap(), "/tmp/out.txt");
            }
            _ => panic!("Expected Transcribe command"),
        }
    }

    #[test]
    fn parse_transcribe_no_timestamps() {
        let cli = Cli::try_parse_from(["parrot-cli", "transcribe", "audio.wav", "--no-timestamps"]).unwrap();
        match cli.command {
            Commands::Transcribe { no_timestamps, .. } => {
                assert!(no_timestamps);
            }
            _ => panic!("Expected Transcribe command"),
        }
    }

    #[test]
    fn parse_summarize() {
        let cli = Cli::try_parse_from(["parrot-cli", "summarize", "transcript.txt"]).unwrap();
        match cli.command {
            Commands::Summarize { file, detail, sections } => {
                assert_eq!(file, "transcript.txt");
                assert!(!detail);
                assert!(!sections);
            }
            _ => panic!("Expected Summarize command"),
        }
    }

    #[test]
    fn parse_summarize_with_flags() {
        let cli = Cli::try_parse_from(["parrot-cli", "summarize", "transcript.txt", "--detail"]).unwrap();
        match cli.command {
            Commands::Summarize { detail, .. } => assert!(detail),
            _ => panic!("Expected Summarize command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "summarize", "transcript.txt", "--sections"]).unwrap();
        match cli.command {
            Commands::Summarize { sections, .. } => assert!(sections),
            _ => panic!("Expected Summarize command"),
        }
    }

    #[test]
    fn parse_actions() {
        let cli = Cli::try_parse_from(["parrot-cli", "actions", "transcript.txt", "--assignee"]).unwrap();
        match cli.command {
            Commands::Actions { file, assignee } => {
                assert_eq!(file, "transcript.txt");
                assert!(assignee);
            }
            _ => panic!("Expected Actions command"),
        }
    }

    #[test]
    fn parse_decisions() {
        let cli = Cli::try_parse_from(["parrot-cli", "decisions", "transcript.txt"]).unwrap();
        match cli.command {
            Commands::Decisions { file } => assert_eq!(file, "transcript.txt"),
            _ => panic!("Expected Decisions command"),
        }
    }

    #[test]
    fn parse_followup_with_channel() {
        let cli = Cli::try_parse_from(["parrot-cli", "followup", "transcript.txt", "--channel", "slack"]).unwrap();
        match cli.command {
            Commands::Followup { file, channel } => {
                assert_eq!(file, "transcript.txt");
                assert_eq!(channel.unwrap(), "slack");
            }
            _ => panic!("Expected Followup command"),
        }
    }

    #[test]
    fn parse_doc_default_format() {
        let cli = Cli::try_parse_from(["parrot-cli", "doc", "transcript.txt"]).unwrap();
        match cli.command {
            Commands::Doc { file, format } => {
                assert_eq!(file, "transcript.txt");
                assert_eq!(format, "notebooklm"); // default
            }
            _ => panic!("Expected Doc command"),
        }
    }

    #[test]
    fn parse_doc_obsidian_format() {
        let cli = Cli::try_parse_from(["parrot-cli", "doc", "transcript.txt", "--format", "obsidian"]).unwrap();
        match cli.command {
            Commands::Doc { format, .. } => assert_eq!(format, "obsidian"),
            _ => panic!("Expected Doc command"),
        }
    }

    #[test]
    fn parse_digest() {
        let cli = Cli::try_parse_from(["parrot-cli", "digest", "meeting.mp4", "--format", "markdown"]).unwrap();
        match cli.command {
            Commands::Digest { file, format } => {
                assert_eq!(file, "meeting.mp4");
                assert_eq!(format, "markdown");
            }
            _ => panic!("Expected Digest command"),
        }
    }

    #[test]
    fn parse_model_subcommands() {
        let cli = Cli::try_parse_from(["parrot-cli", "model", "list"]).unwrap();
        match cli.command {
            Commands::Model { command: ModelCommands::List } => {}
            _ => panic!("Expected Model List command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "model", "pull", "whisper-medium"]).unwrap();
        match cli.command {
            Commands::Model { command: ModelCommands::Pull { name } } => assert_eq!(name, "whisper-medium"),
            _ => panic!("Expected Model Pull command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "model", "remove", "whisper-tiny"]).unwrap();
        match cli.command {
            Commands::Model { command: ModelCommands::Remove { name } } => assert_eq!(name, "whisper-tiny"),
            _ => panic!("Expected Model Remove command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "model", "use", "whisper-large-v3"]).unwrap();
        match cli.command {
            Commands::Model { command: ModelCommands::Use { name } } => assert_eq!(name, "whisper-large-v3"),
            _ => panic!("Expected Model Use command"),
        }
    }

    #[test]
    fn parse_mcp_subcommands() {
        let cli = Cli::try_parse_from(["parrot-cli", "mcp", "list"]).unwrap();
        match cli.command {
            Commands::Mcp { command: McpCommands::List } => {}
            _ => panic!("Expected Mcp List command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "mcp", "add", "obsidian"]).unwrap();
        match cli.command {
            Commands::Mcp { command: McpCommands::Add { name } } => assert_eq!(name, "obsidian"),
            _ => panic!("Expected Mcp Add command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "mcp", "remove", "slack"]).unwrap();
        match cli.command {
            Commands::Mcp { command: McpCommands::Remove { name } } => assert_eq!(name, "slack"),
            _ => panic!("Expected Mcp Remove command"),
        }

        let cli = Cli::try_parse_from(["parrot-cli", "mcp", "test", "github"]).unwrap();
        match cli.command {
            Commands::Mcp { command: McpCommands::Test { name } } => assert_eq!(name, "github"),
            _ => panic!("Expected Mcp Test command"),
        }
    }

    #[test]
    fn parse_export() {
        let cli = Cli::try_parse_from(["parrot-cli", "export", "doc.md", "--to", "obsidian"]).unwrap();
        match cli.command {
            Commands::Export { file, to } => {
                assert_eq!(file, "doc.md");
                assert_eq!(to, "obsidian");
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn parse_invalid_command_fails() {
        assert!(Cli::try_parse_from(["parrot-cli", "nonexistent"]).is_err());
    }
}