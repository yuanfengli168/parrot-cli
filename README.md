# 🦜 Parrot CLI

A local-first, MCP-extensible CLI that turns meeting recordings into actionable knowledge.

## Features

- **Transcribe** — Extract audio via ffmpeg, transcribe via Whisper
- **Summarize** — TLDR + key points via Ollama (qwen3:14b default)
- **Action Items** — Extract who owes what by when
- **Decisions** — Extract decisions made, stripped of discussion
- **Follow-up** — Draft follow-up messages (Slack, email)
- **Document Generation** — Structured docs for NotebookLM, Obsidian, Markdown
- **Digest** — All-in-one: transcribe → summarize → actions → decisions → doc
- **Model Management** — Pull, list, swap Whisper models
- **MCP Integration** — Export to Obsidian, Notion, Slack, and more

## Install

```bash
# Build from source (requires Rust)
git clone https://github.com/jacky/parrot-cli.git
cd parrot-cli
cargo install --path .
```

## Prerequisites

- [ffmpeg](https://ffmpeg.org/) — audio extraction from video files
- [Whisper](https://github.com/openai/whisper) — transcription (`pip install openai-whisper`)
- [Ollama](https://ollama.ai/) — local LLM for summarization (default: qwen3:14b)

## Usage

```bash
# Transcribe a meeting recording
parrot-cli transcribe meeting.mp4

# Summarize a transcript
parrot-cli summarize transcript.txt

# Extract action items
parrot-cli actions transcript.txt

# Extract decisions
parrot-cli decisions transcript.txt

# Draft follow-up message
parrot-cli followup transcript.txt --channel slack

# Generate structured document
parrot-cli doc transcript.txt --format notebooklm

# All-in-one digest
parrot-cli digest meeting.mp4
```

## Model Management

```bash
# List available and installed models
parrot-cli model list

# Download a model
parrot-cli model pull whisper-medium

# Set default model
parrot-cli model use whisper-medium
```

## MCP Integration

```bash
# List MCP servers
parrot-cli mcp list

# Add a server
parrot-cli mcp add obsidian

# Test connection
parrot-cli mcp test obsidian

# Export to a target
parrot-cli export digest.md --to obsidian
```

## Configuration

Config file: `~/.parrot/config.json`

```json
{
  "transcription": {
    "model": "whisper-medium",
    "language": null,
    "timestamps": true
  },
  "llm": {
    "provider": "ollama",
    "model": "qwen3:14b",
    "ollama_url": "http://localhost:11434"
  },
  "output": {
    "dir": "~/.parrot/output"
  },
  "mcp": {
    "servers": {}
  }
}
```

## Philosophy

- **Local-first** — Your data stays on your machine
- **Privacy-respecting** — No cloud APIs required (Ollama runs locally)
- **Model-agnostic** — Choose your own Whisper model size
- **MCP-extensible** — Plugin system for connecting to your tools

## License

MIT