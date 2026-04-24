# samsara

AI Agent 知识管理 CLI — 写入、晋升、校验和反思学习教训。

与 [`skm`](https://github.com/mocikadev/mocika-skills-cli) 共同构成 Samsara 知识系统的工具层：
- **skm** 管理 `~/.agents/skills/`（Layer 1，skill 包）
- **samsara** 管理 `~/.agents/knowledge/`（Layer 2，知识教训）

---

## 安装

**方式 1：一键安装（推荐）**

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
```

二进制安装到 `~/.local/bin/samsara`，无需 Rust 工具链。

```bash
# 指定版本
SAMSARA_VERSION=v0.1.0 bash <(curl -fsSL .../install.sh)

# 自定义安装目录
SAMSARA_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL .../install.sh)
```

**方式 2：从源码编译**

```bash
cargo install --path .   # 需要 Rust 1.88+
```

**系统要求**：git 已安装并在 PATH 中。

---

## 快速开始

```bash
# 1. 初始化知识库（创建 ~/.agents/ 目录结构）
samsara init

# 2. 写入一条教训
samsara write rust cargo-fmt --summary "提交前顺序：cargo fmt → clippy → test" --type error

# 3. 再次遇到同一问题，occurrences +1（不覆盖正文）
samsara write rust cargo-fmt

# 4. 验证规则有效（verified +1，影响推荐分）
samsara write rust cargo-fmt --verify

# 5. 搜索知识库
samsara search cargo                          # 全库搜索
samsara search rebase --domain git            # 限定 domain
samsara search fmt --type error               # 限定 lesson 类型
samsara search fmt --lessons-only             # 只搜 lessons
samsara search fmt --rules-only               # 只搜 rules

# 6. 查看知识库状态
samsara status

# 7. 查看操作日志
samsara log                   # 最近 20 条
samsara log --tail 5          # 最近 5 条
samsara log --action write    # 只看 WRITE 操作

# ── 以下命令在 v0.2 实装 ──────────────────────────

# 晋升为规则（occurrences ≥ 3 后）
samsara promote rust cargo-fmt

# 晋升到 AGENTS.md（100 行安全检查）
samsara promote rust cargo-fmt --layer0

# 检查知识库健康度
samsara lint [--fix]

# 分析学习模式
samsara reflect

# ── 以下命令在 v0.3 实装 ──────────────────────────

# 查看 Top N 推荐晋升规则
samsara prime
samsara prime --limit 5 --domain rust

# 归档不再活跃的教训
samsara archive rust old-pattern

# 从 AGENTS.md 降级规则
samsara demote cargo-fmt --yes

# 归档旧日志（保留最近 90 天）
samsara log --rotate --keep 90
```

---

## 命令列表

### ✅ v0.1 已实装

| 命令 | 说明 |
|------|------|
| `samsara init [--yes]` | 初始化知识库目录和工具映射 |
| `samsara write <domain> <keyword>` | 写入或更新教训（upsert 语义） |
| `samsara search <query>` | 按相关性搜索 lessons/rules |
| `samsara status` | 知识库统计摘要 |
| `samsara log [--tail N] [--action <type>]` | 查看操作日志 |

### ✅ v0.2 已实装

| 命令 | 说明 |
|------|------|
| `samsara promote <domain> <keyword> [--layer0]` | 晋升为规则 / 晋升到 AGENTS.md |
| `samsara lint [--fix]` | 检查知识库健康度（13 项） |
| `samsara reflect` | 静态分析学习模式 |
| `samsara skill-note <name> [--fail --note "..."]` | 记录 skill 使用结果 |
| `samsara domain list\|add` | 管理 domain |

### ✅ v0.3 已实装

| 命令 | 说明 |
|------|------|
| `samsara prime [--limit N] [--domain d] [--sort recent\|score]` | Top N 推荐晋升规则 |
| `samsara archive <domain> <keyword>` | 归档教训到 archive/ 目录 |
| `samsara demote <pattern> [--yes]` | 从 AGENTS.md 降级规则 |
| `samsara log --rotate [--keep N]` | 归档 N 天前的日志条目 |

### ✅ v0.4 已实装

| 命令 | 说明 |
|------|------|
| `samsara remote add\|set\|show` | 管理 git 远端地址 |
| `samsara push` | 推送 knowledge/ 到远端 git |
| `samsara pull` | 从远端 git 拉取并重建 INDEX |
| `samsara self-update [--check]` | 升级 samsara 到最新版本 |
| `samsara mcp serve` | 启动 MCP 服务（stdio 模式）|

---

## MCP 集成（✅ v0.4 已实装）

AI 工具可通过 MCP 协议直接调用 samsara，替代部分 `bash("samsara ...")` 调用。

### 配置方式（一次性手动配置）

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

> ⚠️ **注意**：OpenCode 与 Claude Code 的 MCP 配置格式**不同**。
> OpenCode 使用 `mcp.{name}.type + command`（数组）；Claude Code 使用 `mcpServers.{name}.command + args`。
> 混用格式会导致 MCP 无法启动。

配置后重启工具，AI 即可直接调用 MCP 工具。samsara 进程由工具按需自动启动（stdio 模式），**无需手动启动**。

### 暴露的 MCP 工具

| Tool | 等价 CLI | 说明 |
|------|----------|------|
| `write_lesson` | `samsara write` | 写入/更新 lesson |
| `search_knowledge` | `samsara search` | 检索知识库 |
| `get_status` | `samsara status` | 知识库统计 |
| `promote_lesson` | `samsara promote` | 晋升 lesson |
| `read_index` | —— | 获取 INDEX.md 完整内容 |
| `prime_context` | `samsara prime` | 生成紧凑上下文摘要 |

> v0.3 之前，所有操作均通过 `bash("samsara ...")` 调用 CLI，效果完全等同。

---

## 开发状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| v0.1 | ✅ 已完成（2026-04-23） | `init` / `write` / `search` / `status` / `log` |
| v0.2 | ✅ 已完成（2026-04-23） | `promote` / `lint` / `reflect` / `skill-note` / `domain` |
| v0.3 | ✅ 已完成（2026-04-24） | `prime` / `archive` / `demote` / `log --rotate` |
| v0.4 | ✅ 已完成（2026-04-24） | `push` / `pull` / `self-update` / `mcp serve` / `remote` |

详见 `docs/process.md` 和 `docs/samsara-engineering.md`。

---

## 许可证

MIT OR Apache-2.0
