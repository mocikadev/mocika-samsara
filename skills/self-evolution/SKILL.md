---
name: self-evolution
version: 0.4.0
description: Guide AI agents to use samsara CLI for knowledge management
tags: [samsara, knowledge, lesson, rules, agents-md]
compatible_agents: [opencode, claude-code, codex, gemini]
---

# self-evolution · AI 自我进化操作手册

> **实装状态**：全部命令已在 v0.4 实装（`init`/`write`/`search`/`status`/`log`/`promote`/`lint`/`reflect`/`skill-note`/`domain`/`prime`/`archive`/`demote`/`push`/`pull`/`remote`/`self-update`/`mcp serve`）。

## 何时使用本 skill

- 遇到可归纳的错误或教训时
- 学到新技能、模式或洞察后想持久化时
- 需要将高频规则晋升到 AGENTS.md 时
- 需要记录 skill 使用结果（成功 / 失败）时

---

## 基本工作流

### 1. 初始化知识库

```bash
samsara init          # 交互确认
samsara init --yes    # 跳过确认（AI 调用推荐）
```

创建 `~/.agents/knowledge/{lessons,rules,archive}/`、初始 INDEX.md、log.md、37 个 seed domain 目录。

### 2. 写入教训

```bash
samsara write <domain> <keyword> --summary "简明教训" [--type error|skill|pattern|insight] [--yes]
```

示例：
```bash
samsara write rust cargo-fmt --summary "提交前顺序：cargo fmt → clippy → test" --type error --yes
samsara write git rebase-stash --summary "rebase 前必须先 stash" --type skill --yes
```

- `--summary` 跳过编辑器直接写入正文，适合 AI 调用
- `--yes` 跳过新建确认
- keyword 已存在时：occurrences +1，不覆盖正文（upsert 语义）

### 3. 再次遇到同一问题

```bash
samsara write <domain> <keyword>
```

追加 occurrence（+1 次），不修改正文。

### 4. 验证规则有效

```bash
samsara write <domain> <keyword> --verify
```

`verified` 字段 +1，影响 `samsara prime` 推荐分（每次 +15 分）。

### 5. 搜索知识库

```bash
samsara search <query>                    # 全库搜索（lessons + rules）
samsara search <query> --domain <name>    # 限定 domain
samsara search <query> --type error       # 限定 lesson 类型
samsara search <query> --lessons-only     # 只搜 lessons
samsara search <query> --rules-only       # 只搜 rules
samsara search <query> --limit 5          # 最多返回 5 条
```

相关性评分规则（越高越靠前）：
- 文件名精确匹配 +100
- tag 精确匹配 +50
- domain 精确匹配 +40
- body 每处出现 +10（上限 +50）

### 6. 查看知识库状态

```bash
samsara status
```

输出：知识库路径、domain 数、lesson 数（总/已晋升/未晋升）、rules 文件数、log 条目数、未提交变更列表。

### 7. 查看操作日志

```bash
samsara log                    # 最近 20 条
samsara log --tail 5           # 最近 5 条
samsara log --action write     # 只看 WRITE 操作
samsara log --action promote   # 只看 PROMOTE 操作
```

---

## v0.2 命令（⏳ 尚未实装）

### 晋升为规则（occurrences ≥ 3）

```bash
samsara promote <domain> <keyword>            # 晋升到 rules/<domain>.md
samsara promote <domain> <keyword> --layer0   # 晋升到 AGENTS.md（需确认，100 行安全检查）
```

### 检查知识库健康度

```bash
samsara lint          # 13 项检查，输出 [ERROR]/[WARN]/[INFO]
samsara lint --fix    # 自动修复 ⑤⑥⑧⑪ 四项
```

### 分析学习模式

```bash
samsara reflect
```

输出：待晋升候选、高频 domain 建议、skill 失败统计、AAAK 候选条目。

### 查看 Top N 推荐规则（v0.3）

```bash
samsara prime [--limit 10] [--domain rust]
```

输出包含可执行的 `samsara promote --layer0` 命令，直接复制执行即可。

---

## MCP 集成（✅ v0.4 已实装）

在工具配置中注册一次，之后高频操作（write/search/promote）直接走 MCP 工具，无需 bash。

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

> ⚠️ OpenCode 与 Claude Code 格式**不同**：OpenCode 用 `mcp.{name}.type + command`（数组），Claude Code 用 `mcpServers.{name}.command + args`。混用会导致 MCP 启动失败。

- samsara 进程由工具**按需自动启动**（stdio 模式），无需手动启动
- 如无 MCP，用 `bash("samsara ...")` 调用 CLI 效果完全等同

---

## 命令速查

| 命令 | 状态 | 场景 |
|------|------|------|
| `samsara init [--yes]` | ✅ | 初始化知识库 |
| `samsara write <d> <k> --summary "..."` | ✅ | 写入 / 更新教训（无需编辑器） |
| `samsara write <d> <k> --verify` | ✅ | 验证规则有效（verified +1） |
| `samsara write <d> <k> --type error` | ✅ | 标记为错误教训 |
| `samsara search <q> [--domain d]` | ✅ | 按相关性搜索 lesson/rules |
| `samsara status` | ✅ | 知识库统计摘要 |
| `samsara log [--tail N] [--action t]` | ✅ | 查看操作日志 |
| `samsara promote <d> <k>` | ✅ | 晋升到 rules/<domain>.md |
| `samsara promote <d> <k> --layer0` | ✅ | 晋升到 AGENTS.md（100 行安全检查） |
| `samsara lint [--fix]` | ✅ | 检查 / 修复知识库（13 项） |
| `samsara reflect` | ✅ | 分析学习模式 |
| `samsara skill-note <name>` | ✅ | 记录 skill 使用成功 |
| `samsara skill-note <name> --fail` | ✅ | 记录 skill 失败 |
| `samsara prime [--limit N]` | ✅ | Top N 推荐晋升规则 |
| `samsara archive <d> <k>` | ✅ | 归档不活跃教训 |
| `samsara demote <pattern> [--yes]` | ✅ | 从 AGENTS.md 降级规则 |
| `samsara log --rotate [--keep N]` | ✅ | 归档旧日志 |
| `samsara remote add\|set\|show` | ✅ | 管理 git 远端地址 |
| `samsara push [--dry-run]` | ✅ | 推送 knowledge/ 到远端 git |
| `samsara pull` | ✅ | 从远端拉取并重建 INDEX |
| `samsara self-update [--check]` | ✅ | 升级到最新版本 |
| `samsara mcp serve` | ✅ | 启动 MCP 服务（stdio 模式）|

---

## 注意事项

- AI 读取 lesson 文件**不计入** occurrences，不重置 90 天计时器
- `write` 有 upsert 语义：keyword 已存在则追加 occurrence，不覆盖正文
- `promote --layer0` 有 100 行安全检查，超出时拒绝并引导 `samsara demote`（v0.2）
- `lint --fix` 仅自动修复 ⑤⑥⑧⑪ 四项，其余需人工处理（v0.2）
- `type` 字段影响 prime 评分：`error` +20，其余同等
- 多设备同步：`samsara push` / `samsara pull`（需先 `samsara remote add <url>`，v0.3）
