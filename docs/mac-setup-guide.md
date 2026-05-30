# 🦜 Parrot CLI — Mac Setup & Usage Guide

Everything you need to install, configure, and use Parrot CLI on macOS.

---

## Prerequisites

### 1. Rust (to build from source)

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload your shell
source "$HOME/.cargo/env"

# Verify
rustc --version
cargo --version
```

### 2. ffmpeg (audio extraction from video)

```bash
# Install via Homebrew
brew install ffmpeg

# Verify
ffmpeg -version
```

### 3. Ollama (local LLM for summarization)

```bash
# Install via Homebrew
brew install ollama

# Or download from https://ollama.ai

# Start Ollama
ollama serve

# In a new terminal, pull the default model
ollama pull qwen3:14b

# Verify
ollama list
```

### 4. Tesseract OCR (for screenshot text extraction)

```bash
# Install via Homebrew
brew install tesseract

# Verify
tesseract --version
```

> **Note:** Tesseract is used by the `screenshot` command to extract text from video frames via OCR. It's optional — the screenshot command will warn you if it's missing.

### 5. Whisper (transcription)

You have two options:

**Option A: whisper.cpp (recommended, fastest on Mac with Apple Silicon)**

```bash
# Install via Homebrew
brew install whisper-cpp

# Or build from source for maximum performance
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
make

# Verify (if built from source)
./main --help
```

**Option B: OpenAI Whisper (Python, easier setup)**

```bash
# Install Python whisper
pip install openai-whisper

# Verify
whisper --help
```

> **Note:** Parrot CLI uses `whisper` (the Python CLI) by default. If you use whisper.cpp, you'll need to configure the binary path in `~/.parrot/config.json`.

### 6. NotebookLM CLI (optional, for NotebookLM export)

```bash
# Install via pip
pip install notebooklm-mcp-cli

# Authenticate with Google (opens browser)
nlm login

# Verify
nlm login --check
```

> **Note:** Required only if you want to export meeting notes to Google NotebookLM via `parrot-cli export --to notebooklm`.

---

## Install Parrot CLI

```bash
# Clone the repo
git clone https://github.com/yuanfengli168/parrot-cli.git
cd parrot-cli

# Build and install
cargo install --path .

# Verify
parrot-cli --help
```

---

## Quick Start

### Step 1: Transcribe a meeting

```bash
# Basic transcription (auto-detects language)
parrot-cli transcribe meeting.mp4

# With timestamps
parrot-cli transcribe meeting.mp4 --timestamps

# Specify a different Whisper model
parrot-cli transcribe meeting.mp4 --model whisper-large-v3

# Save to a specific file
parrot-cli transcribe meeting.mp4 --output transcript.txt
```

This will:
1. Extract audio from your mp4 using ffmpeg
2. Run Whisper transcription
3. Save the transcript to `~/.parrot/output/` (or your specified path)

### Step 2: Summarize

```bash
# TLDR + key points
parrot-cli summarize transcript.txt

# More detailed summary
parrot-cli summarize transcript.txt --detail

# Topic-segmented summary
parrot-cli summarize transcript.txt --sections
```

### Step 3: Extract action items

```bash
parrot-cli actions transcript.txt

# Try to attribute items to speakers
parrot-cli actions transcript.txt --assignee
```

### Step 4: Extract decisions

```bash
parrot-cli decisions transcript.txt
```

### Step 5: Draft a follow-up

```bash
# Default format
parrot-cli followup transcript.txt

# Slack format
parrot-cli followup transcript.txt --channel slack

# Email format
parrot-cli followup transcript.txt --channel email
```

### Step 6: Extract a screenshot

```bash
# Extract frame at a specific timestamp
parrot-cli screenshot meeting.mp4 "32:15"

# Search transcript for topic, then screenshot at that time
parrot-cli screenshot meeting.mp4 --search "budget discussion"

# Find specific sentence in transcript, screenshot at that time
parrot-cli screenshot meeting.mp4 --sentence "the revenue target is 5M"
```

This will:
1. Extract the video frame at the given timestamp using ffmpeg
2. Run OCR on the frame using tesseract
3. Save the screenshot (.png) and OCR text (.txt) to `~/.parrot/output/screenshots/`

### Step 8: Generate a document

```bash
# NotebookLM-ready format
parrot-cli doc transcript.txt --format notebooklm

# Obsidian markdown
parrot-cli doc transcript.txt --format obsidian

# Standard markdown
parrot-cli doc transcript.txt --format markdown
```

### All-in-one: Digest

```bash
# Transcribe + summarize + actions + decisions + doc in one command
parrot-cli digest meeting.mp4
```

---

## Model Management

### List models

```bash
parrot-cli model list
```

Output:
```
Available Whisper Models:

  whisper-large-v3   ~3.0GB   ~8GB RAM   (best quality, slower)
  whisper-medium     ~1.5GB   ~4GB RAM   (good balance)
  whisper-small      ~500MB   ~2GB RAM   (fast, decent)
  whisper-tiny       ~75MB    ~1GB RAM   (fastest, rough)

Installed:
  ✅ whisper-medium

Default: whisper-medium
```

### Download a model

```bash
# Use an alias
parrot-cli model pull whisper-large-v3

# Or use a full HuggingFace URL
parrot-cli model pull https://huggingface.co/openai/whisper-large-v3
```

### Set default model

```bash
parrot-cli model use whisper-large-v3
```

### Remove a model

```bash
parrot-cli model remove whisper-small
```

---

## MCP Integration

MCP (Model Context Protocol) lets you push results to your existing tools.

### Configure MCP servers

```bash
# List available and configured servers
parrot-cli mcp list

