# 轮回（Samsara）· AI 自我进化知识系统

> **设计文档 v0.8**  
> **状态**：设计阶段—规格修复完成，§18 命令规格已无"待定"字段；大幅结构重构推迟至 v0.9  
> **上次更新**：2026-04-23  
> **文档历史**：[v0.1](#版本历史) → v0.2（多工具兼容重构）→ v0.3（skm 整合）→ v0.4（occurrences 模型、调研来源记录）→ v0.5（§3.6 调研推导链、§18 samsara CLI 设计）→ v0.6（§3.7 daerwen 调研；AAAK 子层；samsara init/reflect/skill-note；git 集成；SKILL_USE/SKILL_FAIL）→ v0.7（AAAK 合并进 AGENTS.md；Domain 动态注册机制；补充 search/domain/log-rotate 命令；修复 §9.1 Step 1 逻辑；§12 改用 CLI 命令；§13 新增 log.md 约束；P-04/P-06 关闭）→ **v0.8**（竞品调研 §3.8；MCP §19；命令算法补全；occurrences 语义、100 行约束、push/pull 冲突策略、promote --layer0 安全算法、write upsert、conflicts_with 格式、lint 13 项 + --fix、prime 触发时机、SKILL_FAIL 位置；DNA Memory 整合：type/verified 字段、lint ⑬ 蒸馏候选、reflect 按 type 分组、prime 多维评分）

---

## 目录

1. [问题背景](#1-问题背景)
2. [命名与理念](#2-命名与理念)
3. [调研结果](#3-调研结果)
   - 3.5 [skm — Samsara 基础设施](#35-skm--samsara-基础设施)
   - 3.6 [调研发现对设计的影响](#36-调研发现对设计的影响)
   - 3.7 [Rust-daerwen 调研—进化引擎理念](#37-rust-daerwen-调研进化引擎理念)
   - 3.8 [竞品对标与安全考量](#38-竞品对标与安全考量)
4. [核心设计原则](#4-核心设计原则)
5. [三层架构](#5-三层架构)
6. [中央仓库设计](#6-中央仓库设计)
7. [工具兼容矩阵](#7-工具兼容矩阵)
8. [适配层策略](#8-适配层策略)
9. [完整工作流](#9-完整工作流)
10. [文件格式规范](#10-文件格式规范)
11. [AGENTS.md 自进化协议文本](#11-agentsmd-自进化协议文本)
12. [self-evolution Skill 内容](#12-self-evolution-skill-内容)
13. [防膨胀约束](#13-防膨胀约束)
14. [迁移计划](#14-迁移计划)
15. [已确认的技术事实](#15-已确认的技术事实)
16. [待决策项](#16-待决策项)
17. [版本历史](#17-版本历史)
18. [samsara CLI 设计](#18-samsara-cli-设计)
19. [MCP Server 设计](#19-mcp-server-设计)

---

## 1. 问题背景

### 痛点

AI 编程助手在使用过程中存在两类知识损耗：

1. **错误复发**：修复过的错误（如 `cargo fmt` 遗漏）在下次会话中重复出现，因为 AI 没有跨会话记忆。
2. **知识盲区重复探索**：每次遇到陌生 API / 工具 / 模式时重新搜索，结论不被保存。

### 现有方案的缺陷

| 现有方案 | 缺陷 |
|---------|------|
| 把所有规则堆入 AGENTS.md | 文件越来越大，信噪比下降 |
| 单一 `lessons-learned.md` | 不自动加载，AI 不会主动读；随时间膨胀 |
| 全依赖 skills/ | Skills 需要显式 load，不适合"每次都必须知道"的规则 |
| 绑定单一工具（如 OpenCode） | 切换工具后知识库全部失效，重复建设 |

### 目标

设计一个**工具无关、自动写入、按需读取、渐进晋升**的知识持久化系统，使 AI 能：

- 遇到知识盲区 → 搜索并记录，下次直接用
- 犯错修复后 → 自动归档，防止重犯
- 重复出现的模式 → 自动晋升为永久规则
- 主动发现并安装适合的 skill 包
- **切换工具时知识无缝迁移**，无需重建

---

## 2. 命名与理念

### 轮回（Samsara）

> 梵文 *saṃsāra*，轮回——存在在不同形态间的循环流转。

知识在会话间"轮回"复生，而不是随会话消亡：

- **TIL（今日所学）** → 积累 → **Lesson（经验）** → 反复验证 → **Rule（规则）** → 绝不违反 → **Core Memory（戒律）**
- 错误被记录，不重复犯——业力不灭
- 切换工具，知识依然在——轮回不绝
- 晋升是渐进的，不是跃迁——轮回是过程，不是终点

### 核心隐喻映射

| 轮回概念 | Samsara 系统对应 |
|---------|----------------|
| 业力（Karma）记录 | `lessons/` 原子化错误记录 |
| 轮回晋升 | TIL → Lesson → Rule → Core Memory 晋升机制 |
| 涅槃（解脱）| 知识足够完整，AI 不再犯同类错误 |
| 中阴（Bardo）| 会话间隙，知识持久化在文件系统 |

---

## 3. 调研结果

### 3.1 AGENTS.md 标准化现状（2026-04-22 调研）

**结论：AGENTS.md 已是事实标准，不是 OpenCode 私有格式。**

| 里程碑 | 时间 | 详情 |
|-------|------|------|
| OpenAI 发布 AGENTS.md | 2025年8月 | Codex CLI 首发，随即被 GitHub Copilot 采纳 |
| GitHub Copilot 原生支持 | 2025年8月 | [官方 Changelog](https://github.blog/changelog/2025-08-28-copilot-coding-agent-now-supports-agents-md-custom-instructions/) |
| 捐献 Linux Foundation | 2025年12月 | 成立 Agentic AI Foundation (AAIF)，[aaif.io](https://aaif.io) |
| v1.1 规范草案 | 2026年1月 | 支持 YAML frontmatter，向后兼容，[Issue #135](https://github.com/agentsmd/agents.md/issues/135) |

**创始白金成员**：AWS、Anthropic、Google、Microsoft、OpenAI、Cloudflare、Block、Bloomberg

**采用规模**：20+ 工具原生支持，60,000+ 开源项目（2026年4月）

**唯一不原生支持的主流工具**：Claude Code（使用 `CLAUDE.md`，但支持 `@路径` 导入语法）

### 3.2 主流工具全局指令文件对照表

| 工具 | 全局指令文件 | 用户级路径 | 自动加载 | AGENTS.md 支持 | 大小限制 |
|------|------------|---------|---------|----------------|--------|
| **OpenCode** | `AGENTS.md` | `~/.config/opencode/AGENTS.md` | ✅ | ✅ 原生 | 无明确限制 |
| **Codex CLI** | `AGENTS.md` / `AGENTS.override.md` | `~/.codex/AGENTS.md` | ✅ | ✅ 原生（创始者） | 32 KiB |
| **Claude Code** | `CLAUDE.md` | `~/.claude/CLAUDE.md` | ✅ | ❌（支持 `@import`） | 建议 ≤200 行 |
| **Gemini CLI** | `GEMINI.md` | `~/.gemini/GEMINI.md` | ✅ | ✅（agents.md 列出） | 无明确限制 |
| **Windsurf** | `global_rules.md` | `~/.codeium/windsurf/memories/global_rules.md` | ✅ | ✅ Wave 8+ | **6 KB**（全局） |
| **GitHub Copilot** | `AGENTS.md` / `copilot-instructions.md` | 项目级 `.github/` | ✅ | ✅ 原生 | 无明确限制 |
| **Cursor** | `AGENTS.md` / `.cursorrules` | 项目级 `.cursor/rules/` | ✅ | ✅ 原生 | 无明确限制 |
| **Aider** | `AGENTS.md` + `.aider.conf.yml` | `~/.aider.conf.yml` | ✅ | ✅ 原生 | 无明确限制 |
| **Continue.dev** | `.continue/rules/*.md` | `~/.continue/` | ✅ | 待确认 | 无明确限制 |
| **Cline** | `.clinerules/` | 项目级 | ✅ | 待确认 | 无明确限制 |

### 3.3 各工具配置目录结构（精确路径）

**Codex CLI** (`~/.codex/`)
```
~/.codex/
├── AGENTS.md                ← 全局指令
├── AGENTS.override.md       ← 覆盖（最高优先级）
└── config.toml              ← 行为配置（TOML）
```
加载优先级：`AGENTS.override.md` > `~/.codex/AGENTS.md` > `.codex/AGENTS.md`（项目级）

**Claude Code** (`~/.claude/`)
```
~/.claude/
├── CLAUDE.md                ← 全局指令（支持 @路径 导入）
├── settings.json            ← 行为配置（JSON）
├── skills/                  ← 用户级 skills（格式同 OpenCode）
├── agents/                  ← 子代理配置
└── projects/<hash>/memory/  ← 自动生成的项目记忆（机器本地）
```
加载优先级：企业策略 > `~/.claude/CLAUDE.md` > `./CLAUDE.md` > `./CLAUDE.local.md`

**Gemini CLI** (`~/.gemini/`)
```
~/.gemini/
├── GEMINI.md                ← 全局指令（Markdown）
└── settings.json            ← 行为配置（JSON）
```
加载优先级：系统 > 工作区 > 用户全局（4层）

**Windsurf** (`~/.codeium/windsurf/`)
```
~/.codeium/windsurf/
└── memories/
    └── global_rules.md      ← 全局规则（支持 YAML frontmatter，6KB 上限）
```
支持激活模式：`always_on` / `glob` / `model_decision` / `manual`

### 3.4 OpenCode 加载机制（实测确认）

```
确认事实 1：全局 AGENTS.md 每次会话自动加载 ✅
  路径：~/.config/opencode/AGENTS.md
  注入方式：直接进入系统提示

确认事实 2：项目级 AGENTS.md 与全局叠加，不是替代 ✅
  证据：当前会话同时出现两个 "Instructions from:" 条目

确认事实 3：skills/ 自动发现但不自动注入 ✅
  注入条件：AI 显式调用 skill({ name: "..." }) 工具

确认事实 4：新安装的 skill 需重启 OpenCode 才生效 ⚠️
  已知 bug：github.com/sst/opencode/issues/12741
  当前会话 workaround：直接读文件 ~/.config/opencode/skills/[name]/SKILL.md
```

### 3.5 skm — Samsara 基础设施

> skm 是 Samsara 系统的**底层基础设施**，由同一作者 vibe coding 完成，与 Samsara 协同迭代。

#### 两个仓库

| 仓库 | 说明 | 链接 |
|------|------|------|
| **mocika-skills-cli**（skm CLI） | Rust 编写的 AI Agent 技能包本地管理工具，版本 v0.1.2 | [github.com/mocikadev/mocika-skills-cli](https://github.com/mocikadev/mocika-skills-cli) |
| **skm-skill** | skm 的配套 skill 包，让 AI 直接用自然语言操作 skm | [github.com/mocikadev/skm-skill](https://github.com/mocikadev/skm-skill) |

#### skm 的核心职责

skm 管理 **Layer 1**（Domain Knowledge），负责：

- **统一存储**：所有 skill 包唯一存储于 `~/.agents/skills/`
- **多 Agent 部署**：通过 symlink 机制，一份 skill 文件同时服务多个 Agent
- **自动检测**：`skm scan` 检测本机已安装的 AI Agent，无需手动配置
- **版本管理**：更新前自动备份，支持快照级回滚
- **注册表搜索**：`skm search [关键词]` 搜索公开 skill 注册表（[skills.sh](https://skills.sh)）

#### ~/.agents/ 数据目录（skm 原有）

```
~/.agents/
├── skills/              ← Layer 1 唯一存储位置（skm 管理）
│   ├── skm/             ← skm-skill（已安装）
│   │   └── SKILL.md
│   └── [skm install 安装的其他 skills]/
├── .skill-lock.json     ← 安装元数据（与 skilly GUI 共用）
├── .skm-backups/        ← 技能备份快照
├── sources.toml         ← 注册表源配置
└── agents.toml          ← 已注册 Agent 配置
```

#### skm 已支持的 Agent（v0.1.2）

```
claude-code · codex · gemini-cli · copilot-cli · opencode
cursor · kiro · trae · trae-cn · junie
qoder · codebuddy · openclaw · antigravity
```

未列出的 Agent 可通过 `skm agent add` 手动注册。

#### skill 部署机制

```bash
# skm 安装 skill 并链接到所有 Agent（一条命令）
skm install mobile-android-design --link-to all

# 等价于 skm 自动完成：
#   ~/.agents/skills/mobile-android-design/  ← 源文件
#   ~/.config/opencode/skills/mobile-android-design → symlink
#   ~/.claude/skills/mobile-android-design   → symlink
#   ~/.codex/skills/mobile-android-design    → symlink（如支持）
#   ... 其他已注册 Agent
```

#### skm-skill 安装（推荐）

```bash
# 让 AI 直接操作 skm，无需记忆命令
skm install mocikadev/skm-skill --link-to all
```

安装后，AI Agent 可通过自然语言触发所有 skm 操作。

#### Samsara 与 skm 的职责边界

| 职责 | 负责方 |
|------|--------|
| skill 包的安装、更新、备份 | **skm** |
| skill 到各 Agent 目录的 symlink | **skm** |
| Agent 检测与注册 | **skm** |
| 知识库写入（lessons/rules） | **Samsara 协议** |
| 晋升机制（TIL → Rule → Core） | **Samsara 协议** |
| AGENTS.md 主协议文件维护 | **Samsara 协议** |
| 工具适配层（Claude/Gemini 等） | **Samsara 协议** |

---

## 3.6 调研发现对设计的影响

> 本节记录各调研来源的关键结论，以及它们如何具体影响了 Samsara 的设计决策。  
> 作为后续迭代的参考依据，避免设计决策失去来源而被误改。

### 3.6.1 MemPalace（27K stars）— 记忆检索机制

**调研时间**：2026-04-22  
**仓库**：github.com/mempalace/mempalace

MemPalace 是目前最受关注的 AI agent 记忆管理框架，核心是**不使用向量数据库，用结构化文件 + grep 实现记忆检索**。

#### 三个有实验数据的关键发现

| 发现 | 实验数据 | Samsara 对应决策 |
|------|---------|----------------|
| **Verbatim-first**：原文存储比摘要准确率高 | 96.6% vs ~82% | Lesson 文件保留根因原文，不压缩改写 |
| **结构过滤 > 向量搜索**：按 tag/domain 过滤比语义相似度匹配更准 | 94.8% vs ~88% | INDEX.md 的 domain tag 过滤机制，拒绝引入向量数据库 |
| **标记失效 > 物理删除**：直接删除造成 11% 的误删率 | 对照组 11% 数据丢失 | `valid_until:` 字段 + `archive/` 归档，lesson 永不物理删除 |

#### 核心启示

> **AI 的记忆检索不需要向量数据库**。文件系统 + 结构化 tag + grep = 足够准确，且零依赖、零运维。

这是 Samsara 拒绝引入 Chroma/FAISS 等向量存储的根本原因。

---

### 3.6.2 Karpathy LLM Wiki（2026-04-04 Gist）— 增量编译理念

**作者**：Andrej Karpathy  
**核心主张**：把 AI 知识库当作**代码仓库**来维护——LLM 一次修改 10-15 个文件，维护成本接近零；关键是要有**操作日志**。

#### 对 Samsara 的具体影响

1. **`log.md` 来源于此**：Karpathy 的设计中有 `log.md` 记录"谁、何时、做了什么操作"。AI 是无状态的，log 给它提供了操作历史上下文——相当于 `git log`，让 AI 知道"上次我在这个 lesson 里做了什么"。

2. **增量写入而非全量重建**：每次遇到知识盲区只写/更新一个文件，而不是重新整理整个知识库。这也是为什么 Lesson 用 `[keyword].md` 而不是 `YYYY-MM-DD-slug.md`——同一知识点永远是同一个文件，只追加时间戳。

---

### 3.6.3 Hermes Agent（Nous Research，95,600 stars）— 自动 Skill 生成

**仓库**：github.com/NousResearch/hermes-agent  
**核心能力**：从 agent 任务轨迹（工具调用序列 + 结果）**自动提炼生成 SKILL.md**，无需人工撰写。

#### 当前阶段：不采用，记录为 v1.0+ 演化方向

Samsara 现在用**主动触发**（AI 判断后手动写 lesson），原因：
- Hermes 的自动生成 pipeline 需要额外的模型调用（成本高）
- 主动触发更可控，符合 Samsara"渐进晋升"的哲学

**v1.0+ 演化路径**：任务结束后自动分析轨迹 → AI 决定是否生成/更新 lesson → 写入 Layer 2，替代纯手动触发。

---

### 3.6.4 Anthropic Agent Skills 规范 — 格式对齐

**来源**：agentskills.io/specification

Anthropic 提出的 skill 结构规范，SKILL.md 的 frontmatter 格式为：
```yaml
---
name: [slug]
displayName: [人类可读名]
description: [一句话描述，用于 AI 决定是否加载]
version: [semver]
tags: [tag1, tag2]
---
```

**Samsara 的 self-evolution/SKILL.md 严格遵循此格式**，原因：
- 跨工具可读性（Anthropic/OpenCode/其他工具的 skill loader 均能解析）
- 防止未来工具迁移时格式不兼容

---

### 3.6.5 Oracle 设计审查 — 发现的结构性缺陷

**时间**：2026-04-22，针对 Samsara v0.3 草案

Oracle 指出了两个结构性缺陷，均在 v0.4 中修复：

#### 缺陷 1：`recurrences: N` 整数计数（最脆弱的单点）

**问题**：整数只记录"发生了几次"，无法回答"最近一次是什么时候"。
- 若某个 rule 已有 3 次 recurrences 但距今已 2 年，它还适用吗？无从判断
- 整数被错误覆盖后不可恢复

**修复**：改为 `occurrences: ["2026-04-22", "2026-04-28"]` 时间戳数组：
- 保留完整历史，可随时计算"最近活跃度"
- 追加操作幂等安全（只追加，不覆盖）

#### 缺陷 2：Layer 0 缺少防误写机制

**问题**：协议未规定"写前检查是否已存在"，导致同一知识点会碎片化为多个文件（`cargo-fmt.md`、`cargo-fmt-vs-clippy.md`、`fmt-issue.md`）

**修复**：
1. 写入前强制 grep 查重：`grep -r "[关键词]" ~/.agents/knowledge/lessons/`
2. domain 预定义枚举列表（防止 `Rust`/`rust`/`cargo` 三个碎片 domain 并存）

---

### 3.6.6 设计取舍备忘

| 放弃的方案 | 放弃原因 | 来源依据 |
|-----------|---------|---------|
| 向量数据库（Chroma/FAISS） | 结构过滤更准（94.8% > 向量搜索），且零依赖 | MemPalace 实验 |
| 单一大文件（lessons-learned.md） | 无法按 domain 精准加载，随时间膨胀 | 原痛点分析 |
| 全自动写入（任务结束即生成） | 需要额外 pipeline，先用主动触发降复杂度 | Hermes 评估 |
| 物理删除过期 lesson | 11% 误删率风险 | MemPalace 实验 |
| 时间戳在文件名中（YYYY-MM-DD-slug.md） | 与 occurrences 数组语义重叠；upsert 语义要求同名文件 | Oracle 审查 + 架构一致性 |

---

## 3.7 Rust-daerwen 调研—进化引擎理念

**调研时间**：2026-04-22  
**来源**：Rust-daerwen 项目技术文章（私有未开源）

本节记录从该文章提炼的 4 点理念，以及它们对 Samsara 设计的具体影响。

### 3.7.1 AAAK（Always Available Ambient Knowledge）格式

**核心思路**：在 Layer 0 下划出一个超紧凑的子层，专门存放"每次对话都必须即时可用"的关键事实。

| 特性 | 规格 |
|------|------|
| 存储位置 | `~/.agents/AGENTS.md` 末尾的 `## AAAK` section（**不是独立文件**） |
| 条目格式 | `[entity\|relation\|value\|date]`，每行 ≈ 60 字符 |
| 预算上限 | ~120 tokens（≈ 480 字符），超预算时按 `date` 升序剔除最旧条目 |
| 写入触发 | `samsara promote --aaak` 时在 AGENTS.md 的 `## AAAK` section 追加/更新条目 |

**加载保证**：AAAK 条目随 AGENTS.md 由所有工具自动加载——无需单独配置，加载可靠性等同于 AGENTS.md 本身。原有独立 `aaak.md` 方案废弃，根本原因：没有任何工具会自动读取该孤立文件，"每次对话必须可用"的承诺无法兑现。

**示例（AGENTS.md 末尾片段）**：
```
## AAAK
<!-- auto-managed by `samsara promote --aaak` · budget: ~120 tokens · do not hand-edit -->
[cargo-fmt|must-run-before-commit|cargo fmt → clippy → test|2026-04-22]
[samsara-home|path|~/.agents/knowledge|2026-04-22]
[opencode-skill-bug|workaround|restart required after skm install|2026-04-22]
```

**对 Samsara 的影响**：Layer 0 维持单一 `AGENTS.md`，AAAK 作为末尾 section 附加，不需要独立文件，不需要额外加载配置。

---

### 3.7.2 Reflection Loop 理念

**核心思路**：进化的主要引擎不是写入，而是**定期回顾**——静态分析历史日志，发现待晋升候选、高频 domain、AAAK 建议。

对 Samsara 的影响：在 v0.2 中新增 `samsara reflect` 命令（**静态分析，无 LLM 调用**）：
- 扫描 `log.md`，统计各 domain 的 UPDATE 频率
- 识别 occurrences ≥ 3 但 promoted=false 的 lesson（待晋升）
- 识别高频 domain（建议安装对应 skill）
- 输出 AAAK 候选条目（高频出现的关键事实）

---

### 3.7.3 知识库 Git 版本控制

**核心思路**：知识库是一个 git repo，每次 `write`/`promote`/`archive` 操作后自动提交。

优势：
- 可 diff（每次进化都有完整的变更记录）
- 可回滚（误写可 `git revert`）
- 可审计（`git log` 就是进化历史）
- 比 `log.md` 纯文本更强：有真正的 diff 能力

对 Samsara 的影响：
1. `samsara init` 在初始化知识库目录时执行 `git init`
2. 每次写操作（write/promote/archive）完成后执行 `git add -A && git commit -m "samsara: <action> <target>"`
3. P-05（knowledge/ 是否纳入版本控制）的决策方向：**是，由 samsara 自动管理，无噪音**

---

### 3.7.4 Skill 使用追踪（skill_usage）

**核心思路**：记录每次 skill 加载和报错，是驱动 reflection 的关键原始数据。

对 Samsara 的影响：
1. `log.md` 新增两个 action 类型：`SKILL_USE`（成功使用）和 `SKILL_FAIL`（使用中出错）
2. 新增 `samsara skill-note <name> [--fail] "备注"` 命令，让 AI 在会话中一键记录
3. `samsara reflect` 分析 `SKILL_FAIL` 日志，识别需要修复的 skill

**示例日志行**：
```
2026-04-22 SKILL_USE  rust-skills (task: cargo build 优化)
2026-04-28 SKILL_FAIL rust-skills (load 报错: SKILL.md 缺少 tags 字段)
```

---

### 3.7.5 对取舍表的补充

| 放弃的方案 | 放弃原因 | 来源依据 |
|-----------|---------|---------|
| SQLite for log（FTS 加速） | log.md + grep 已足够；零依赖优先 | daerwen 评估（他也说"文件系统是最好的数据库"） |
| 自动 Reflection Loop（后台常驻） | 无 LLM 可用时静态分析仍有价值；先实现静态版本 | daerwen 三循环架构评估 |

---

## 3.8 竞品对标与差异化定位

> 来源：2026-04 竞品调研（Mem0、Letta、Zep、agentmemory、engram、memex-kb；Vercel 2025 实证；OWASP AISVS 2025）

### 3.8.1 主流方案全景

| 方案 | 类型 | 持久化 | 跨工具 | 版本控制 | 生命周期 | 离线 |
|------|------|--------|--------|----------|----------|------|
| Mem0 | SaaS API | 向量数据库 + 图 | API 调用 | ❌ | 部分 | ❌ |
| Letta | 框架（Python） | PostgreSQL | 框架内 | ❌ | ❌ | ✅ |
| Zep | SaaS/自托管 | 向量数据库 + Neo4j | API 调用 | ❌ | 部分 | ❌ |
| agentmemory | 库（Python） | ChromaDB | 单工具 | ❌ | ❌ | ✅ |
| engram | 库（Python） | SQLite | 单工具 | ❌ | ❌ | ✅ |
| memex-kb | CLI（Node.js） | Markdown + YAML | 部分 | ❌ | ❌ | ✅ |
| **Samsara** | **CLI（Rust）** | **纯 Markdown + Git** | **全兼容** | **✅ git** | **✅ 完整** | **✅** |

### 3.8.2 Samsara 核心差异化

**唯一同时满足以下全部条件的方案：**

1. **跨工具兼容**：AGENTS.md / CLAUDE.md / cursor rules 等平行格式，任何 CLI agent 切换无损失
2. **版本可控**：knowledge/ 为独立 git repo，进化历史可 diff 可回滚
3. **分层生命周期**：TIL → Rule → Core Memory 三层晋升，支持主动降级与归档
4. **纯文本无依赖**：Markdown + grep，无数据库、无嵌入模型、无 API 调用
5. **离线完全可用**：所有操作本地执行，无网络依赖

### 3.8.3 设计灵感来源

| 来源 | 启发点 | 应用到 Samsara |
|------|--------|---------------|
| **Mem0** | 自动记忆提取思路 | ❌ 否（需 embedding，违反无依赖原则） |
| **Letta** | MCP 工具接口（AI 主动调用） | ✅ §19 MCP Server 设计 |
| **memex-kb** | `prime` 紧凑上下文生成 | ✅ `samsara prime` 命令 |
| **Zep** | 知识冲突检测（图结构） | ✅ 简化为 `conflicts_with` 字段（§10.1） |
| **Vercel 2025** | AGENTS.md 可靠性 94% vs Skills 78%；>200 行被忽略 | ✅ 佐证 §13 防膨胀约束（≤100 行） |
| **OWASP ASI06** | 记忆投毒攻击成功率 70%+ | ✅ git 审计日志 + 人工触发写入为天然防御 |

### 3.8.4 安全考量（OWASP ASI06）

记忆投毒（Memory Poisoning）是 OWASP AISVS 2025 Top 10 安全威胁之一（ASI06）。Samsara 的防御机制：

- **人工触发写入**：knowledge 写入由 AI 执行 CLI 命令触发，不自动接受外部输入；用户可 audit
- **git 审计日志**：每次 write/promote/archive 均产生 git commit，攻击者注入可被 diff 发现
- **层级隔离**：Layer 0（AGENTS.md）只能通过 `samsara promote --layer0` 写入，门槛高于 Layer 2

---

## 4. 核心设计原则

1. **工具无关**：知识存在 `~/.agents/`，任何工具切换不损失记忆；skm 负责 skill 的多工具部署
2. **分层隔离**：不同生命周期的知识放不同层，互不污染
3. **写入自动，读取按需**：AI 主动写，但只在命中 domain 时读
4. **渐进晋升**：TIL（occurrences=1）→ 累计 → Rule（occurrences≥3）→ Core Memory（"绝不"级别）
5. **Skill 优先**：有现成 skill 包时优先安装，而非自己记录
6. **防膨胀有上限**：每一层都有明确的大小约束
7. **AGENTS.md 为核心协议**：采用 v1.1 规范，利用其行业标准地位实现跨工具兼容
8. **原文存储不摘要**：lesson 保留根因原文，不压缩改写 *（来源：MemPalace §3.6.1）*
9. **结构过滤不向量搜索**：domain tag + grep 过滤，拒绝引入向量数据库 *（来源：MemPalace §3.6.1）*
10. **标记失效不物理删除**：`valid_until` + `archive/` 归档 *（来源：MemPalace §3.6.1）*
11. **知识库 git 化**：`samsara init` 执行 `git init`，每次 write/promote/archive 自动提交；进化历史可 diff 可回滚 *（来源：daerwen §3.7.3）*
12. **AAAK 内嵌 AGENTS.md**：AAAK 条目作为 AGENTS.md 末尾 `## AAAK` section，随 AGENTS.md 自动加载，无需独立文件；预算 ~120 tokens，超出时剔除最旧条目 *（来源：daerwen §3.7.1；独立文件方案废弃原因：无任何工具自动加载保证）*

---

## 5. 三层架构

```
┌─────────────────────────────────────────────────────────┐
│  Layer 0 · Core Memory                                  │
│  ~/.agents/AGENTS.md（主文件，工具无关）                 │
│  • 绝大多数工具直接读取此文件                           │
│  • 只放跨所有任务的强制约束                             │
│  • 含自进化元协议（精简版，保持简洁）                    │
│  • 上限：实质规则 ≤ 100 行                             │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄        │
│  └─ ## AAAK section（内嵌于 AGENTS.md 末尾）         │
│     • 超紧凑关键事实，格式：[entity|relation|value|date] │
│     • 预算：~120 tokens，超出时剔除最旧条目             │
│     • 随 AGENTS.md 自动加载，无需独立文件               │
└─────────────────────────────────────────────────────────┘
           ↑ 只有"绝不"级别规则才晋升到此层

┌─────────────────────────────────────────────────────────┐
│  Layer 1 · Domain Knowledge                             │
│  ~/.agents/skills/（由 skm 统一管理）                   │
│  • 按需加载（显式 load_skills 或 AI 判断后 load）       │
│  • 完整的领域参考手册（如 rust-skills 179 条规则）      │
│  • 通用知识，非个人记录                                │
│  • skm install/update/backup 管理生命周期              │
│  • skm scan 自动检测 Agent，skm link 部署 symlink       │
└─────────────────────────────────────────────────────────┘
           ↑ 无 skill 覆盖时，知识沉入 Layer 2

┌─────────────────────────────────────────────────────────┐
│  Layer 2 · Self-Evolving Memory                         │
│  ~/.agents/knowledge/                                   │
│  • 个人错误记录、搜索所得知识片段、盲区总结             │
│  • 按 domain 分目录，原子化文件（[keyword].md）         │
│  • 任务开始时检查 INDEX.md，按需读取                   │
│  • occurrences 累计 ≥ 3 → 晋升到 rules/               │
│  • "绝不"级别 → 晋升 Layer 0                           │
│  ※ 原文存储不摘要（MemPalace: 96.6% vs 摘要 82%）     │
│  ※ 标记失效不删除（MemPalace: 删除有 11% 误删率）      │
└─────────────────────────────────────────────────────────┘
```

### 三层的严格边界

| 层 | 放什么 | **禁止放什么** |
|---|--------|--------------|
| **AGENTS.md（Layer 0）** | 跨所有任务的强制约束 + 自进化协议入口 | domain 细节、错误案例、技术清单 |
| **skills/（Layer 1）** | 完整领域参考手册（通用知识） | 个人犯错记录、临时笔记 |
| **knowledge/（Layer 2）** | 个人错误、搜索所得知识、盲区总结 | 通用知识（应在 skills） |

---

## 6. 中央仓库设计

### 6.1 目录结构

> `~/.agents/` 是 skm 的原生数据目录（**P-01 已解决**）。  
> Samsara 在此基础上新增 `AGENTS.md`、`knowledge/`、`adapters/` 三部分，与 skm 共存。

```
~/.agents/                               ← skm 数据目录 + Samsara 知识中枢（共用）
│
│  ── skm 原有文件（不要修改）──
├── .skill-lock.json                     ← skm 安装元数据（与 skilly GUI 共用）
├── .skm-backups/                        ← skm 技能备份快照
├── sources.toml                         ← skm 注册表源配置
├── agents.toml                          ← skm 已注册 Agent 配置
│
│  ── Layer 1：skm 管理（Samsara 只读/通过 skm 写入）──
├── skills/                              ← 所有 skill 包的唯一存储位置
│   ├── skm/                             ← skm-skill（操作 skm 的 AI 技能）
│   │   └── SKILL.md
│   ├── self-evolution/                  ← Samsara 自进化操作手册 skill（新建）
│   │   └── SKILL.md
│   └── [skm install 安装的其他 skills]/
│
│  ── Layer 0：Samsara 新增──
├── AGENTS.md                            ← 主协议文件（v1.1 格式）；末尾含 ## AAAK section（由 samsara promote --aaak 管理，勿手动编辑）
│
│  ── Layer 2：Samsara 新增──
├── knowledge/                           ← 自进化记忆（git repo，由 samsara init 初始化）
│   ├── .git/                            ← git 仓库（samsara 自动提交，进化历史可 diff）
│   ├── INDEX.md                         ← 导航枢纽（任务前扫一眼）
│   ├── log.md                           ← 操作日志（写入/晋升/安装事件流水账）
│   ├── lessons/                         ← 原子化 TIL（< 30 行/条）
│   │   ├── rust/
│   │   │   └── cargo-fmt-vs-clippy.md
│   │   ├── git/
│   │   ├── ci/
│   │   ├── api/
│   │   └── [domain]/
│   ├── rules/                           ← 晋升后的稳定规则（< 100 行/文件）
│   │   ├── rust.md
│   │   └── git.md
│   └── archive/                         ← 90 天无新 write 的 lesson 归档
│
│  ── Samsara 新增──
└── adapters/                            ← 仅需特殊处理的工具适配层
    ├── claude-code/
    │   ├── CLAUDE.md                    ← 内容：@~/.agents/AGENTS.md
    │   └── install.sh
    ├── gemini/
    │   ├── GEMINI.md                    ← 待确认：@import 或内容同步
    │   └── install.sh
    └── windsurf/
        ├── global_rules.md              ← 待确认：@import 或内容同步（注意 6KB 上限）
        └── install.sh
```

### 6.2 知识库 INDEX.md 结构

```markdown
# Samsara Knowledge Index

## Domain Map

| Domain | Tags | Lessons | Rules | 关联 Skill |
|--------|------|---------|-------|-----------|
| Rust | rust cargo fmt clippy | lessons/rust/ (1) | rules/rust.md | rust-skills |
| Git  | git commit rebase push | lessons/git/ (0) | — | — |
| CI   | ci github-actions fmt check | lessons/ci/ (0) | — | — |

## 最近写入

- 2026-04-22 · rust/cargo-fmt-vs-clippy.md (occurrences: 2)

## 已安装的 Skill

| Skill 名称 | 覆盖 Domain | 安装时间 | 安装原因 |
|-----------|------------|---------|---------|
| rust-skills | rust | 预装 | Rust 开发规范 |
| skm | skill-management | 预装 | skm 命令操作 |
```

---

## 7. 工具兼容矩阵

### 7.1 接入方式分类

**A 类：直接 symlink（零适配成本）**

这些工具原生支持 AGENTS.md，直接将工具的全局指令文件 symlink 到 `~/.agents/AGENTS.md`：

```bash
ln -sf ~/.agents/AGENTS.md ~/.config/opencode/AGENTS.md   # OpenCode
ln -sf ~/.agents/AGENTS.md ~/.codex/AGENTS.md             # Codex CLI
```

> **注意**：skills 的 symlink 由 `skm link` / `skm install --link-to` 统一管理，无需手动操作。

| 工具 | symlink 目标 | 验证命令 |
|------|------------|---------|
| OpenCode | `~/.config/opencode/AGENTS.md` | 启动会话查看 Instructions from |
| Codex CLI | `~/.codex/AGENTS.md` | `codex --show-config` |
| Cursor | `./AGENTS.md`（项目级） | Cursor Settings > Rules |
| Aider | `./AGENTS.md`（项目级） | `aider --show-prompt` |

**B 类：@import 注入（一行适配）**

这些工具有不同文件名，但支持 `@路径` 语法导入外部文件：

```bash
# Claude Code：~/.claude/CLAUDE.md
echo "@~/.agents/AGENTS.md" > ~/.claude/CLAUDE.md
```

| 工具 | 全局文件 | 适配内容 |
|------|---------|---------|
| Claude Code | `~/.claude/CLAUDE.md` | `@~/.agents/AGENTS.md` |

**C 类：内容同步（待调研确认）**

这些工具是否支持 `@import` 语法尚未确认，可能需要内容同步脚本：

| 工具 | 全局文件 | 注意事项 |
|------|---------|---------|
| Gemini CLI | `~/.gemini/GEMINI.md` | 是否支持 `@import`：**待确认** |
| Windsurf | `~/.codeium/windsurf/memories/global_rules.md` | **6KB 上限**，需裁剪版本；**待确认** |

**D 类：项目级（无用户级全局文件）**

这些工具没有用户级全局指令文件，只能在每个项目中单独配置：

| 工具 | 项目级文件 | 说明 |
|------|----------|------|
| GitHub Copilot | `.github/copilot-instructions.md` + `AGENTS.md` | 无全局用户文件 |
| Continue.dev | `.continue/rules/*.md` | 待确认是否有全局路径 |
| Cline | `.clinerules/` | 待确认 |

### 7.2 Skills 跨工具复用

**skm 统一管理 skill symlink**，无需手动为每个工具配置：

```bash
# 安装并一键链接到所有已检测 Agent
skm install self-evolution --link-to all

# 新装了一个 Agent？一条命令补齐所有 skill 链接
skm relink opencode
```

| 工具 | Skills 路径（skm 管理的 symlink 目标） | skm 支持 |
|------|--------------------------------------|---------|
| OpenCode | `~/.config/opencode/skills/[name]` | ✅ |
| Claude Code | `~/.claude/skills/[name]` | ✅ |
| Codex CLI | （需确认 skill 目录路径） | ✅（已在 agents.toml 中） |
| Gemini CLI | `~/.gemini/skills/[name]`（待确认） | ✅（已在 agents.toml 中） |
| Windsurf | （需确认 skill 目录路径） | ✅（已在 agents.toml 中） |
| Cursor | （需确认 skill 目录路径） | ✅（已在 agents.toml 中） |

---

## 8. 适配层策略

### 8.1 install.sh 统一安装脚本（草稿）

```bash
#!/usr/bin/env bash
# ~/.agents/install.sh
# 建立 Samsara 到各工具的 AGENTS.md 映射
# 注意：skills 的部署由 skm 管理，本脚本只处理 AGENTS.md 和知识库路径

AGENTS_HOME="$HOME/.agents"

# 前置检查：skm 必须已安装
if ! command -v skm &>/dev/null; then
  echo "❌ 请先安装 skm："
  echo "   curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash"
  exit 1
fi

setup_knowledge_dirs() {
  mkdir -p "$AGENTS_HOME/knowledge/lessons"/{rust,git,ci,api}
  mkdir -p "$AGENTS_HOME/knowledge"/{rules,archive}
  mkdir -p "$AGENTS_HOME/adapters"/{claude-code,gemini,windsurf}
  touch "$AGENTS_HOME/AGENTS.md"
  touch "$AGENTS_HOME/knowledge/INDEX.md"
  echo "✅ 知识库目录初始化完成"
}

setup_opencode() {
  mkdir -p "$HOME/.config/opencode"
  ln -sf "$AGENTS_HOME/AGENTS.md" "$HOME/.config/opencode/AGENTS.md"
  echo "✅ OpenCode AGENTS.md 映射完成"
}

setup_codex() {
  mkdir -p "$HOME/.codex"
  ln -sf "$AGENTS_HOME/AGENTS.md" "$HOME/.codex/AGENTS.md"
  echo "✅ Codex CLI AGENTS.md 映射完成"
}

setup_claude_code() {
  mkdir -p "$HOME/.claude"
  if [ ! -f "$HOME/.claude/CLAUDE.md" ]; then
    echo "@$AGENTS_HOME/AGENTS.md" > "$HOME/.claude/CLAUDE.md"
    echo "✅ Claude Code CLAUDE.md 创建完成"
  else
    echo "⚠️  ~/.claude/CLAUDE.md 已存在，请手动添加：@$AGENTS_HOME/AGENTS.md"
  fi
}

setup_skills() {
  # skm 负责 self-evolution skill 的安装和链接
  echo "正在安装 Samsara 核心 skill..."
  skm install mocikadev/skm-skill --link-to all
  # self-evolution skill 需要先本地创建再 link
  mkdir -p "$AGENTS_HOME/skills/self-evolution"
  skm relink  # 补齐所有 Agent 的 skill 链接
  echo "✅ Skills 部署完成（由 skm 管理）"
}

setup_gemini() {
  # TODO：确认 Gemini CLI 是否支持 @import
  echo "⚠️  Gemini CLI 适配待确认（@import 支持情况未知，见 P-03）"
}

setup_windsurf() {
  # TODO：确认 Windsurf global_rules.md 是否支持 @import（6KB 上限）
  echo "⚠️  Windsurf 适配待确认（见 P-03）"
}

# 执行
setup_knowledge_dirs
setup_opencode
setup_codex
setup_claude_code
setup_skills
setup_gemini
setup_windsurf

echo ""
echo "✅ Samsara 初始化完成"
echo "   知识库：$AGENTS_HOME/knowledge/"
echo "   主协议：$AGENTS_HOME/AGENTS.md"
echo "   已安装 Agent："
skm agent list
```

### 8.2 AGENTS.md Windsurf 裁剪版（预备方案）

Windsurf 全局文件有 6KB 上限，若不支持 @import，需维护一个裁剪版：

```
~/.agents/adapters/windsurf/
├── global_rules.md      ← 裁剪版（< 6KB，仅核心规则 + knowledge 路径引用）
└── sync.sh              ← 从 AGENTS.md 自动生成裁剪版的脚本
```

---

## 9. 完整工作流

### 9.1 知识写入流程

```
触发条件（AI 自行判断，无需用户提醒）
  ├─ A. 知识盲区：不熟悉的 API / 工具 / 模式
  ├─ B. 犯错被修复后
  └─ C. 用户说"记住这个"

  ↓

Step 1：本地查重
  检查 ~/.agents/knowledge/lessons/<domain>/<keyword>.md 是否存在
  （或直接运行 `samsara write <domain> <keyword>`，CLI 自动处理查重）
  ├─ 文件已存在 → 追加时间戳到 occurrences 数组 → 检查是否需要晋升
  └─ 文件不存在 → Step 2

  ↓

Step 2：搜索 skill 注册表（优先复用现成知识）
  skm search [关键词]
  ├─ 有合适 skill（AI 判断相关性，不盲目安装）
  │   → skm install [name] --link-to opencode
  │   → 记录到 INDEX.md："已安装 [name] 覆盖 [domain]"
  │   → 当前会话：直接读 ~/.agents/skills/[name]/SKILL.md
  └─ 无合适 skill → Step 3

  ↓

Step 3：写入 lessons/（原子化记录）
  路径：~/.agents/knowledge/lessons/[domain]/[keyword].md
  同时：更新 INDEX.md 的 domain 条目

  ↓

Step 4：晋升判断（每次写 lesson 时触发）
  len(occurrences) >= 3
  ├─ 是 → 提炼核心规则写入 ~/.agents/knowledge/rules/[domain].md
  │        在 lesson 文件头部标记 promoted: true
  │        在 INDEX.md 更新 Rules 列
  └─ 规则属于"绝不"/"必须"级别
      → 追加 1 行到 ~/.agents/AGENTS.md 自进化协议节下方
```

### 9.2 知识读取流程

```
任务开始时
  ↓
识别 domain（Rust? Git? CI? API? React? Docker?）
  ↓
读 ~/.agents/knowledge/INDEX.md（快速扫描，< 5 秒）
  ├─ 命中 domain → 读取对应 lessons/ + rules/ 文件
  │               → 用历史知识武装当前任务
  └─ 未命中 → 直接开始任务

任务过程中如遇知识盲区
  → 触发 9.1 写入流程
```

### 9.3 Skill 自动发现流程

```
AI 遇到知识盲区，domain = X
  ↓
  1. skm search [X 相关关键词]

  2. AI 评估搜索结果（相关性判断标准）
     ✅ 安装条件：skill 描述与当前 domain 高度匹配，且预期会多次用到
     ❌ 跳过条件：skill 功能与需求无关，或只是一次性问题

  3. 决定安装
     skm install [skill-name] --link-to opencode

  4. 记录到 INDEX.md
     | [skill-name] | [domain] | [日期] | [安装原因一句话] |

  5. 当前会话使用（绕过缓存 bug）
     直接读文件：~/.agents/skills/[name]/SKILL.md

  6. 下次会话
     重启工具 → load_skills=["skill-name"] 正常可用
```

---

## 10. 文件格式规范

### 10.1 Lesson 文件（`knowledge/lessons/[domain]/[keyword].md`）

```markdown
---
date: 2026-04-22
domain: rust
type: error                # 可选；取值：error | skill | pattern | insight
                           # 不填时 prime/reflect 按"未分类"处理
tags: [rust, cargo, fmt, clippy, ci]
occurrences: ["2026-04-22", "2026-04-28"]  # 时间戳数组，比整数计数可回溯（来源：Oracle §3.6.5）
promoted: false
verified: 0                # 可选；规则被验证有效的次数（samsara write --verify 递增）
valid_until:               # 可选，填写后超期自动归档；不填则依赖 lint 周期检查（来源：MemPalace §3.6.1）
conflicts_with: []         # 可选，格式 "<domain>/<keyword>"（不含 .md，支持跨 domain）
                           # 例：["rust/cargo-fmt", "git/rebase-vs-merge"]
                           # lint ⑫ 检查引用有效性，失效时报 WARN（不自动删除）
---

# cargo fmt 与 clippy 是独立检查

## 根因
`cargo clippy -- -D warnings` 通过 ≠ `cargo fmt --check` 通过
两者检查不同内容，互不依赖。

## 规则
提交前顺序：cargo fmt → cargo clippy -- -D warnings → cargo test

## 来源
CI 报错 "Diff in src/cli/doctor.rs"，本地 clippy 通过但 fmt 未跑
```

**约束**：
- 文件上限 **30 行**
- frontmatter 必填字段：`date`, `domain`, `tags`, `occurrences`, `promoted`
- 可选字段：`type`（error|skill|pattern|insight）、`verified`（整数，初始 0）、`valid_until`、`conflicts_with`
- `conflicts_with` 值格式：`["<domain>/<keyword>"]`（不含 `.md`，支持跨 domain），例：`["rust/cargo-fmt"]`
- 标题一句话说清楚问题
- 根因 + 规则 必填，来源可选

### 10.2 Rules 文件（`knowledge/rules/[domain].md`）

```markdown
# Rust Rules

> 由 lessons/rust/ 晋升，记录稳定的 Rust 开发约束

## 提交前检查顺序

来源：lessons/rust/cargo-fmt-vs-clippy.md（occurrences: 3）

cargo fmt → cargo clippy -- -D warnings → cargo test → git commit

原因：fmt 和 clippy 独立，clippy 通过不代表 fmt 通过；CI 同时检查两者。

---
<!-- 新规则追加在此 -->
```

**约束**：文件上限 **100 行**，每条规则注明来源 lesson 和 occurrences 次数

### 10.3 AGENTS.md v1.1 frontmatter（可选）

```yaml
---
name: "Samsara"
description: "AI 自我进化知识系统 - 个人全局规则与自进化协议"
version: "0.2"
---
```

### 10.4 AAAK 条目格式（`~/.agents/AGENTS.md` 的 `## AAAK` section）

AAAK 存储在 AGENTS.md 末尾的独立 section，由 `samsara promote --aaak` 自动管理，**不手动编辑**：

```markdown
## AAAK
<!-- auto-managed by `samsara promote --aaak` · budget: ~120 tokens · do not hand-edit -->
[cargo-fmt|must-run-before-commit|cargo fmt → clippy → test|2026-04-22]
[samsara-home|path|~/.agents/knowledge|2026-04-22]
[opencode-skill-bug|workaround|restart required after skm install|2026-04-22]
```

**条目规则**：
- 格式：`[entity|relation|value|date]`，每行 ≤ 80 字符
- `entity`：知识主体（工具名/命令/概念）
- `relation`：关系类型（`must-run-before`/`path`/`workaround`/`version` 等）
- `value`：事实值（简洁，优先英文）
- `date`：写入日期（`YYYY-MM-DD`），用于超出预算时按升序剔除最旧

**写入触发**：
- `samsara promote --aaak` — 晋升 lesson 时同步在 `## AAAK` section 追加/更新条目
- `samsara reflect` 输出 AAAK 候选条目建议，用户确认后执行 `samsara promote --aaak`

**管理机制**：`samsara promote --aaak` 写入后检查 `## AAAK` section 总字符数，超出 480 字符时按 `date` 升序删除最旧条目，直至符合预算。

---

## 11. AGENTS.md 自进化协议文本

以下文字追加到 `~/.agents/AGENTS.md`（其他工具通过 symlink 或 @import 自动获取）：

```markdown
## 自我进化协议（Samsara）

### 触发条件（AI 自行判断，无需用户提醒）
- 遇到知识盲区 / 犯错被修复后 / 用户说"记住这个"

### 标准流程
1. `skm search [关键词]` → 有合适 skill → `skm install [name]` → 记录到 INDEX.md
2. 无 skill → `samsara write <domain> <keyword>`（自动查重、frontmatter、git commit）
3. `len(occurrences) ≥ 3` → `samsara promote <domain> <keyword>`
   （occurrences 以 lesson 文件 frontmatter 数组为权威数据源；CLI 写入/输出仅作提示）
4. "绝不/必须"级别 → `samsara promote --layer0 <domain> <keyword>`（晋升 AGENTS.md）

> domain 机制与 seed domain 完整列表见 `~/.agents/skills/self-evolution/SKILL.md`

### 任务开始时
读 `~/.agents/knowledge/INDEX.md` → 按以下双重匹配规则加载命中的 lessons/rules 文件：
- **domain 匹配**：任务描述中出现的语言/平台/领域名（如 rust/git/android）→ 加载对应 `rules/<domain>.md`
- **tags 匹配**：任务描述关键词与 INDEX.md 中各 lesson 的 tags 比对 → 命中则加载对应 lesson 文件
- INDEX.md 不存在（首次使用或损坏）→ 先执行 `samsara init`，不静默失败

### 防膨胀约束
- AGENTS.md 实质规则 ≤ 100 行（不含 ## AAAK section）；单个 lesson ≤ 30 行
- rules/[domain].md ≤ 100 行；log.md ≤ 1000 行（超出时 `samsara log rotate`）
- lessons/ 90 天无新 write → `samsara archive`；AAAK 预算 ≤ 120 tokens（超出删最旧）

### Skill 使用记录
- 成功 → `samsara skill-note <name> "备注"` ；失败 → `samsara skill-note <name> --fail "原因"`
  （两者均追加一行到 `log.md`，格式：`SKILL_USE <name>` / `SKILL_FAIL <name> (<原因>)`）

### AAAK section（勿手动编辑）
<!-- 由 samsara promote --aaak 自动维护 -->
```

---

## 12. self-evolution Skill 内容

`~/.agents/skills/self-evolution/SKILL.md` 完整内容草稿：

```markdown
---
name: self-evolution
displayName: 自我进化工作流（Samsara）
description: Samsara 知识系统的完整操作手册。遇到知识盲区或犯错时加载。包含写入流程、晋升规则、skill 搜索安装方法和文件格式规范。
version: 0.4.0
author: personal
tags: [meta, knowledge, self-evolution, workflow, samsara]
---

# Samsara 自我进化工作流手册

本 skill 是 AGENTS.md 中自进化协议的详细版。需要执行自进化操作时加载此 skill。

## 知识库位置

| 文件/目录 | 用途 |
|---------|------|
| `~/.agents/knowledge/INDEX.md` | 导航枢纽，任务开始前读 |
| `~/.agents/knowledge/log.md` | 操作日志，记录每次写入/晋升/安装事件 |
| `~/.agents/knowledge/lessons/[domain]/` | 原子化 TIL（`[keyword].md`），< 30 行/条 |
| `~/.agents/knowledge/rules/[domain].md` | 晋升后的稳定规则 |
| `~/.agents/skills/[name]/` | 已安装的领域知识 skill |

## Lesson 写入模板

路径：`~/.agents/knowledge/lessons/[domain]/[keyword].md`

**写入前先查重**（推荐直接用 CLI，自动处理查重、frontmatter 和 git commit）：
```bash
samsara write <domain> <keyword>
```

或手动检查：
```bash
ls ~/.agents/knowledge/lessons/<domain>/<keyword>.md
```
- 有同名文件 → 追加时间戳到 `occurrences` 数组，不新建文件
- 无记录 → 新建

必填字段：`date`（创建日）, `domain`, `tags`, `occurrences`（初始为 `["YYYY-MM-DD"]`）, `promoted`（初始为 false）
可选字段：`valid_until`（填写则超期自动归档）
结构：根因 + 规则（必填），来源（可选）
上限：30 行

## 晋升规则

| 条件 | 动作 |
|------|------|
| `len(occurrences) ≥ 3` | 提炼核心规则写入 `rules/[domain].md`；lesson 标记 `promoted: true` |
| 规则属于"绝不/必须"级别 | 追加 1 行到 `~/.agents/AGENTS.md` |

**每次晋升后，追加一行到 `log.md`**：
```
2026-04-22 PROMOTE rust/cargo-fmt-vs-clippy → rules/rust.md
```

## Skill 搜索安装流程

```bash
# 1. 搜索
skm search [关键词] --limit 10

# 2. 安装（AI 判断相关性后决定）
skm install [name] --link-to opencode

# 3. 当前会话立即使用（绕过缓存 bug）
# 直接读文件：~/.agents/skills/[name]/SKILL.md
```

安装后更新 INDEX.md 的"已安装 Skill"表格。

## 防膨胀检查

每次写入时顺手检查（或统一用 CLI）：
```bash
# 全量检查（推荐，统一入口）
samsara lint

# 查看知识库状态
samsara status

# 查看操作日志
samsara log --tail 20
```

发现问题 → `samsara archive <domain> <keyword>` 归档，或手动编辑后再次执行 `samsara lint` 验证。

lint ⑨ 报告 AGENTS.md 超 100 行，或 reflect 报告 3+ 条晋升候选时 → `samsara prime` 辅助决策（只输出到 stdout，无副作用）。

## Lint 周期检查（每月一次或手动触发）

```bash
samsara lint
```

输出按严重程度分组（ERROR / WARN / INFO），覆盖：过期 lesson、孤立 lesson、超行数、rules 引用失效、AGENTS.md 行数、log.md 膨胀。发现问题后在 `log.md` 追加一行记录（CLI 自动完成）。

```

---

## 13. 防膨胀约束

| 文件/层级 | 硬上限 | 触发清理的条件 |
|---------|--------|--------------|
| `AGENTS.md` 实质规则行数（不含 ## AAAK section） | **100 行** | 超出时找最低优先级规则降级到 knowledge/ |
| 单个 lesson 文件 | **30 行** | 超出时拆分为多条 lesson |
| `rules/[domain].md` | **100 行** | 超出时审查是否有可合并/删除的规则 |
| `knowledge/log.md` | **1000 行**（约 1 年） | 超出时 `samsara log rotate --keep 90d` 保留最近 90 天，旧日志存入 `log.archive-YYYY.md` |
| Windsurf `global_rules.md` | **6 KB**（工具硬限制） | 维护专用裁剪版 |
| `lessons/[domain]/` 保留时长 | **90 天** | 90 天无新 write → 移入 `knowledge/archive/`（引用 = `samsara write` 同一 keyword；读取不计入） |
| INDEX.md Domain Map | 无硬上限 | 无 lesson 无 rule 的 domain 行应删除 |

---

## 14. 迁移计划

### 14.1 现有内容迁移（OpenCode → Samsara）

| 现有内容 | 迁移目标 | 操作 |
|---------|---------|------|
| `~/.config/opencode/AGENTS.md` | `~/.agents/AGENTS.md` | 移动，原路径改为 symlink |
| `~/.config/opencode/skills/` | `~/.agents/skills/` | 移动，原路径改为 symlink |
| `~/.config/opencode/docs/process/lessons-learned.md` LL-001 | `~/.agents/knowledge/lessons/rust/cargo-fmt-vs-clippy.md` | 转换格式后写入 |
| `~/.config/opencode/docs/process/lessons-learned.md` 通用清单 | `~/.agents/knowledge/rules/[lang].md` | 拆分后写入 |

### 14.2 初始化命令

> ⚠️ **本节 shell 命令已由 `samsara init` 替代**（见 §18.2），仅保留供理解底层操作，不建议直接执行。

```bash
# 推荐：一条命令完成所有初始化
samsara init

# samsara init 完成的操作包括：
# - 创建 ~/.agents/knowledge/ 目录结构（含 37 个种子 domain 目录）
# - 初始化 knowledge/ 为 git repo + .gitattributes（log.md union merge）
# - 创建 ~/.agents/AGENTS.md（含 ## AAAK section 占位）
# - 建立 A 类工具 symlink（OpenCode / Codex CLI）
# - 注入 B 类工具 @import（Claude Code）
# - 安装 self-evolution skill（via skm，若 skm 已安装）
```

### 14.3 验证清单

```bash
# OpenCode
cat ~/.config/opencode/AGENTS.md | head -5   # 应显示 Samsara AGENTS.md 内容

# Codex CLI
cat ~/.codex/AGENTS.md | head -5             # 应显示 Samsara AGENTS.md 内容

# Claude Code
cat ~/.claude/CLAUDE.md                      # 应显示 @~/.agents/AGENTS.md
```

---

## 15. 已确认的技术事实

| 事实 | 确认方式 | 影响 |
|------|---------|------|
| AGENTS.md 由 Linux Foundation 正式治理 | 官方公告 + aaif.io | 长期稳定，可作为核心协议格式 |
| 20+ 工具原生支持 AGENTS.md | agents.md 官网 + 各工具文档 | 跨工具兼容大多数情况免适配 |
| Claude Code 不原生支持 AGENTS.md | 官方文档 | 需要 @import 适配层 |
| Claude Code 支持 `@路径` 导入语法 | 官方 Memory 文档 | 可用一行完成适配 |
| Codex CLI `~/.codex/AGENTS.md` 全局加载 | 源码 agents_md.rs L96-113 | 直接 symlink 即可 |
| Gemini CLI 用 `~/.gemini/GEMINI.md` | 官方文档 + 源码 paths.ts | 需要单独适配，@import 支持待确认 |
| Windsurf 全局规则有 6KB 上限 | 官方文档 | 需维护裁剪版，不能直接 symlink |
| OpenCode 全局 AGENTS.md 每次会话自动加载 | 实测（会话 Instructions from 列表） | 自进化协议放这里 100% 可靠 |
| OpenCode 项目 AGENTS.md 与全局叠加（非替代） | 实测（两个 Instructions from 同时出现） | 不需要在每个项目重复写协议 |
| OpenCode skills 自动发现但不自动注入 | 源码分析 + 实测 | 需要显式 load_skills 或 AI 主动调用 |
| 新安装 skill 需重启 OpenCode 才生效 | 已知 bug #12741 | 当前会话 workaround：直接读文件 |
| `~/.agents/` 是 skm 的原生数据目录 | mocika-skills-cli README | P-01 解决：Samsara 以此为中央目录 |
| skm 支持 14 个 Agent，含本文涉及的全部主流工具 | mocika-skills-cli v0.1.2 源码 | skills 的跨工具部署由 skm 统一管理 |
| skm 通过 symlink 实现 skill 多 Agent 部署 | README 数据目录说明 | Layer 1 的跨工具兼容问题由 skm 解决 |
| skm-skill 使 AI 可直接用自然语言操作 skm | skm-skill README | AI 无需记忆 skm 命令，降低使用门槛 |

---

## 16. 待决策项

> 以下项目标注优先级，影响架构的先决策。  
> ✅ 已解决的项保留在此作为历史记录。

### ✅ P-01【已解决】中央仓库目录名

**决策**：使用 `~/.agents/`。

**原因**：`~/.agents/` 是 skm（Samsara 的基础设施工具）的原生数据目录，`~/.agents/skills/` 已是 Layer 1 的实际存储位置。Samsara 在其基础上新增 `knowledge/`、`AGENTS.md` 和 `adapters/`，共用同一根目录，无需额外目录。

---

### ✅ P-02【已解决】knowledge/ 是否跨工具 symlink

**决策**：仅在 AGENTS.md 中声明路径，**不** symlink knowledge/ 给各工具。

**理由**：
- AI 通过 AGENTS.md 协议文本中的绝对路径（`~/.agents/knowledge/`）直接读文件，无需工具感知目录
- 各工具是否会自动读 knowledge/ 子目录存疑（无任何工具文档声明此行为）
- Symlink 增加 `samsara init` 复杂度，且收益不明确
- `samsara mcp serve` / bash 调用均可直接访问绝对路径

**实现**：`samsara init` 只建立 A/B 类工具对 AGENTS.md 的映射，不对 knowledge/ 做任何 symlink。

---

### ✅ P-03【已解决】Gemini CLI 和 Windsurf 的 @import 支持

**问题**：这两个工具是否支持在全局指令文件中使用 `@路径` 语法导入外部文件？

**决策**（2026-04-24 实测确认）：

| 工具 | @import 支持 | 方案 |
|------|-------------|------|
| **Gemini CLI** | ✅ 完全支持 | 在 `~/.gemini/GEMINI.md` 中直接用 `@/home/user/.agents/AGENTS.md` 绝对路径引入，无需脚本 |
| **Windsurf** | ❌ 不支持 | 需内容同步脚本，受 6KB 全局上限约束，维护裁剪版（见 §8.2） |

**Gemini CLI 证据**：官方 Memory Import Processor 文档及 [memoryImportProcessor.ts](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/utils/memoryImportProcessor.ts)。
支持语法：`@./relative.md`、`@/absolute/path/to/file.md`；最大递归深度 5 层；仅限 `.md` 文件；代码块内 `@` 被忽略。

**Windsurf 证据**：官方文档明确无等效 @import 机制，全局规则 6KB 上限。
替代方案：使用 `.windsurf/rules/` 多文件目录（Wave 8+）+ 手动或脚本同步关键规则。

**对 samsara 实现的影响**：
- `samsara init` 对 Gemini CLI 的适配：在 `~/.gemini/GEMINI.md` 写入一行 `@/home/<user>/.agents/AGENTS.md` 即完成对接，无需额外命令
- `samsara init` 对 Windsurf 的适配：预留 `scripts/sync-windsurf-rules.sh` 脚本（见 §8.2），由用户按需手动运行或配置 cron

---

### ✅ P-04【已解决】Domain 注册机制

**决策**：文件系统即注册表——`lessons/<domain>/` 目录存在即为合法 domain。

**理由**：
- 原"硬编码枚举"方案限制了语言/平台扩展（遗漏 Android/iOS/Flutter/Swift/Kotlin/C++/C/Makefile/Windows/Linux 等）
- 用户工作场景多样，无法预定义完整列表
- 文件系统目录是天然注册表，无需维护额外配置文件

**实现**：
- `samsara init` 预建 37 个种子 domain 目录（见 §14.2 init 说明）
- `samsara write` 遇到新 domain 时交互式确认创建
- `samsara domain list` 列出所有已有 domain

---

### ✅ P-05【已解决】knowledge/ 纳入 git 版本控制

**决策**：是，knowledge/ 为独立 git repo，由 `samsara init` 初始化，每次 write/promote/archive 自动提交（来源：daerwen §3.7.3，v0.6 已定）。

---

### ✅ P-06【已解决】lessons/archive/ 清理触发机制

**决策**：由 `samsara lint` 命令替代手动清理（见 §18.2）。

---

> 以下 P-07 ~ P-09 为 **Layer C（代码图谱）专项待决策**，暂不纳入主系统。  
> 调研结论详见工作区 `samsara_layer_c_research_2026.md`（Layer C 完整调研报告）。

### P-07【Layer C 专项】代码扫描引擎选型

**背景**：调研了 8 个工具，最终备选两个：

| 工具 | 语言 | 特点 | 适用场景 |
|------|------|------|---------|
| **StakGraph** | Rust | 框架感知、原生支持 Kotlin+Swift、变更追踪 | Android/iOS 项目（首选） |
| **Codebase-Memory** | C | 66 语言、零依赖二进制、<1ms 查询 | 超大型/多语言混合项目 |

**待决策**：选哪个作为 `samsara scan` 的底层引擎，或两者均支持（通过适配层）。

---

### P-08【Layer C 专项】`samsara scan` 生成的 AGENTS.md 的 `valid_until` 默认值

**问题**：代码扫描生成的 AGENTS.md 随代码变化会过期，需要设置默认有效期。

| 方案 | 值 | 说明 |
|------|---|------|
| 与 lesson 对齐 | 90 天 | 统一一套 lint 规则 |
| 更短 | 30 天 | 代码变化快的项目 |
| 基于 git 活跃度 | 动态 | 活跃仓库更短，归档仓库更长 |

**待决策**。

---

### P-09【Layer C 专项】Swift 的 tree-sitter tags.scm

**背景**：Swift 有 tree-sitter 解析器，但没有 `tags.scm`（符号提取查询文件）。这是 iOS 项目支持的主要技术缺口。

- 方案 A：自行编写 Swift tags.scm（参考 Aider 的 Kotlin 版本）
- 方案 B：等待社区实现（Aider 社区有讨论但尚未合并）
- 方案 C：暂时只支持 Android，iOS 延后

**待决策**（影响 Layer C Phase 1 的范围）。

---

## 18. samsara CLI 设计

> **决策**：实现独立 Rust CLI 工具 `samsara`，风格与 skm 对齐。  
> **职责边界**：samsara 管 Layer 2（knowledge/），skm 管 Layer 1（skills/）。

### 18.1 命令设计

```
samsara <subcommand> [args] [flags]

子命令：
  init                        初始化知识库目录结构和工具映射
  write  <domain> <keyword>   写入/更新 lesson（自动查重、frontmatter、git commit）
                              [--update] [--summary "..."] [--yes]
  promote <domain> <keyword>  晋升 lesson → rules/
                              [--aaak] [--layer0] [--yes] [--dry-run]
  reflect                     静态分析日志，输出待晋升候选 + AAAK 候选 + skill 健康报告
  skill-note <name>           记录 skill 使用 / 失败事件到 log.md
  lint     [--fix] [--dry-run] 检查知识库健康状况（13 项，ERROR/WARN/INFO 分级）
  search   <query>            全文搜索知识库，按相关性排序
  status                      输出知识库概览（domain/lesson/rules 统计）
  archive  <domain> <keyword> 归档指定 lesson（移至 archive/ 目录）
                              [--yes] [--stale]
  prime    [--limit N]        从知识库提炼 Top N 规则，输出到 stdout（辅助决定哪些规则放 AGENTS.md）
                              [--sort <recent|occurrences|domain>] [--domain <d>]
  demote   <pattern>          将 AGENTS.md 中的某条规则行降级（从 AGENTS.md 删除，保留在 rules/）
                              [--yes] [--dry-run]
  domain   <subcommand>       domain 管理（list / add）
  log      [--tail N]         查看操作日志（子命令 rotate）
                              [--action <write|promote|archive|lint|...>]
  remote   <subcommand>       管理 knowledge/ git repo 的远端（add / set / show）
  push     [--force]          推送 knowledge/ 到远端
  pull     [--rebase]         从远端拉取 knowledge/

全局 flags：
  --home <path>   覆盖默认知识库路径（默认 ~/.agents/knowledge/）
  --dry-run       仅打印操作，不写入文件
```

### 18.2 各命令行为规范

#### `samsara init`

```
1. 创建知识库目录结构（已存在则跳过）：
   ~/.agents/knowledge/{lessons/,rules/,archive/}
   种子 domain 目录（见 §11 Domain 机制，§11 为权威列表）：
   ~/.agents/knowledge/lessons/{rust/,python/,typescript/,javascript/,go/,java/,kotlin/,swift/,cpp/,c/,dart/,flutter/,android/,ios/,git/,ci/,docker/,k8s/,infra/,makefile/,cmake/,cargo/,windows/,linux/,macos/,api/,database/,auth/,testing/,perf/,security/,ml/,samsara/,skm/,opencode/,vscode/,terminal/}
   ~/.agents/skills/self-evolution/
   ~/.agents/adapters/{claude-code/,gemini/,windsurf/}
2. 创建/更新文件（幂等，已存在时按 upsert 策略处理）：
   ~/.agents/AGENTS.md         → 若不存在：写入协议模板（含 ## AAAK 占位 section）
                                 若已存在：检查并追加缺失的 ## AAAK section（已有则跳过，不覆盖内容）
   ~/.agents/knowledge/INDEX.md → 不存在则创建；已存在则跳过（首次 write 触发重建）
   ~/.agents/knowledge/log.md   → 不存在则创建；已存在则跳过
3. git 初始化：
   git init ~/.agents/knowledge/（若已是 git repo 则跳过）
   写入 .gitignore（archive/ 归档不纳入版本追踪；若文件已存在则 upsert 追加缺失行）
   写入 .gitattributes（upsert 方式：检查并追加缺失属性行，不覆盖现有内容）：
       knowledge/log.md   merge=union   ← 多设备 merge 时 log.md 条目自动合并（保留双方条目）
       knowledge/INDEX.md merge=ours    ← 衍生数据，pull 后由 index::rebuild() 强制重建，忽略 merge 结果
4. 工具映射（只在目标不存在时执行，已存在则打印 ⚠️ 提示手动确认）：
   A 类（symlink，目标工具配置目录存在时）：
     ln -sf ~/.agents/AGENTS.md ~/.config/opencode/AGENTS.md
     ln -sf ~/.agents/AGENTS.md ~/.codex/AGENTS.md
   B 类（@import 注入，~/.claude 存在时）：
     若 ~/.claude/CLAUDE.md 不存在 → 写入 "@~/.agents/AGENTS.md"
     若已存在 → 打印 ⚠️ 提示用户手动添加 @~/.agents/AGENTS.md
   C 类（Gemini/Windsurf）→ 打印 ⏭️ 待 P-03 确认后实现
5. 若 skm 已安装 → 安装/更新 self-evolution skill（skm install ... --link-to all）
6. 输出初始化报告（每步 ✅/⚠️/⏭️ 状态）
```

#### `samsara write <domain> <keyword>`

```
1. Domain 验证（filesystem-based）：
   检查 lessons/<domain>/ 目录是否存在
   ├─ 存在 → 继续
   └─ 不存在：
       ├─ 有 --yes flag → 静默创建目录
       └─ 无 --yes flag → 交互提示："'<domain>' 是新 domain，是否创建？[y/N]"
                          N → 打印已有 domain 列表（samsara domain list），退出
                          Y → mkdir lessons/<domain>/
2. 查重（upsert 语义）：
    ├─ 文件已存在（update 路径）：
    │   a. 追加今日时间戳到 occurrences 数组（允许同一天重复追加，不去重）
    │   b. 若有 --verify flag：verified += 1（在追加 occurrence 基础上额外标记验证生效）
    │   c. 其余字段（summary/root_cause/tags/valid_until/conflicts_with/type）
    │      全部保留旧值，不覆盖
    │   d. 若有 --update flag：按所提供的字段选项覆盖对应字段（见 flags）
    │   e. 检查 len(occurrences) 是否触发晋升提示（≥ 3 且 promoted=false）
   └─ 文件不存在（create 路径）：
       a. 创建文件，frontmatter 初始化（occurrences: ["today"]）
       b. 打开编辑器让用户/AI 填写根因 + 规则
          （--summary / --root-cause 等 flag 可跳过编辑器）
3. 更新 INDEX.md（index::rebuild()）
4. 写入 log.md：WRITE <domain>/<keyword>.md (occurrences: N)
               或 UPDATE <domain>/<keyword>.md (occurrences: N)
5. git commit -m "samsara: write <domain>/<keyword>"
```

**flags**：
- `--summary "..."`        新建时设置 summary，跳过编辑器
- `--root-cause "..."`     新建时设置 root_cause，跳过编辑器
- `--tags tag1,tag2`       新建时设置 tags
- `--type <type>`          设置记忆类型（error|skill|pattern|insight）；更新时可覆盖
- `--valid-until YYYY-MM-DD`  新建时设置过期时间
- `--conflicts-with domain/kw` 新建时设置冲突引用
- `--verify`               标记该规则已被验证生效（verified += 1，同时追加 occurrence）
                           适用：规则被成功应用后的显式确认；与 --update 无关，可单独使用
- `--update`               更新已有文件的内容字段（以下字段按需提供）：
  - `--summary "..."`      覆盖 summary
  - `--root-cause "..."`   覆盖 root_cause
  - `--tags tag1,tag2`     覆盖 tags
  - `--valid-until ...`    覆盖过期时间
  - `--conflicts-with ...` 覆盖冲突引用
- `--yes`                  跳过新 domain 确认提示

#### `samsara promote <domain> <keyword>`

```
1. 读取 lessons/<domain>/<keyword>.md
2. 验证 len(occurrences) >= 3（否则拒绝并提示当前计数）
3. 将核心规则追加到 rules/<domain>.md
4. 更新 lesson frontmatter：promoted: true
5. 若 --layer0 flag（晋升到 AGENTS.md 实质规则区）：
   a. dry-run 预览：打印将写入 AGENTS.md 的行内容，询问"确认写入？[y/N]"
      （--yes flag 跳过确认）
   b. 备份：cp ~/.agents/AGENTS.md ~/.agents/.backup/AGENTS.md.bak
      （覆盖式，只保留最近 1 份）
   c. 行数检查：统计 AGENTS.md 实质规则行数
      （排除 ## AAAK section、空行、纯注释行）
      当前行数 + 新增行数 ≤ 100 → 继续
      超出 → 拒绝并打印：
        "AGENTS.md 实质规则已有 N 行，新增后将超过 100 行上限。
         请先运行 `samsara demote <domain> <keyword>` 降级低优先级规则后重试。"
   d. 写入 AGENTS.md（追加到实质规则末尾、## AAAK section 之前）
   e. 写入 log.md：LAYER0 <domain>/<keyword>.md → AGENTS.md
6. 若 --aaak flag：
   a. 提示用户输入 AAAK 条目（entity/relation/value），或从规则中自动提炼
   b. 写入 ~/.agents/AGENTS.md 的 ## AAAK section（若 section 不存在则在文件末尾追加）
   c. 检查 ## AAAK section 总字符数（≤480），超出时按 date 升序删除最旧条目
7. 写入 log.md：PROMOTE <domain>/<keyword>.md → rules/<domain>.md
8. git commit -m "samsara: promote <domain>/<keyword> [--layer0]"
```

**flags**：
- `--layer0`   晋升到 AGENTS.md 实质规则区（含安全检查，见步骤 5）
- `--aaak`     同时写入 AAAK section（可与 --layer0 同时使用）
- `--yes`      跳过 --layer0 的交互确认
- `--dry-run`  仅预览，不写入任何文件

#### `samsara reflect`

```
静态分析（无 LLM 调用，纯文件扫描）：

1. 扫描 log.md，按 domain 统计 WRITE/UPDATE 频率
2. 识别待晋升候选：occurrences ≥ 3 且 promoted=false
3. 识别高频 domain：30 天内 UPDATE > 5 次 → 建议安装对应 skill
4. 分析 SKILL_FAIL 日志 → 列出失败次数 > 1 的 skill（建议修复）
5. 识别 AAAK 候选：高频出现（> 3 次）且尚未在 AGENTS.md ## AAAK section 中的知识点
6. 输出报告：

Samsara Reflection Report (2026-04-22)
────────────────────────────────────
📈 高频域（建议安装 skill）：
   rust: 8 次 UPDATE（30天内）→ skm search rust

⚡ 待晋升 lesson（按类型）：
   [error]   rust/cargo-fmt-vs-clippy.md  (occurrences: 3, last: 2026-04-22, verified: 1)
   [skill]   rust/async-trait-object.md   (occurrences: 3, last: 2026-04-20)
   [未分类]  git/rebase-stash.md          (occurrences: 3, last: 2026-04-18)

🔧 Skill 健康：
   rust-skills: 2 次 SKILL_FAIL → 建议检查 SKILL.md

💡 AAAK 候选（高频知识点，尚未记录）：
   [cargo-fmt|must-run-before-commit|cargo fmt → clippy → test|今日]
```

#### `samsara skill-note <name>`

```
1. 记录 skill 使用到 log.md：
   ├─ 无 --fail：SKILL_USE  <name> (<备注>)
   └─ 有 --fail：SKILL_FAIL <name> (<失败原因>)
2. 不修改其他文件，不触发 index::rebuild
3. git commit -m "samsara: skill-note <name>"（仅记录，轻量提交）
```

**flags**：
- `--fail` — 标记为失败事件
- `--note "..."` — 附加备注（成功时可省略）

#### `samsara lint`

```
检查项（共 13 项，标注严重程度与是否可自动修复）：

  ① lesson 文件 > 30 行                                    ERROR  ❌ 需人工拆分
  ② rules/ 文件 > 100 行                                   WARN   ❌ 需人工整理
  ③ frontmatter 缺必填字段                                  ERROR  ❌ 需人工补全
     （必填：date / domain / tags / occurrences / promoted）
  ④ occurrences 非数组或含非 ISO-8601 日期                  ERROR  ❌ 需人工修正
  ⑤ valid_until 已过期的 lesson                            WARN   ✅ --fix 移入 archive/
  ⑥ lesson 90 天无新 write（promoted=false 的孤立记录）      INFO   ✅ --fix 移入 archive/
     *注：occurrences 追加 = 引用；AI 读取文件不计入，不重置计时器*
  ⑦ promoted=true 但 rules/ 中无对应条目                   WARN   ❌ 需人工确认
  ⑧ INDEX.md 中的 domain 与实际目录不一致                   WARN   ✅ --fix 重建 INDEX.md
     （INDEX 未记录 / 实际目录不存在）
  ⑨ AGENTS.md 实质规则行数（不含 ## AAAK section）> 100    WARN   ❌ 需人工（建议 samsara demote）
  ⑩ rules/ 文件中引用的 lesson 路径不存在                   WARN   ❌ 需人工
     （lesson 被归档后 rules 引用变为死链）
  ⑪ log.md 行数 > 1000                                     INFO   ✅ --fix 执行 samsara log rotate
  ⑫ conflicts_with 列出的 keyword 在 lessons/ 或 rules/ 中
     均不存在                                               WARN   ❌ 仅报告
     （被归档后引用变为死链；不自动删除——archive 文件仍有参考价值）
  ⑬ 同 domain 内存在 tags 高度重叠的 lesson 对
     （Jaccard 相似度 ≥ 0.7）                              INFO   ❌ 建议人工合并
     （可能是同一根因的不同描述，review 后用 samsara write --update 合并）

输出：按严重程度分组（ERROR / WARN / INFO）
```

**flags**：
- `--fix` — 仅执行可安全自动修复的检查项（⑤⑥⑧⑪），其余项只报告不修改文件
- `--dry-run` — 与 `--fix` 联用，预览将要执行的修改，不实际写入

**`--fix` 执行顺序**：
```
1. 收集 ⑤⑥ 过期/孤立 lesson 列表 → 逐条询问（或 --yes 批量确认）→ 移入 archive/
2. ⑧ 重建 INDEX.md
3. 步骤 1+2 合并一次 git commit: "samsara: lint --fix (archive N lessons, rebuild INDEX)"
4. ⑪ 调用 log rotate 逻辑（含独立 git commit: "samsara: log rotate"）
5. 输出最终报告，未修复项保留在报告中标记 [skipped]
```

#### `samsara search <query>`

```
全文搜索知识库，按相关性排序。

Algorithm:
  1. 收集候选文件：遍历 lessons/**/*.md 和 rules/*.md
  2. 对每个文件计算相关性分值：
     - 文件名（stem）== query → +100
     - frontmatter tags 含 query → +50
     - domain 目录名 == query → +40
     - 文件正文含 query → 每处 +10（上限 +50）
     总分 == 0 的文件过滤掉
  3. 按分值降序排列
  4. 输出（每个命中文件）：
     [路径]  标签... 出现次数（lesson）
       匹配行预览（最多 2 行，高亮匹配词）

flags:
  --domain <name>    仅搜索 lessons/<domain>/
  --type <type>      仅搜索指定类型（error|skill|pattern|insight）
  --rules-only       仅搜索 rules/
  --lessons-only     仅搜索 lessons/
  --limit N          最多显示 N 个结果（默认 10）

示例：
  $ samsara search cargo-fmt
  [lessons/rust/cargo-fmt-vs-clippy.md]  rust, cargo  occurrences: 3
    Line 8: `cargo clippy` 通过 ≠ `cargo fmt --check` 通过
  [rules/rust.md]
    Line 12: cargo fmt → cargo clippy -- -D warnings → cargo test
```

#### `samsara domain list`

```
列出所有已有 domain（枚举 lessons/ 子目录），格式化输出。

示例输出：
  Domain          Lessons  Rules
  ─────────────────────────────
  rust            3        ✅ rust.md
  git             1        —
  android         0        —  （空目录，已预注册）
  flutter         2        —
```

#### `samsara domain add <name>`

```
预注册新 domain 目录（mkdir lessons/<name>/），不写 lesson。

Algorithm:
  1. 验证 name 不含非法字符（/ 空格等）
  2. 检查 lessons/<name>/ 是否已存在 → 存在则打印已有并退出
  3. mkdir lessons/<name>/
  4. 输出：✅ domain 'flutter' 已注册（lessons/flutter/）
```

#### `samsara log rotate [--keep 90d]`

```
轮转 log.md，保留最近 N 天的条目，旧日志存入归档文件。

Algorithm:
  1. 解析 log.md 全部条目
  2. cutoff_date = today - keep_days（默认 90）
  3. 分割：
     recent = entries where date >= cutoff_date
     old    = entries where date < cutoff_date
  4. 若 old 为空 → 打印 "无需轮转" 并退出
  5. 按年份分组 old，追加到 log.archive-YYYY.md（append-only）
  6. 将 recent 写回 log.md（覆盖）
  7. auto_commit("samsara: log rotate")
  8. 输出：归档 N 条到 log.archive-YYYY.md，保留 M 条

触发时机：
  - 用户手动执行
  - samsara lint 发现 log.md > 1000 行时给出 WARN 提示（见 lint ⑧）
```

#### `samsara archive <domain> <keyword>`

```
将指定 lesson 移至 archive/ 目录，从 lessons/ 下线。

Algorithm:
  1. 验证 lessons/<domain>/<keyword>.md 存在
  2. 若 frontmatter promoted=true → 拒绝归档（已晋升的 lesson 需先确认 rules/ 中已有对应规则）
  3. mkdir -p archive/<domain>/
  4. mv lessons/<domain>/<keyword>.md archive/<domain>/<keyword>.md
  5. 在归档文件头部追加注释：# archived: <today>（不修改 frontmatter）
  6. 更新 INDEX.md（index::rebuild()）
  7. 写入 log.md：ARCHIVE <domain>/<keyword>.md
  8. git commit -m "samsara: archive <domain>/<keyword>"

批量归档（lint ⑤⑥ 建议的全部过期 lesson）：
  samsara archive --stale [--yes]
  → 对 valid_until 已过期 或 90 天无 occurrence（且 promoted=false）的 lesson 依次执行 archive
  → --yes 跳过逐条确认（建议先 samsara lint 预览再执行）

flags:
  --stale   批量归档所有过期/孤立 lesson
  --yes     跳过逐条确认
```

#### `samsara prime [--limit N] [--sort <recent|occurrences|domain>] [--domain <d>]`

```
从知识库提炼 Top N 规则，输出到 stdout（不写文件）。
用途：辅助决定哪些规则值得放入 AGENTS.md，配合 samsara promote 手动操作。

Algorithm:
  1. 收集所有 rules/*.md 的规则条目（按 "## 规则标题" 解析）
  2. 收集 occurrences >= 3 且 promoted=true 的 lessons 的核心规则行
  3. 对每条规则计算"推荐分"：
     - occurrences 总数 × 10
     - 最近 occurrence 距今天数 d → score += max(0, 30 - d) × 5（越近越高）
     - type == 'error' → +20（错误教训优先晋升到 AGENTS.md）
     - verified 次数 × 15（被显式验证有效的规则加分）
     - conflicts_with 非空 → -10（有冲突声明的规则谨慎晋升）
     - 已在 AGENTS.md 中出现 → 分数减半（减少重复推荐）
  4. 按推荐分降序，取前 N 条（默认 10）
  5. 格式化输出到 stdout：
     ────────────────────────────────────────────────
     samsara prime: Top 10 推荐规则 (sorted: recent)
     ────────────────────────────────────────────────
      #1  [rust/cargo-fmt-vs-clippy]  [error] cargo fmt → clippy → test（提交前顺序）
          来源: rules/rust.md:8 | occurrences: 5 | verified: 1 | last: 2026-04-22 | score: 120
          → samsara promote --layer0 rust cargo-fmt-vs-clippy
       #2  [git/rebase-stash]  [skill] 交互式 rebase 前必须 stash
          来源: rules/git.md:15 | occurrences: 3 | verified: 0 | last: 2026-04-20 | score: 42
          → samsara promote --layer0 git rebase-stash
      ...
     ────────────────────────────────────────────────
     提示: 直接复制上方 samsara promote --layer0 命令执行晋升

flags:
  --limit N         最多显示 N 条（默认 10）
  --sort recent     按最近 occurrence 排序（默认）
  --sort occurrences 按总次数排序
  --sort domain     按 domain 分组输出
  --domain <d>      仅输出指定 domain 的规则
```

**触发时机**（以下任一条件满足时运行）：

| 触发来源 | 条件 | 用途 |
|----------|------|------|
| `samsara lint` ⑨ | AGENTS.md 超 100 行实质规则 | 辅助决策哪条规则值得 demote |
| `samsara reflect` | 报告 3+ 条 promoted=false 且 occurrences≥3 的候选 | 辅助决策哪条候选值得晋升到 AGENTS.md |
| 手动 | 任意时刻 | 无副作用，只输出到 stdout，随时可调用 |

#### `samsara demote <pattern>`

```
将 AGENTS.md 中的某条用户规则行降级回 rules/ 层（从 AGENTS.md 删除，不删 rules/ 内容）。
适用场景：AGENTS.md 超过 100 行实质规则时，将低优先级规则降回 Layer 2。

Algorithm:
  1. 解析 ~/.agents/AGENTS.md，标记受保护 sections（不可操作）：
     - ## 自我进化协议（Samsara）及其全部内容
     - ## AAAK 及其全部内容（含 <!-- --> 注释行）
  2. 在剩余可操作行中，grep 匹配 pattern（不区分大小写，支持部分词匹配）
     若无匹配 → 打印「未找到可降级规则，请检查 pattern」，不修改任何文件，退出
  3. 展示匹配行（含行号和上下文 ±1 行），要求确认（--yes 跳过）
  4. 从 AGENTS.md 删除匹配行
  5. 提示："规则应已存在于 rules/<domain>.md，请确认后运行 samsara lint 验证"
  6. 写入 log.md：DEMOTE rule "<pattern>" from AGENTS.md
  7. git commit -m "samsara: demote rule from AGENTS.md"

flags:
  --yes       跳过逐条确认
  --dry-run   预览匹配行，不执行删除
```

#### `samsara remote add <url>` / `samsara remote set <url>` / `samsara remote show`

```
管理 knowledge/ git repo 的远端地址（用于多设备同步）。

samsara remote add <url>:
  1. git -C ~/.agents/knowledge remote add origin <url>
  2. 写入 ~/.agents/samsara.toml [sync] remote_url = "<url>"
  3. 输出：✅ 已设置远端 origin: <url>
  4. 提示：可用 `samsara push` 推送 / `samsara pull` 拉取

samsara remote set <url>:
  1. git -C ~/.agents/knowledge remote set-url origin <url>
  2. 更新 ~/.agents/samsara.toml [sync] remote_url = "<url>"
  3. 输出：✅ 已更新远端 origin: <url>

samsara remote show:
  输出 git remote -v + samsara.toml [sync] 配置
```

#### `samsara push [--force]`

```
推送 knowledge/ 到远端 git repo。

Algorithm:
  1. 检查是否有未提交变更（git -C ~/.agents/knowledge status -s）
     → 若有未提交变更 → 报错：建议先完成 samsara write/promote 等操作
  2. git -C ~/.agents/knowledge push origin main [--force]
  3. 输出推送统计（新增/更新文件数）

flags:
  --force   强制推送（需二次确认，可能覆盖远端）
```

#### `samsara pull [--rebase]`

```
从远端拉取 knowledge/ 更新（用于多设备同步）。

Algorithm:
  1. 检查是否有未提交本地变更
     → 若有 → 报错：建议先用 samsara write 等完成写入（保证 git 状态干净）
  2. git -C ~/.agents/knowledge fetch origin
  3. git -C ~/.agents/knowledge pull [--rebase] origin main
     冲突解决策略（按文件类型）：
     ├─ log.md           → union merge（.gitattributes 保证，自动合并）
     ├─ INDEX.md         → 忽略冲突（步骤 4 强制重建，以本地为准）
     ├─ lessons/**       → last-push-wins（自动 checkout --theirs，接受远端版本）
     ├─ rules/**         → last-push-wins（同上）
     └─ 其他文件冲突     → 打印 ❌ 并中止，要求用户手动解决后重新 pull
  4. index::rebuild()（INDEX.md 强制重建，同步新增 lesson）
  5. 输出同步统计（新增/更新 lesson 数）

flags:
  --rebase   使用 rebase 而非 merge（推荐：保持线性历史，避免 merge commit）
```

#### `samsara status`

```
输出示例：

Samsara Knowledge Base
─────────────────────
Domains:  4 (rust, git, ci, api)
Lessons:  7 total, 2 promoted
Rules:    2 files (rust.md: 23 lines, git.md: 8 lines)
Skills:   5 installed (via skm)
Log:      12 entries, last: 2026-04-22

Pending Promotions (occurrences ≥ 3, not promoted):
  rust/cargo-fmt-vs-clippy.md  (3 occurrences, last: 2026-05-10)
```

### 18.3 数据目录结构（CLI 负责维护）

CLI 保证以下文件的格式一致性，AI 仍可直接读写，但建议通过 CLI 写入：

```
~/.agents/knowledge/
├── INDEX.md          ← samsara status/write 自动维护
├── log.md            ← 所有操作自动追加
├── lessons/<domain>/ ← samsara write 写入
├── rules/            ← samsara promote 写入
└── archive/          ← samsara archive / samsara lint --fix 移入
```

### 18.4 技术选型

| 项目 | 选择 | 原因 |
|------|------|------|
| 语言 | Rust | 与 skm 风格对齐，单二进制无依赖 |
| YAML 解析 | `serde_yaml` | frontmatter 解析 |
| 日期处理 | `chrono` | occurrences 时间戳操作 |
| CLI 框架 | `clap` | 与 skm 同款 |
| 安装方式 | `cargo install` 或 `skm install samsara-cli` | 待定 |

### 18.5 与 skm 的关系

```
用户视角：
  skm install rust-skills     → 管 Layer 1（skill 包）
  samsara write rust clippy   → 管 Layer 2（个人记忆）

AI 视角：
  load_skills=["rust-skills"]                        → 读 Layer 1
  bash("samsara write rust clippy --summary '...'")  → 写 Layer 2
  bash("samsara promote rust clippy")                → 晋升 Layer 2
```

### 18.6 实现优先级

| Phase | 命令 | 说明 |
|-------|------|------|
| v0.1 | `init`, `write`, `search`, `status`, `log` | init 完成目录+git+symlink+工具映射；write 支持动态 domain；search 提供基础检索能力 |
| v0.2 | `lint`（13 项）, `promote`（含 --aaak）, `reflect`, `skill-note`, `domain` | lint 覆盖 13 项检查（含 conflicts_with 引用检查、frontmatter 校验、promoted 状态一致性、蒸馏候选⑬）；promote --aaak 写入 AGENTS.md section；domain list/add 可用 |
| v0.3 | `archive`, `prime`, `demote`, `remote`, `push`, `pull`, `log rotate`, `--dry-run`, `mcp serve` | 完整防膨胀自动化；紧凑上下文生成（stdout）；降级机制；多设备同步；MCP Server（AI 原生调用接口） |
| v1.0 | `query`（结构化查询语言）, Web UI, 知识统计分析 | 待专项研究后排期 |

---

## 19. MCP Server 设计（v0.3）

> 目标：让 AI 通过 MCP 协议原生调用 Samsara，替代部分 `bash("samsara ...")` 调用，提升调用效率和工具感知。

### 19.1 启动方式

```bash
samsara mcp serve [--port 3000]   # stdio 模式（默认）或 HTTP SSE 模式
```

在工具（opencode / Claude Code 等）的 MCP 配置中注册：

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

### 19.2 暴露的 MCP 工具

| Tool Name | 等价 CLI | 说明 |
|-----------|----------|------|
| `write_lesson` | `samsara write` | 写入/更新 lesson，自动查重 + git commit |
| `search_knowledge` | `samsara search` | 按 domain / tag / keyword 检索 |
| `get_status` | `samsara status` | 返回知识库统计（domains/lessons/rules/log） |
| `promote_lesson` | `samsara promote` | 晋升 lesson 到 rules/ 或 AGENTS.md |
| `read_index` | `cat INDEX.md` | 获取 INDEX.md 完整内容（任务开始时用） |
| `prime_context` | `samsara prime` | 生成紧凑上下文摘要（输出到工具返回值） |

### 19.3 工具参数规范（关键工具）

```
write_lesson(domain: str, keyword: str, summary: str, root_cause?: str,
             tags?: [str], type?: str, valid_until?: str, conflicts_with?: str)
  → {ok: bool, path: str}

search_knowledge(query: str, domain?: str, tag?: str, limit?: int = 10)
  → [{path, summary, occurrences}]

promote_lesson(domain: str, keyword: str, target: "rules" | "layer0",
               aaak_entry?: str)
  → {ok: bool, promoted_to: str}

prime_context(domains?: [str], max_tokens?: int = 800)
  → {text: str, sources: [str]}
```

### 19.4 与 bash 调用的关系

MCP 工具不替代所有 CLI 调用，而是为高频操作提供更好的调用体验：

| 场景 | 推荐方式 |
|------|----------|
| 写入 lesson（高频） | MCP `write_lesson` |
| 搜索知识（高频） | MCP `search_knowledge` |
| 一次性维护命令（lint / archive / sync） | `bash("samsara ...")` |
| 初始化（一次性） | `bash("samsara init")` |

---

## 17. 版本历史

| 版本 | 日期 | 变更内容 |
|------|------|---------|
| **v0.8** | 2026-04-23 | 竞品调研整合（§3.8 新增：Mem0/Letta/Zep/memex-kb 对标，OWASP ASI06 安全考量）；MCP Server 设计（§19 新增：6 个 MCP 工具，参数规范，与 bash 调用关系）；命令扩展（archive/prime/demote/remote/push/pull 算法补全）；seed domain 统一为 37 个；`conflicts_with` 字段（§10.1）；lint ⑫ conflicts_with 引用检查；§11 协议文本精简至 22 行（domain 机制移至 self-evolution SKILL）；P-02 关闭（不 symlink，仅路径引用）；§18.6 实现路线更新（v0.2 lint→12项，v0.3 扩展 7 个命令+MCP，新增 v1.0 行）；**DNA Memory 竞品调研整合**：Lesson `type` 字段（error\|skill\|pattern\|insight，§10.1 + §18.2 write/reflect/prime/search）；`verified` 字段（整数，write --verify 递增，prime 评分 ×15）；lint ⑬ 蒸馏候选检测（Jaccard ≥ 0.7，INFO 级）；reflect 按 type 分组输出；prime 评分公式加入 type/verified/conflicts_with 三维度；§18.6 v0.2 lint→13项 |
| **v0.7** | 2026-04-23 | AAAK 合并进 AGENTS.md（废弃独立 aaak.md，根本原因：无工具自动加载保证）；Domain 改为文件系统注册表（废弃硬编码枚举，种子 domain 扩充到 23 个含 Android/iOS/Flutter 等）；补充 search/domain/log-rotate 命令；修复 §9.1 Step 1 判重逻辑（INDEX.md → 文件检查）；§12 lint 操作改用 samsara CLI 命令；§13 新增 log.md 1000 行约束；§14.2 改为 samsara init 说明；P-04/P-06 关闭；§18.6 发布路线更新 |
| **v0.6** | 2026-04-22 | 新增 §3.7 daerwen 调研（AAAK/Reflection/git 版本控制/skill 使用追踪）；核心原则新增第 11-12 条；§5 Layer 0 加 AAAK 子层；§6.1 目录结构加 aaak.md 和 knowledge/.git；§10.4 AAAK 条目格式规范；§11 协议文本加 SKILL_USE/SKILL_FAIL 记录规则；§18 新增 init/reflect/skill-note 命令行为规范；§18.6 实现路线更新；P-05 方向更新 |
| **v0.5** | 2026-04-22 | 新增 §3.6 调研来源与设计推导链（MemPalace/Karpathy/Hermes/Oracle/取舍表）；核心原则新增第 8-10 条并标注来源；新增 §18 samsara CLI 完整设计（命令规范/行为/技术选型/实现路线）；文档头部更新版本号；目录新增 §3.6 和 §18 入口 |
| **v0.4** | 2026-04-22 | `recurrences:N` → `occurrences:[timestamps]`；Lesson 文件名改为 `[keyword].md`（upsert 语义）；新增 `log.md` 操作日志；§11 新增 grep 查重规则和 domain 枚举列表；Lesson frontmatter 新增 `valid_until` 可选字段；§12 新增 lint 周期检查节；移除重复目录树块 |
| **v0.3** | 2026-04-22 | 新增 §3.5 skm 基础设施详情；P-01 解决（中央目录确定为 `~/.agents/`）；全局路径由 `~/.samsara/` 更新为 `~/.agents/`；Layer 1 标注由 skm 统一管理；§7.2 改为 skm 统一部署方案；install.sh 整合 skm 命令；§15 新增 4 条 skm 相关技术事实 |
| **v0.2** | 2026-04-22 | 重构：多工具兼容架构；整合 AGENTS.md 标准调研；引入适配层；补充 Codex/Claude Code/Gemini/Windsurf 精确路径 |
| **v0.1** | 2026-04-22 | 初稿：三层架构、晋升机制、防膨胀约束（OpenCode 专属版本） |

---

*文档结束。在下次会话中打开此文件继续讨论。*
