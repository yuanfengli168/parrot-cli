# Parrot 🦜 — Meeting Assistant CLI

> A local-first, MCP-extensible CLI that turns meeting recordings into actionable knowledge.

---

## Overview

Parrot is a macOS CLI tool for post-meeting processing. It transcribes audio/video, summarizes discussions, extracts action items and decisions, and exports results to your tools — all powered by local models with MCP plugin extensibility.

**Philosophy:** Local-first, privacy-respecting, extensible via MCP. You own your data.

**Input:** mp4 video files only (no audio-only, no built-in recording for now). ffmpeg extracts audio internally.

**Language:** Auto-detect (Whisper handles 99 languages).

**License:** Open source (MIT or Apache 2.0 — TBD).

**Pricing:** Core CLI free + open source. Cloud transcription/API as optional paid feature.

---

## Name & Branding

- **Parrot** 🦜 — parrots repeat what they hear, just like meeting transcripts
- CLI binary: `parrot-cli`
- GitHub/npm: `parrot-cli`
- Config dir: `~/.parrot/`
- Models dir: `~/.parrot/models/`
- Output dir: `~/.parrot/output/` (default, configurable)

---

## Core Pipeline

```
Video/Audio → Transcribe → (Summarize | Actions | Decisions | Follow-up) → Export
```

Each step is a composable command. Intermediate artifacts are cached so you can re-run one step without redoing everything.

---

## Commands

### Transcription

```bash
parrot-cli transcribe <file>                         # transcribe with default model
parrot-cli transcribe <file> --model whisper-large-v3  # specific model
parrot-cli transcribe <file> --timestamps             # include timestamps
parrot-cli transcribe <file> --no-timestamps           # clean text only
parrot-cli transcribe <file> --language en             # hint language (default: auto-detect)
parrot-cli transcribe <file> --output transcript.srt   # custom output path
```

### Summarization

```bash
parrot-cli summarize <file-or-transcript>              # TLDR + key points
parrot-cli summarize <file> --detail                   # longer, more thorough
parrot-cli summarize <file> --sections                  # topic-segmented summary
```

### Action Items

```bash
parrot-cli actions <file-or-transcript>                 # extract action items
parrot-cli actions <file> --assignee                    # attempt to attribute to speakers
```

### Decisions

```bash
parrot-cli decisions <file-or-transcript>               # extract decisions made
```

### Follow-up

```bash
parrot-cli followup <file-or-transcript>                 # draft follow-up message
parrot-cli followup <file> --channel slack              # format for Slack
parrot-cli followup <file> --channel email               # format as email
```

### Clip Extraction

```bash
parrot-cli clip <file> "32:10-35:00"                    # extract segment
parrot-cli clip <file> --search "budget discussion"      # find and extract by topic
```

### Screenshots

Extract frames from the meeting video at specific timestamps, with OCR for searchable text.

```bash
parrot-cli screenshot <file> "32:15"                          # extract frame at timestamp
parrot-cli screenshot <file> --search "budget discussion"      # find timestamp via transcript search, then screenshot
parrot-cli screenshot <file> --sentence "the revenue target is 5M"  # find sentence in transcript, screenshot at that time
```

**How it works:**
1. `ffmpeg` extracts the frame at the given timestamp
2. `tesseract` (or similar) runs OCR on the extracted frame
3. OCR text is saved alongside the image for searchability
4. For `--search` and `--sentence`: first searches the transcript to find the timestamp, then extracts the frame

**Storage — separate files with links:**
```
~/.parrot/output/
├── screenshots/
│   ├── 2026-05-30_meeting_32-15.png
│   ├── 2026-05-30_meeting_32-15.txt     # OCR text
│   ├── 2026-05-30_meeting_45-02.png
│   └── 2026-05-30_meeting_45-02.txt     # OCR text
├── 2026-05-30_meeting-transcript.txt
└── 2026-05-30_meeting-summary.txt
```

**Integration with other exports:**
- **Obsidian:** `![[2026-05-30_meeting_32-15.png]]` embedded inline next to the transcript line at 32:15
- **NotebookLM:** Import screenshot as image source alongside transcript text
- **Future LLM search:** "find the slide about revenue" → searches OCR text → returns the right screenshot

### Diff

```bash
parrot-cli diff <file1> <file2>                          # compare two meeting transcripts
# Uses Diffchecker MCP if available, otherwise local diff
```

### Document Generation (NotebookLM-ready)

```bash
parrot-cli doc <file>                                    # generate structured doc
parrot-cli doc <file> --format notebooklm                # formatted for NotebookLM import
parrot-cli doc <file> --format markdown                   # general markdown
parrot-cli doc <file> --format obsidian                   # Obsidian-flavored markdown
```

**Document contents:**
1. TLDR of transcript
2. Full transcript with speaker labels + timestamps
3. Action items section
4. Decisions section

### Digest (All-in-One)

```bash
parrot-cli digest <file>                                 # transcribe + summarize + actions + decisions + doc
```

---

## Model Management

### Pull Models

