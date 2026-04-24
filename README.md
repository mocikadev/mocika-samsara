# samsara

让 AI Agent 像人一样积累经验——遇到错误记下来，多次遇到晋升为规则，规则沉淀进 AGENTS.md。

与 [`skm`](https://github.com/mocikadev/mocika-skills-cli) 配合使用：
- **skm** 管理技能包（`~/.agents/skills/`）
- **samsara** 管理知识教训（`~/.agents/knowledge/`）

---

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

安装到 `~/.local/bin/samsara`，无需 Rust 环境，git 需在 PATH 中。

```bash
# 指定版本
SAMSARA_VERSION=v0.1.0 bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)

# 自定义安装目录
SAMSARA_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh)
```

---

## 初始化

```bash
samsara init
```

创建 `~/.agents/` 目录结构，自动安装 `self-evolution` skill（需已安装 skm），并将自进化协议注入到已检测到的 AI 工具配置中。

---

## 工作流

### 记录教训

```bash
# 遇到错误，写下来
samsara write rust cargo-fmt --summary "提交前顺序：cargo fmt → clippy → test" --type error

# 再次踩坑，occurrences +1（不覆盖正文）
samsara write rust cargo-fmt

# 验证规则有效，verified +1
samsara write rust cargo-fmt --verify
```

### 晋升为规则

```bash
# occurrences ≥ 3 后，晋升到 rules/rust.md
samsara promote rust cargo-fmt

# 晋升到 AGENTS.md（AI 每次启动都会读到）
samsara promote rust cargo-fmt --layer0
```

### 搜索

```bash
samsara search "cargo fmt"
samsara search rebase --domain git
samsara search fmt --type error
```

### 日常维护

```bash
samsara status                  # 知识库统计
samsara lint [--fix]            # 检查知识库健康度
samsara reflect                 # 分析学习模式
samsara prime                   # Top 10 推荐晋升的规则
samsara archive rust old-topic  # 归档不活跃教训
```

### 多设备同步

```bash
samsara remote add https://github.com/yourname/knowledge.git
samsara push
samsara pull
```

---

## AI 工具集成

### 第一步：配置 MCP

AI 工具通过 MCP 协议直接调用 samsara，无需手动执行命令。配置一次后自动生效。

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

> ⚠️ OpenCode 与 Claude Code 的配置格式不同，不可混用。

配置后重启工具即可，samsara 进程由工具按需启动，无需手动运行。

### 第二步：安装 self-evolution skill

```bash
skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all
```

或通过 `samsara init` 自动安装（需已安装 skm）。

后续升级：

```bash
skm update self-evolution
```

### AI 的调用方式

配置完成后，AI 通过 MCP 工具操作知识库：

| MCP 工具 | 触发场景 |
|----------|----------|
| `write_lesson` | 遇到错误或学到新知识时主动记录 |
| `search_knowledge` | 解决问题前先查已有经验 |
| `promote_lesson` | 某条教训反复出现，建议晋升 |
| `prime_context` | 任务开始时加载最相关的经验摘要 |
| `read_index` | 了解知识库全貌 |
| `get_status` | 查看知识库健康状态 |

skill 加载后，AI 会在合适的时机自动调用这些工具——无需用户提示。

---

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

---

## 许可证

MIT OR Apache-2.0
