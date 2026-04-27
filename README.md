> English version: [docs/README.en.md](docs/README.en.md)

<div align="center">

<img src="assets/logo.png" height="120" alt="samsara" /><br/>
<strong>samsara · 轮回</strong>

<sub>AI Agent 知识管理 CLI — 让经验随轮回积累，不再重蹈覆辙</sub>

<br/>

[![CI](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-samsara/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/mocikadev/mocika-samsara)](https://github.com/mocikadev/mocika-samsara/releases/latest)

</div>

---

大多数 AI 工具只会"按指令执行"。**samsara** 想解决的是：AI 如何像人一样从经验中学习——遇到错误记录下来，反复踩坑后晋升为规则，规则写进 AGENTS.md，下次启动自动生效，永不重蹈覆辙。

## 快速开始

### 1. 安装 skm

[`skm`](https://github.com/mocikadev/mocika-skills-cli) 是 samsara 的技能包管理器，负责安装和管理 AI Agent skill，需要先安装：

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

### 2. 安装 samsara

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

**Windows**（PowerShell）

```powershell
irm https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.ps1 | iex
```

Installs to `~/.local/bin/samsara`（Windows 为 `~\.local\bin\samsara.exe`），无需 Rust 环境，git 需在 PATH 中。安装脚本会自动检测 skm 是否已安装，并自动执行 `samsara init` 初始化知识库。

### 3. 配置 MCP（让 AI 接管）

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

配置完成后重启 AI 工具，samsara 进程由工具按需自动启动，无需手动运行。

---

## 安装 self-evolution skill

`self-evolution` 是配套的 AI Agent 技能包，让 AI 助手知道**何时**以及**如何**调用 samsara，无需你手动提示：

```bash
skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all
```

> 如果 `samsara init` 时已安装 skm，skill 会自动安装，无需手动执行。

安装后，AI 会自动：

- 遇到错误或踩坑 → 调用 `samsara_write_lesson` 记录教训
- 开始任务前 → 调用 `samsara_search_knowledge` 检索已有经验
- 发现高频错误 → 主动建议 `samsara_promote_lesson`

## 首次激活示例

第一次和 AI 对话时，AI 会先检索知识库，然后正常工作。遇到第一个可归纳的问题时，AI 会自动记录：

> **AI**：发现一个可归纳的错误，正在记录到知识库……
>
> *调用 `samsara_write_lesson`*
> ```
> domain:  rust
> keyword: cargo-fmt-order
> summary: 提交前顺序必须是 cargo fmt → clippy → test，顺序颠倒会导致 CI 失败
> type:    error
> ```
>
> ✅ 已记录，再次遇到时会自动关联。

当同一问题出现 3 次后，AI 会主动建议晋升：

> **AI**：`rust/cargo-fmt-order` 已出现 3 次，建议晋升为规则写入 AGENTS.md，以后每次启动都会提醒。要晋升吗？

---

## 迁移已有知识库

如果你在 `AGENTS.md` 或 `lessons-learned.md` 中已有积累，可以迁移进来。

**方式一：让 AI 帮你批量迁移**

把已有的经验文本发给 AI，告诉它：

> 请把以下内容逐条用 `samsara_write_lesson` 写入知识库，domain 根据内容归类，type 选 error / skill / pattern / insight 之一。

AI 会自动调用 MCP 逐条写入，无需手动操作。

**方式二：手动逐条迁移**

```bash
samsara write rust cargo-fmt --summary "提交前：fmt → clippy → test" --type error
samsara write git commit   --summary "commit 格式：type: 中文描述" --type skill
```

---

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

## 三层知识体系

与 [`skm`](https://github.com/mocikadev/mocika-skills-cli) 共同构成完整的 Samsara AI 知识系统：

```
  AI 工具启动时自动读取
         │
         ▼
┌─────────────────────────────────────────────┐
│  Layer 0 · AGENTS.md · 永久生效层            │
│  已晋升的 layer0 规则，每次会话强制加载      │
└──────────────────┬──────────────────────────┘
      promote --layer0 写入 ↑
                   │
      ┌────────────┴────────────┐
      │                         │
┌─────┴──────────────┐  ┌───────┴──────────────────┐
│ Layer 1 · skm       │  │ Layer 2 · samsara          │
│ ~/.agents/skills/   │  │ ~/.agents/knowledge/       │
│ 技能包（行为模板）  │  │ lessons/ → rules/          │
│ self-evolution 等   │  │ 教训记录 → 晋升为规则      │
└────────────────────┘  └──────────────────────────┘
```

## 为什么不用别的方案？

| 能力 | 手动维护 AGENTS.md | Mem0 | Zep | LangChain Memory | **samsara** |
|------|:---:|:---:|:---:|:---:|:---:|
| 结构化教训记录 | ❌ | ❌ | ❌ | ❌ | ✅ |
| 渐进式晋升（occurrences 计数）| ❌ | ❌ | ❌ | ❌ | ✅ |
| 自动写入 AGENTS.md | ⚠️ 手动 | ❌ | ❌ | ❌ | ✅ |
| 无需 LLM / embedding | ✅ | ❌ | ❌ | ❌ | ✅ |
| MCP 原生集成 | ❌ | ❌ | ❌ | ❌ | ✅ |
| 多设备 git 同步 | ⚠️ 手动 | ❌ | ❌ | ❌ | ✅ |
| 本地优先、数据自有 | ✅ | ❌ | ❌ | ❌ | ✅ |
| 零依赖安装（单二进制）| ✅ | ❌ | ❌ | ❌ | ✅ |
| 跨 AI 工具通用 | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |

## 平台支持

| 平台 | 架构 | 状态 |
|------|------|------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | x86_64 | ✅ |

## 从源码构建

```bash
git clone https://github.com/mocikadev/mocika-samsara
cd mocika-samsara
cargo build --release
# 产物：./target/release/samsara
```

需要 Rust 1.88+。

## 命令参考

完整命令列表见 [docs/commands.md](docs/commands.md)。

## 许可证

本项目采用 **MIT OR Apache-2.0** 双协议授权，你可以选择其中任意一种。

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