```bash
parrot-cli model pull whisper-large-v3          # alias → downloads from HuggingFace
parrot-cli model pull whisper-medium             # alias
parrot-cli model pull whisper-small              # alias
parrot-cli model pull whisper-tiny               # alias
parrot-cli model pull https://huggingface.co/... # full URL for other models
```

### Built-in Aliases

| Alias | HF Source | Size | RAM |
|---|---|---|---|
| `whisper-large-v3` | openai/whisper-large-v3 | ~3GB | ~8GB |
| `whisper-medium` | openai/whisper-medium | ~1.5GB | ~4GB |
| `whisper-small` | openai/whisper-small | ~500MB | ~2GB |
| `whisper-tiny` | openai/whisper-tiny | ~75MB | ~1GB |

**Future:** `siliconmind-v1`, `faster-whisper-large-v3`, etc.

### Model Commands

```bash
parrot-cli model list                    # show installed + available aliases with sizes
parrot-cli model pull <alias-or-url>     # download a model
parrot-cli model remove <alias>          # delete a model
parrot-cli model use <alias>             # set as default for transcription
```

If input starts with `https://`, treat as direct HuggingFace URL. Otherwise resolve alias from embedded map.

---

## MCP Integration

MCP (Model Context Protocol) is Parrot's plugin system. It connects transcription output to your existing tools without building integrations from scratch.

### MCP Subcommands

```bash
parrot-cli mcp list                      # show configured + available servers
parrot-cli mcp add <name>                # add/configure an MCP server
parrot-cli mcp remove <name>             # remove an MCP server
parrot-cli mcp test <name>               # ping and verify connection
```

### `parrot-cli mcp list` Output

```
Installed:
  ✅ obsidian        connected
  ✅ notion          connected
  ⬜ notebooklm      not configured
  ⬜ slack           not configured

Available:
  jira, diffchecker, google-docs, email, github, todoist, confluence, telegram
```

### Curated MCP Servers (v1)

| MCP Server | Use Case | Integration |
|---|---|---|
| **NotebookLM** | Import transcripts + summaries | jacob-bd/notebooklm-mcp-cli (pip install) — uses `nlm` CLI + MCP server |
| **Obsidian** | Export meeting notes to Obsidian vault | StevenStavrakis/obsidian-mcp (optional, for advanced features) — basic export is direct file write |
| **Notion** | Create meeting notes pages | TBD |
| **Slack** | Post follow-ups and summaries to channels | TBD |
| **Jira / Linear** | Create tickets from action items | TBD |
| **Diffchecker** | Compare transcripts between meetings | TBD |
| **Google Docs** | Export docs to shared Drive | TBD |
| **Email** | Draft and send follow-up emails | TBD |
| **GitHub** | Link discussions to issues/PRs | TBD |
| **Todoist / TickTick** | Push action items to task app | TBD |
| **Confluence** | Enterprise wiki integration | TBD |
| **Telegram / WhatsApp** | Share clips or summaries to groups | TBD |

#### NotebookLM Integration Details

