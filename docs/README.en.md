> 中文版本：[README.md](../README.md)

<div align="center">

<img src="../assets/logo.png" height="120" alt="samsara" /><br/>
<strong>samsara &nbsp;·&nbsp; 輪廻</strong>

<sub>AI Agent Knowledge Management CLI — Let experience accumulate through every cycle</sub>

<br/>

[![CI](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/mocikadev/mocika-samsara)](https://github.com/mocikadev/mocika-samsara/releases/latest)

</div>

---

Most AI tools just "follow instructions". **samsara** solves a different problem: how can AI learn from experience like a human — log errors as lessons, promote repeated ones to rules, write rules into AGENTS.md, and never repeat the same mistake again.

## Quick Start

### 1. Install skm

[`skm`](https://github.com/mocikadev/mocika-skills-cli) is samsara's skill package manager — it handles installing and managing AI Agent skills and must be installed first:

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

### 2. Install samsara

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

**Windows** (PowerShell)

```powershell
irm https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.ps1 | iex
```

Installs to `~/.local/bin/samsara` (Windows: `~\.local\bin\samsara.exe`). No Rust toolchain required; git must be in PATH. The install script automatically checks whether skm is present.

### 3. Initialize the knowledge base

```bash
samsara init
```

Creates `~/.agents/knowledge/`, injects the self-evolution protocol into `~/.agents/AGENTS.md`, and installs the `self-evolution` skill automatically.

### 4. Configure MCP (let AI take over)

**OpenCode** — edit `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "samsara": {
      "type": "local",
      "command": ["samsara", "mcp", "serve"]
    }
  }
}
```

**Claude Code** — edit `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "samsara": {
      "command": "samsara",
      "args": ["mcp", "serve"]
    }
  }
}
```

Restart your AI tool after saving. The samsara process is started on demand — no manual startup needed.

---

## Install self-evolution skill

`self-evolution` is the companion AI Agent skill package that tells your AI assistant **when** and **how** to call samsara — no prompting required:

```bash
skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all
```

> If skm was already installed when you ran `samsara init`, the skill was installed automatically.

Once installed, your AI will automatically:

- Hit an error or repeated pitfall → call `samsara_write_lesson` to log it
- Start a new task → call `samsara_search_knowledge` to surface relevant experience
- Detect high-frequency errors → proactively suggest `samsara_promote_lesson`

## First-run example

On the first conversation, AI searches the knowledge base (empty), then works normally. When it encounters a noteworthy error, it logs it automatically:

> **AI**: Found a recurring pattern worth logging.
>
> *Calling `samsara_write_lesson`*
> ```
> domain:  rust
> keyword: cargo-fmt-order
> summary: Pre-commit order must be cargo fmt → clippy → test; wrong order breaks CI
> type:    error
> ```
>
> ✅ Logged. Will surface automatically next time.

After the same issue appears 3 times, AI will proactively suggest promotion:

> **AI**: `rust/cargo-fmt-order` has occurred 3 times. Promote it to a rule in AGENTS.md so it's loaded on every startup?

---

## Migrate an existing knowledge base

If you already have experience accumulated in `AGENTS.md` or a `lessons-learned.md`, you can migrate it in.

**Option 1: Let AI batch-migrate for you**

Paste your existing notes into the chat and tell the AI:

> Please write each item below into the knowledge base using `samsara_write_lesson`. Infer the domain from context; choose type from error / skill / pattern / insight.

AI will call the MCP tool for each entry — no manual work needed.

**Option 2: Migrate manually**

```bash
samsara write rust   cargo-fmt  --summary "Pre-commit: fmt → clippy → test" --type error
samsara write git    commit-msg --summary "Format: type: description in Chinese"  --type skill
```

---

## Data directory

```
~/.agents/
├── knowledge/
│   ├── lessons/         # Lesson files (organized by domain)
│   ├── rules/           # Promoted rules (rules/<domain>.md)
│   ├── archive/         # Archived lessons
│   ├── INDEX.md         # Full index (auto-maintained)
│   └── log.md           # Operation log
├── AGENTS.md            # Self-evolution protocol + promoted layer0 rules
└── samsara.toml         # Config (sync remote, etc.)
```

## Three-Layer Knowledge System

Works alongside [`skm`](https://github.com/mocikadev/mocika-skills-cli) as the complete Samsara AI knowledge system:

```
  Loaded automatically on AI startup
         │
         ▼
┌─────────────────────────────────────────────┐
│  Layer 0 · AGENTS.md · Permanent layer      │
│  Promoted layer0 rules, loaded every session│
└──────────────────┬──────────────────────────┘
      promote --layer0 ↑
                   │
      ┌────────────┴────────────┐
      │                         │
┌─────┴──────────────┐  ┌───────┴──────────────────┐
│ Layer 1 · skm       │  │ Layer 2 · samsara          │
│ ~/.agents/skills/   │  │ ~/.agents/knowledge/       │
│ Skill packages      │  │ lessons/ → rules/          │
│ self-evolution etc. │  │ Log lessons → promote rules│
└────────────────────┘  └──────────────────────────┘
```

## Why Not Something Else?

| Capability | Manual AGENTS.md | Mem0 | Zep | LangChain Memory | **samsara** |
|------------|:---:|:---:|:---:|:---:|:---:|
| Structured lesson logging | ❌ | ❌ | ❌ | ❌ | ✅ |
| Progressive promotion (occurrences) | ❌ | ❌ | ❌ | ❌ | ✅ |
| Auto-write to AGENTS.md | ⚠️ manual | ❌ | ❌ | ❌ | ✅ |
| No LLM / embedding needed | ✅ | ❌ | ❌ | ❌ | ✅ |
| Native MCP integration | ❌ | ❌ | ❌ | ❌ | ✅ |
| Multi-device git sync | ⚠️ manual | ❌ | ❌ | ❌ | ✅ |
| Local-first, data ownership | ✅ | ❌ | ❌ | ❌ | ✅ |
| Zero-dependency install (single binary) | ✅ | ❌ | ❌ | ❌ | ✅ |
| Works across AI tools | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | x86_64 | ✅ |

## Build from Source

```bash
git clone https://github.com/mocikadev/mocika-samsara
cd mocika-samsara
cargo build --release
# Output: ./target/release/samsara
```

Requires Rust 1.88+.

## Command Reference

Full command list: [docs/commands.md](commands.md).

## License

Licensed under either of [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE) at your option.
