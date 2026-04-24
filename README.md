> English version: [docs/README.en.md](docs/README.en.md)

# samsara

[![CI](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/mocikadev/mocika-samsara)](https://github.com/mocikadev/mocika-samsara/releases/latest)

AI Agent 知识管理 CLI。让 AI 像人一样积累经验——遇到错误记下来，多次遇到晋升为规则，规则沉淀进 AGENTS.md。

与 [`skm`](https://github.com/mocikadev/mocika-skills-cli) 共同构成 Samsara 知识系统的工具层：

- **skm** 管理技能包（`~/.agents/skills/`，Layer 1）
- **samsara** 管理知识教训（`~/.agents/knowledge/`，Layer 2）

## 特性

- **自我进化**：遇到错误 → 记录教训 → 多次触发 → 晋升规则 → 写入 AGENTS.md，AI 下次启动即生效
- **MCP 集成**：配置一次，AI 自动调用，无需手动执行命令
- **纯文件存储**：无数据库、无 daemon，knowledge/ 就是一个 git 仓库
- **多设备同步**：`samsara push` / `samsara pull`，知识库跟着走
- **零 root 权限**：全部数据写入 `~/.agents/`，无需 sudo

## 安装 samsara

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

安装到 `~/.local/bin/samsara`，无需 Rust 环境，git 需在 PATH 中。如需自定义路径：

```bash
SAMSARA_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)
```

安装指定版本：

```bash
SAMSARA_VERSION=v0.1.0 bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)
```

## 快速上手

```bash
# 1. 初始化知识库
samsara init

# 2. 遇到错误，记下来
samsara write rust cargo-fmt --summary "提交前顺序：cargo fmt → clippy → test" --type error

# 3. 再次踩坑，occurrences +1
samsara write rust cargo-fmt

# 4. 出现 3 次后晋升为规则
samsara promote rust cargo-fmt

# 5. 晋升到 AGENTS.md（AI 每次启动都会读到）
samsara promote rust cargo-fmt --layer0
```

## 安装 self-evolution skill（推荐）

`self-evolution` 是配套的 AI Agent 技能包，让你的 AI 助手在合适的时机自动调用 samsara，无需手动提示：

```bash
skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all
```

或通过 `samsara init` 自动安装（需已安装 skm）。

> 安装后，AI Agent 会自动记录教训、检索经验、推荐晋升，无需用户提醒。

## AI 工具集成

配置一次，AI 直接通过 MCP 调用 samsara，无需手动执行命令。

**OpenCode** — 编辑 `~/.config/opencode/opencode.json`：

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

**Claude Code** — 编辑 `~/.claude/claude_desktop_config.json`：

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

> ⚠️ OpenCode 与 Claude Code 的配置格式不同，不可混用。samsara 进程由工具按需自动启动，无需手动运行。

## 命令速查

| 命令 | 说明 |
|------|------|
| `samsara init [--yes]` | 初始化知识库 |
| `samsara write <domain> <keyword> [--summary "..."] [--type error\|skill\|pattern\|insight] [--verify]` | 写入 / 更新教训 |
| `samsara search <query> [--domain d] [--type t]` | 搜索知识库 |
| `samsara promote <domain> <keyword> [--layer0]` | 晋升为规则 / 写入 AGENTS.md |
| `samsara lint [--fix]` | 检查知识库健康度 |
| `samsara reflect` | 分析学习模式 |
| `samsara prime [--limit N] [--domain d]` | 推荐晋升候选 |
| `samsara archive <domain> <keyword>` | 归档教训 |
| `samsara demote <pattern> [--yes]` | 从 AGENTS.md 降级规则 |
| `samsara status` | 知识库统计 |
| `samsara log [--tail N] [--action t] [--rotate]` | 操作日志 |
| `samsara skill-note <name> [--fail] [--note "..."]` | 记录 skill 使用结果 |
| `samsara domain list\|add` | 管理 domain |
| `samsara remote add\|set\|show` | 管理同步远端 |
| `samsara push [--dry-run]` | 推送到远端 |
| `samsara pull` | 从远端拉取 |
| `samsara self-update [--check]` | 升级到最新版本 |
| `samsara mcp serve` | 启动 MCP 服务（由 AI 工具自动调用）|

## 数据目录

```
~/.agents/
├── knowledge/
│   ├── lessons/         # 教训文件（按 domain 分目录）
│   ├── rules/           # 已晋升的规则（rules/<domain>.md）
│   ├── archive/         # 归档的教训
│   ├── INDEX.md         # 全量索引（自动维护）
│   └── log.md           # 操作日志
├── AGENTS.md            # 自进化协议 + 晋升的 layer0 规则
└── samsara.toml         # 配置（同步远端等）
```

## 平台支持

| 平台 | 架构 | 状态 |
|------|------|------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | — | 计划中 |

## 从源码构建

```bash
git clone https://github.com/mocikadev/mocika-samsara
cd mocika-samsara
cargo build --release
# 产物：./target/release/samsara
```

需要 Rust 1.88+。

## 许可证

本项目采用 **MIT OR Apache-2.0** 双协议授权，你可以选择其中任意一种。

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