# Add Obsidian (connects to your vault)
parrot-cli mcp add obsidian

# Add Notion
parrot-cli mcp add notion

# Add NotebookLM
parrot-cli mcp add notebooklm

# Add Slack
parrot-cli mcp add slack

# Remove a server
parrot-cli mcp remove notion

# Test a connection
parrot-cli mcp test obsidian
```

### Export via MCP

```bash
# Push notes to Obsidian vault
parrot-cli export transcript.txt --to obsidian

# Create a Notion page
parrot-cli export transcript.txt --to notion

# Import into NotebookLM
parrot-cli export transcript.txt --to notebooklm

# Post summary to Slack
parrot-cli export transcript.txt --to slack
```

---

## Configuration

Config file: `~/.parrot/config.json`

Parrot creates a default config on first run. You can edit it manually:

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
    "servers": {
      "obsidian": {
        "vault_path": "/Users/jacky/MyVault"
      }
    }
  }
}
```

### Key config options

| Setting | Default | Description |
|---|---|---|
| `transcription.model` | `whisper-medium` | Default Whisper model |
| `transcription.language` | `null` (auto-detect) | Language hint (e.g., `en`, `zh`) |
| `transcription.timestamps` | `true` | Include timestamps in transcript |
| `llm.provider` | `ollama` | LLM provider (ollama, openai) |
| `llm.model` | `qwen3:14b` | Model for summaries |
| `llm.ollama_url` | `http://localhost:11434` | Ollama server URL |
| `output.dir` | `~/.parrot/output` | Where outputs are saved |
| `output.screenshots_dir` | `~/.parrot/output/screenshots` | Where screenshots are saved |
| `mcp.servers` | `{}` | Configured MCP servers |

---

## Directory Structure

```
~/.parrot/
├── config.json          # Your configuration
├── models/              # Downloaded Whisper models
│   ├── whisper-medium/
│   └── whisper-large-v3/
└── output/              # Transcripts, summaries, docs
    ├── screenshots/     # Extracted video frames + OCR text
    │   ├── 2026-05-30_meeting_32-15.png
    │   └── 2026-05-30_meeting_32-15.txt
    ├── 2026-05-30_meeting-transcript.txt
    ├── 2026-05-30_meeting-summary.txt
    └── 2026-05-30_meeting-doc.md
```

---

## Common Workflows

### Typical meeting processing

```bash
# 1. After a meeting, run the digest
parrot-cli digest my-meeting.mp4

# 2. Review the output
cat ~/.parrot/output/my-meeting-digest.md

# 3. Export to your notes
parrot-cli export my-meeting-digest.md --to obsidian
```

### Export to NotebookLM

```bash
# 1. Make sure nlm is installed and authenticated
pip install notebooklm-mcp-cli
nlm login

# 2. Add NotebookLM as an MCP server
parrot-cli mcp add notebooklm

# 3. Export your meeting notes
parrot-cli export my-meeting-digest.md --to notebooklm
```

### Using a larger model for important meetings

```bash
# Pull the best model (one-time)
parrot-cli model pull whisper-large-v3

# Use it for this meeting
parrot-cli transcribe board-meeting.mp4 --model whisper-large-v3
```

### Multilingual meetings

```bash
# Auto-detect (default)
parrot-cli transcribe bilingual-meeting.mp4

# Or hint the language
parrot-cli transcribe bilingual-meeting.mp4 --language zh
```

### Quick follow-up after a meeting

```bash
# One-liner: digest + send follow-up
parrot-cli digest standup.mp4 && parrot-cli followup ~/.parrot/output/standup-transcript.txt --channel slack
```

---

## Troubleshooting

### "ffmpeg not found"

```bash
brew install ffmpeg
```

### "tesseract not found"

```bash
brew install tesseract
```

### "Ollama not running"

```bash
ollama serve
```

### "Model not found"

```bash
# Pull the model first
parrot-cli model pull whisper-medium

# Or for Ollama
ollama pull qwen3:14b
```

### Slow transcription on Mac

- Use `whisper-small` or `whisper-tiny` for faster results
- For Apple Silicon Macs, whisper.cpp with Metal is significantly faster than Python whisper
- If using whisper.cpp, make sure it's compiled with Metal support

### Ollama connection refused

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# If not, start it
ollama serve
```

### "nlm not found" (NotebookLM export)

```bash
# Install nlm CLI
pip install notebooklm-mcp-cli

# Authenticate
nlm login
```

### NotebookLM authentication expired

```bash
# Re-authenticate
nlm login
```

---

## Tips & Tricks

1. **Start with `digest`** — it runs the full pipeline in one shot. Only use individual commands if you need fine control.

2. **Use `--model whisper-tiny` for drafts** — fast rough transcription to check if you even want to process a meeting.

3. **Switch LLM models** — if qwen3:14b is too slow, try `qwen3:8b` or `llama3:8b` in your config.

4. **Auto-process with a script** — point Parrot at your Zoom/Teams recording folder:
   ```bash
   # Example: process all new recordings
   for f in ~/Recordings/*.mp4; do
     parrot-cli digest "$f"
   done
   ```

5. **Obsidian integration** — set your vault path in config, then `parrot-cli export --to obsidian` drops markdown notes directly into your vault.

---

*Happy parroting! 🦜*