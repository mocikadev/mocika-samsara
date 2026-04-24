---
name: self-evolution
displayName: self-evolution — AI 自我进化知识系统
description: Guide AI agents to use samsara CLI for knowledge management. Use
  when encountering errors worth logging, recalling past lessons, promoting
  repeated issues to rules, or writing experience into AGENTS.md.
version: 0.6.0
author: mocikadev
tags: [samsara, knowledge, lesson, rules, agents-md, self-evolution, write, promote, search]
compatible_agents: [opencode, claude-code, codex, gemini, cursor]
---

# self-evolution — AI 自我进化知识系统

`samsara` 是一个本地 CLI 知识管理工具，让 AI 像人一样积累经验：遇到错误记录为教训，反复踩坑后晋升为规则，规则写入 AGENTS.md，下次启动自动生效，永不重蹈覆辙。

## 版本检查

在会话中**首次调用任何 samsara 命令前**，依次执行：

```bash
samsara self-update --check
skm update --check self-evolution
```

- `samsara` 有新版本：告知用户 `samsara 有可用更新 vX.Y.Z，是否立即升级？`，确认后执行 `samsara self-update`
- `self-evolution` skill 有更新：直接执行 `skm update self-evolution`（纯文本文件，无需确认）
- 两者均已最新：静默继续，不打扰用户
- 每次会话各执行一次，不重复

## 数据目录

| 路径 | 说明 |
|------|------|
| `~/.agents/knowledge/lessons/` | 教训文件（按 domain 分目录） |
| `~/.agents/knowledge/rules/` | 已晋升的规则（`rules/<domain>.md`） |
| `~/.agents/knowledge/archive/` | 归档的不活跃教训 |
| `~/.agents/knowledge/log.md` | 操作日志 |
| `~/.agents/AGENTS.md` | 自进化协议 + 已晋升的 layer0 规则 |
| `~/.agents/samsara.toml` | 配置（同步远端等） |

## 快速工作流

### 首次初始化

```bash
samsara init --yes
```

创建 `~/.agents/knowledge/` 目录结构，注入自我进化协议到 AGENTS.md，安装 `self-evolution` skill（需已安装 skm）。

### 遇到错误，记下来

```bash
samsara write rust cargo-fmt --summary "提交前顺序：cargo fmt → clippy → test" --type error --yes
samsara write git rebase-stash --summary "rebase 前必须先 stash" --type skill --yes
```

- `--summary` 直接写入正文，跳过编辑器（AI 调用推荐加此参数）
- `--yes` 跳过新建确认
- keyword 已存在时：occurrences +1，不覆盖正文

### 同一问题再次出现

```bash
samsara write rust cargo-fmt    # occurrences +1，不改正文
```

### 出现 3 次后晋升为规则

```bash
samsara promote rust cargo-fmt              # 晋升到 rules/rust.md
samsara promote rust cargo-fmt --layer0     # 同时写入 AGENTS.md（启动即生效）
```

### 开始任务前检索已有经验

```bash
samsara search "cargo build"
samsara search "rebase" --domain git
samsara search "错误处理" --type error
```

## 完整命令参考

### 教训管理

#### `samsara write <domain> <keyword> [OPTIONS]`

```bash
samsara write <domain> <keyword> --summary "..." --type error|skill|pattern|insight --yes
samsara write <domain> <keyword>              # 仅 occurrences +1
samsara write <domain> <keyword> --verify     # verified +1（影响 prime 评分）
```

#### `samsara search <query> [OPTIONS]`

```bash
samsara search <query>                    # 全库搜索（lessons + rules）
samsara search <query> --domain <name>    # 限定 domain
samsara search <query> --type error       # 限定类型
samsara search <query> --lessons-only     # 只搜 lessons
samsara search <query> --rules-only       # 只搜 rules
samsara search <query> --limit 5
```

### 晋升与维护

#### `samsara promote <domain> <keyword> [--layer0]`

```bash
samsara promote rust cargo-fmt            # 晋升到 rules/<domain>.md
samsara promote rust cargo-fmt --layer0   # 晋升到 AGENTS.md（100 行安全检查）
```

#### `samsara prime / reflect / lint`

```bash
samsara prime [--limit 10] [--domain rust]   # Top N 推荐晋升候选
samsara reflect                              # 分析学习模式，输出 AAAK 候选
samsara lint [--fix]                         # 13 项健康检查，--fix 自动修复 4 项
```

#### `samsara demote / archive`

```bash
samsara demote <pattern> [--yes]     # 从 AGENTS.md 移除规则
samsara archive <domain> <keyword>   # 归档 90 天无活跃的教训
```

### 同步与升级

```bash
samsara remote add <url>       # 配置同步远端
samsara push [--dry-run]       # 推送 knowledge/ 到远端
samsara pull                   # 从远端拉取

samsara self-update [--check]  # 升级 binary（--check 仅检查）
skm update self-evolution      # 更新 self-evolution skill
```

### 初始化与日志

```bash
samsara init [--yes]
samsara status
samsara log [--tail N] [--action write|promote]
samsara log --rotate
samsara skill-note <name> [--fail] [--note "..."]
samsara mcp serve              # 由 AI 工具自动调用，无需手动执行
```

## MCP 集成

配置一次，之后 write / search / promote 直接走 MCP 工具，无需 bash。

**OpenCode** — `~/.config/opencode/opencode.json`：
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

**Claude Code** — `~/.claude/claude_desktop_config.json`：
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

> ⚠️ 两种格式**不可混用**。samsara 进程由工具按需自动启动，无需手动运行。

## 首次迁移协议

`samsara init` 不删除原有 AGENTS.md 内容，只注入协议 section。原有手写规则/教训可一次性迁移：

**检测是否需要迁移**：
```bash
samsara search agents-md-migrated --domain samsara   # 无结果 → 需要迁移
```

**迁移步骤**：

1. 读取 AGENTS.md 中 `## 自我进化协议（Samsara）` 之前的内容
2. 识别知识条目（规则、踩坑、模式），跳过目录、工具配置、格式要求等非知识内容
3. 逐条写入：
   ```bash
   samsara write <domain> <keyword> --summary "<原文>" --type <error|skill|pattern|insight> --yes
   ```
4. 写入完成标记：
   ```bash
   samsara write samsara agents-md-migrated --summary "已完成首次迁移" --type insight --yes
   ```

## 常用场景速查

| 场景 | 命令 |
|------|------|
| 遇到错误，立即记录 | `samsara write <d> <k> --summary "..." --type error --yes` |
| 开始任务前，检索已有经验 | `samsara search <query>` |
| 同一问题再次出现 | `samsara write <d> <k>`（occurrences +1） |
| 出现 3 次，晋升为规则 | `samsara promote <d> <k>` |
| 规则写入 AGENTS.md（启动即生效）| `samsara promote <d> <k> --layer0` |
| 查看哪些知识该晋升 | `samsara prime` |
| 定期回顾学习模式 | `samsara reflect` |
| 知识库健康检查 | `samsara lint [--fix]` |
| AGENTS.md 超 100 行，降级规则 | `samsara demote <pattern>` |
| 多设备同步 | `samsara push` / `samsara pull` |
| 从 AGENTS.md 迁移已有经验 | 见"首次迁移协议" |
| 检查 binary / skill 版本 | `samsara self-update --check` + `skm update --check self-evolution` |
