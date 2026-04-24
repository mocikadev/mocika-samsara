> 中文版本：[README.md](../README.md)

# samsara

[![CI](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/mocikadev/mocika-samsara)](https://github.com/mocikadev/mocika-samsara/releases/latest)

A knowledge management CLI for AI Agents. Lets AI accumulate experience like a human — log errors as lessons, promote repeated ones to rules, and surface rules in AGENTS.md automatically.

Works alongside [`skm`](https://github.com/mocikadev/mocika-skills-cli) as the Samsara knowledge system toolchain:

- **skm** manages skill packages (`~/.agents/skills/`, Layer 1)
- **samsara** manages knowledge lessons (`~/.agents/knowledge/`, Layer 2)

## Features

- **Self-evolution**: Error → log lesson → repeated occurrences → promote to rule → write to AGENTS.md, effective on next AI startup
- **MCP integration**: Configure once, AI calls samsara automatically, no manual commands needed
- **Pure file storage**: No database, no daemon — knowledge/ is just a git repository
- **Multi-device sync**: `samsara push` / `samsara pull` keeps your knowledge base in sync
- **Zero root required**: All data written to `~/.agents/`, no sudo needed

## Install samsara

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

Installs to `~/.local/bin/samsara`. No Rust toolchain required; git must be in PATH. To use a custom path:

```bash
SAMSARA_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)
```

To install a specific version:

```bash
SAMSARA_VERSION=v0.1.0 bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)
```

## Quick Start

```bash
# 1. Initialize the knowledge base
samsara init

# 2. Hit an error — log it
samsara write rust cargo-fmt --summary "Pre-commit order: cargo fmt → clippy → test" --type error

# 3. Hit the same issue again — increment occurrences
samsara write rust cargo-fmt

# 4. After 3 occurrences, promote to a rule
samsara promote rust cargo-fmt

# 5. Promote to AGENTS.md (AI reads this on every startup)
samsara promote rust cargo-fmt --layer0
```

## Install self-evolution skill (recommended)

`self-evolution` is the companion AI Agent skill package that lets your AI assistant automatically call samsara at the right moment — no prompting needed:

```bash
skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all
```

Or run `samsara init` to install automatically (requires skm).

> Once installed, your AI Agent will log lessons, search past experience, and recommend promotions on its own.

## AI Tool Integration

Configure once — AI calls samsara directly via MCP, no manual commands needed.

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

> ⚠️ OpenCode and Claude Code use different config formats — do not mix them. The samsara process is started on demand by the tool; no manual startup needed.

## Command Reference

| Command | Description |
|---------|-------------|
| `samsara init [--yes]` | Initialize the knowledge base |
| `samsara write <domain> <keyword> [--summary "..."] [--type error\|skill\|pattern\|insight] [--verify]` | Write / update a lesson |
| `samsara search <query> [--domain d] [--type t]` | Search the knowledge base |
| `samsara promote <domain> <keyword> [--layer0]` | Promote to rule / write to AGENTS.md |
| `samsara lint [--fix]` | Check knowledge base health |
| `samsara reflect` | Analyze learning patterns |
| `samsara prime [--limit N] [--domain d]` | Top N promotion candidates |
| `samsara archive <domain> <keyword>` | Archive a lesson |
| `samsara demote <pattern> [--yes]` | Remove a rule from AGENTS.md |
| `samsara status` | Knowledge base statistics |
| `samsara log [--tail N] [--action t] [--rotate]` | Operation log |
| `samsara skill-note <name> [--fail] [--note "..."]` | Record skill usage result |
| `samsara domain list\|add` | Manage domains |
| `samsara remote add\|set\|show` | Manage sync remote |
| `samsara push [--dry-run]` | Push to remote |
| `samsara pull` | Pull from remote |
| `samsara self-update [--check]` | Upgrade to latest version |
| `samsara mcp serve` | Start MCP server (called automatically by AI tools) |

## Data Directory

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

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | — | Planned |

## Build from Source

```bash
git clone https://github.com/mocikadev/mocika-samsara
cd mocika-samsara
cargo build --release
# Output: ./target/release/samsara
```

Requires Rust 1.88+.

## License

Licensed under either of [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE) at your option.