**Chosen server:** [jacob-bd/notebooklm-mcp-cli](https://github.com/jacob-bd/notebooklm-mcp-cli) (⭐ 4,666)

**Why this one:**
- CLI-first (`nlm` command) — perfect match for Parrot's CLI architecture
- Also includes MCP server (`notebooklm-mcp`) for protocol integration
- Largest community, most actively maintained
- Pipelines & batch operations
- `nlm login` for browser-based Google auth with cookie persistence
- 35 MCP tools covering notebooks, sources, audio, video, research

**Prerequisites:**
```bash
pip install notebooklm-mcp-cli
nlm login  # browser-based Google auth
```

**Integration pattern (shell out to `nlm`):**
```bash
# parrot-cli export meeting-doc.md --to notebooklm
# 1. Check if nlm is installed
# 2. Create notebook:
nlm notebook create "Meeting: 2026-05-30"
# 3. Add source:
nlm source add <notebook-id> --text "$(cat meeting-doc.md)"
# 4. Report success with notebook URL
```

**Alternative pattern (MCP stdio):** Spawn `notebooklm-mcp` as subprocess, communicate via JSON-RPC. More complex but allows richer integration.

**Fallback:** If `nlm` not installed, `parrot-cli mcp test notebooklm` tells user to install.

#### Obsidian Integration Details

**Chosen server:** [StevenStavrakis/obsidian-mcp](https://github.com/StevenStavrakis/obsidian-mcp) (⭐ 712) — optional for advanced features.

**Two integration levels:**

**Level 1 — Basic (no MCP needed):**
Direct file write to vault directory. This covers 90% of the use case.
- `parrot-cli mcp add obsidian` → asks for vault path, saves to config
- `parrot-cli export <file> --to obsidian` → writes markdown file to vault with YAML frontmatter
- Creates: `<vault>/Meetings/YYYY-MM-DD-<filename>.md`
- Frontmatter includes: date, tags (meeting, action-item), type, source file
- No Obsidian plugin or MCP server required

**Level 2 — Advanced (with MCP):**
Install StevenStavrakis/obsidian-mcp for search, tag management, editing existing notes.
- `parrot-cli mcp add obsidian --advanced` → also configures npx obsidian-mcp
- Enables: search-vault, manage-tags, edit-note, move-note
- Requires: `npx -y obsidian-mcp <vault-path>`

**Why direct file write first:**
- Obsidian vaults are just markdown files on disk
- No dependency on running services or plugins
- Works even when Obsidian is closed
- Parrot can write proper frontmatter + tags + wiki-links natively

### Export via MCP

```bash
parrot-cli export <file> --to obsidian        # push to Obsidian vault
parrot-cli export <file> --to notion          # create Notion page
parrot-cli export <file> --to notebooklm      # import into NotebookLM
parrot-cli export <file> --to slack           # post summary to Slack
parrot-cli export <file> --to jira            # create tickets from actions
parrot-cli export <file> --to google-docs     # save to Google Docs
```

---

## LLM Configuration

- **Default local:** Ollama with `qwen3:14b`
- **Cloud option:** OpenAI API, Claude API (future)
- **Configurable** in `~/.parrot/config.json`

```json
{
  "transcription": {
    "model": "whisper-medium",
    "language": null,
    "timestamps": true
  },
  "llm": {
    "provider": "ollama",
    "model": "qwen3:14b"
  },
  "output": {
    "dir": "~/.parrot/output"
  },
  "mcp": {
    "servers": {
      "obsidian": { "vault_path": "/Users/jacky/ObsidianVault" }
    }
  }
}
```

---

## Transcription Backends

| Backend | Type | Quality | Speed | Notes |
|---|---|---|---|---|
| **whisper.cpp** | Local (Metal) | High | Fast | Default, Apple Silicon optimized |
| **mlx-whisper** | Local (MLX) | High | Fast | Alternative Mac-optimized |
| **faster-whisper** | Local (CTranslate2) | High | Medium | Less Mac-optimized |
| **OpenAI Whisper API** | Cloud | Highest | Fast | Costs $0.006/min |

---

## Features Summary

| # | Feature | Description |
|---|---|---|
| 1 | **Transcribe** | Video/audio → timestamped transcript |
| 2 | **Summarize** | TLDR + key points + topic sections |
| 3 | **Action Items** | Extract who owes what by when |
| 4 | **Decisions** | Extract decisions made, stripped of discussion |
| 5 | **Search** | Find moments by keyword/phrase with timestamps |
| 6 | **Document Generation** | Structured doc (TLDR + transcript) for NotebookLM |
| 7 | **Follow-up Draft** | Ready-to-send follow-up message (Slack/email) |
| 8 | **Diff Meetings** | Compare two transcripts via Diffchecker MCP |
| 9 | **Clip Extraction** | Extract audio/video segment by time or topic |
| 10 | **Meeting Quality Score** | Flag if meeting was productive or off-topic |
| 11 | **MCP Export** | Push results to Obsidian, Notion, Slack, etc. |
| 12 | **Model Management** | Pull, list, swap transcription models easily |
| 13 | **Screenshots** | Extract frames from video at timestamps, with OCR for searchable text |

---

## v1 Scope

### In
- Transcribe (whisper.cpp)
- Summarize (qwen3:14b via Ollama)
- Action items
- Decisions
- Follow-up draft
- Document generation (NotebookLM format)
- Model management with aliases
- MCP: Obsidian, Notion, NotebookLM, Slack
- `parrot-cli mcp list/add/remove/test`
- Config file

### Out (later)
- Speaker diarization
- Real-time transcription
- Meeting comparison/diff (v2)
- Meeting quality score (v2)
- Additional MCP servers beyond v1 curated set

---

## Tech Stack (Proposed)

- **Language:** Rust or Go (single binary, fast, no runtime)
- **Transcription:** whisper.cpp (C library, Metal-accelerated on Mac)
- **LLM:** Ollama API (localhost) for summaries
- **MCP:** MCP client protocol over stdio/SSE
- **Audio extraction:** ffmpeg (pre-existing binary)
- **Screenshot extraction:** ffmpeg (frame extraction)
- **OCR:** tesseract (text extraction from screenshots)
- **Config:** JSON (~/.parrot/config.json)

---

## Competitive Differentiation

| Feature | Parrot | Otter.ai | Fireflies | tl;dv |
|---|---|---|---|---|
| Local-first transcription | ✅ | ❌ | ❌ | ❌ |
| Choose your own model | ✅ | ❌ | ❌ | ❌ |
| MCP plugin ecosystem | ✅ | ❌ | Limited | Limited |
| Obsidian export | ✅ | ❌ | ❌ | ❌ |
| NotebookLM import | ✅ | ❌ | ❌ | ❌ |
| Self-hosted / private | ✅ | ❌ | ❌ | ❌ |
| Free (local models) | ✅ | Freemium | Freemium | Freemium |

**Key differentiator:** Parrot is local-first, model-agnostic, and extensible via MCP. No vendor lock-in, no data leaving your machine unless you choose it.

---

*Last updated: 2026-05-30*
*Discussion between Jacky and Secretory 🦜*