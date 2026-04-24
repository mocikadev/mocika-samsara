# Samsara · 研发流程

> **原则**：一次只专注一个阶段。上一阶段退出标准完全满足后，才进入下一阶段。  
> AI agent 须以"当前阶段"约束自身行为范围——**不得在设计未冻结时写实现代码，不得在规格未完整时搭项目骨架。**

---

## 阶段总览

| # | 阶段 | 核心产出物 | 状态 |
|---|------|-----------|------|
| 1 | **产品设计** | `docs/samsara-design.md` | ✅ 完成 |
| 2 | **技术规格** | `docs/samsara-cli-impl.md` | ✅ 完成 |
| 3 | **工程准备** | Rust 项目骨架、CI 配置 | ✅ 完成 |
| 4 | **迭代开发** | v0.1 → v0.2 → v0.3 → v0.4 可运行版本 | ✅ 完成（2026-04-24）|
| 5 | 集成验收 | 真实场景验证、P-03 实测 | 🔄 进行中（主要项目已通过）|
| 6 | 发布 | cargo install / skm install | ⏳ 未开始 |

---

## 阶段 1：产品设计

**目标**：确定系统架构、用户交互、数据模型和命令规格，使后续阶段可以无歧义地执行。

### 进入条件
- 问题背景与目标明确（§1）
- 已有至少一轮架构草案

### 退出标准（全部满足才能进入阶段 2）

- [ ] 三层架构（Layer 0/1/2）边界清晰，无职责重叠
- [ ] 所有 P-xx 待决策项要么**已决策**，要么**明确标注为"可在后续阶段实测后再决"**（不能是"待确认"悬空）
- [ ] §18 命令规格：每条命令有完整的 Algorithm 伪代码，无"待定"字段
- [ ] §11 协议文本：可以直接复制粘贴到 `~/.agents/AGENTS.md` 使用
- [ ] §10 文件格式规范：frontmatter 字段、示例完整，无歧义
- [ ] 与技术规格（阶段 2）无矛盾：设计文档中的算法描述与 impl 文档一致

### 完成情况

- [x] P-02：knowledge/ 不 symlink，仅通过 AGENTS.md 路径引用
- [x] §11 协议文本精简至 <25 行，与 v0.8 设计同步
- [x] §18 所有命令规格 Algorithm 伪代码完整，无"待定"字段

### 行为约束（AI agent）
- ✅ 可以：完善设计文档、补充协议文本、讨论架构方案、更新 §16 待决策项
- ❌ 不可以：创建 Rust 项目文件、写任何 `.rs` 代码、初始化 Cargo.toml

---

## 阶段 2：技术规格

**目标**：将产品设计翻译为可直接指导编码的技术规格，包含完整的数据结构、算法和测试策略。

### 进入条件
- 阶段 1 退出标准全部满足（产品设计冻结）

### 退出标准（✅ 全部满足）

- [x] 所有数据结构（struct/enum）定义完整，字段类型明确
- [x] 所有命令的实现算法完整，与产品设计 §18 一一对应
- [x] 错误类型（`SamsaraError`）覆盖所有可能的失败路径
- [x] 测试策略完整：fixtures 目录已规划，每个 fixture 对应的测试场景明确
- [x] 依赖项（Cargo.toml）确定，见 `docs/samsara-engineering.md` §1.1
- [x] 与产品设计无矛盾（v0.4 最终对齐，6 处不一致全部修复）

### 行为约束（AI agent）
- ✅ 可以：完善 impl 文档、补充数据结构、细化算法伪代码
- ❌ 不可以：创建 Rust 项目文件、写任何 `.rs` 代码

---

## 阶段 3：工程准备

**目标**：建立可编译的项目骨架，使后续开发可以直接在正确结构上进行。

### 进入条件
- 阶段 1 + 阶段 2 退出标准全部满足

### 退出标准

- [x] `cargo build` 通过（空实现，所有命令返回 `unimplemented!()`）
- [x] 目录结构与 impl 文档 §1 一致
- [x] CI 配置：`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test`
- [x] `README.md` 包含安装方式和基本用法
- [x] `samsara --help` 输出所有子命令

### 行为约束（AI agent）
- ✅ 可以：创建项目文件、写骨架代码（`unimplemented!()`）、配置 CI
- ❌ 不可以：在骨架验证前实现具体命令逻辑

---

## 阶段 4：迭代开发

**目标**：按 v0.1 → v0.2 → v0.3 → v0.4 里程碑实现所有命令，每个里程碑独立可验证。

### 里程碑划分

| 版本 | 命令 | 验收标准 |
|------|------|---------|
| **v0.1** | `init`, `write`, `search`, `status`, `log` | init 完成目录+git+symlink；write 支持动态 domain；search 按相关性返回结果 |
| **v0.2** | `lint`, `promote`（含 --layer0）, `reflect`, `skill-note`, `domain` | lint 覆盖 13 项检查含引用失效；promote --layer0 写入 AGENTS.md section |
| **v0.3** | `prime`, `archive`, `demote`, `log --rotate` | 完整功能，`cargo test` 全绿（36/36） |
| **v0.4** | `push`, `pull`, `self-update`, `mcp serve` | 多设备同步 + MCP 服务可运行 |

### 每个里程碑的退出标准

- [ ] 本里程碑所有命令实现完毕
- [ ] `cargo test` 全绿（对应 fixtures 的集成测试通过）
- [ ] `cargo clippy -- -D warnings` 无告警
- [ ] `cargo fmt --check` 通过
- [ ] 手动走通一次真实场景（不仅是单测）

### 行为约束（AI agent）
- ✅ 可以：按里程碑顺序实现命令、写测试、重构
- ❌ 不可以：跳过里程碑（v0.1 未完成时不写 v0.2 代码）
- ❌ 不可以：在测试未通过时标记里程碑完成

---

## 阶段 5：集成验收

**目标**：在真实环境中验证系统端到端可用。

### 退出标准

- [x] `~/.agents/AGENTS.md` 已就绪（`samsara init` 于阶段 4 v0.1 完成，含自我进化协议 + 常用命令表）
- [x] P-03 实测：**Gemini CLI ✅ 支持 `@import` 绝对路径**；**Windsurf ❌ 不支持**，需同步脚本；结论已写入 `samsara-design.md` §16 P-03（2026-04-24）
- [x] 完整流程 write → promote → lint → reflect 无报错（2026-04-24 在真实 `~/.agents/knowledge/` 验证通过）
- [x] OpenCode 兼容性验证：MCP `initialize` + `tools/list` + `tools/call get_status` 全程响应正常；`opencode.json` 已配置 `samsara mcp serve`（2026-04-24）
- [ ] Claude Code / Gemini CLI 兼容性验证（工具未安装，待后续有机器时补充）

---

## 阶段 6：发布

### 退出标准

- [x] `cargo install` 安装后可正常使用（`samsara 0.1.0` 已安装，2026-04-24）
- [x] skm 接入方案确认：开发阶段直接 `cp SKILL.md ~/.agents/skills/self-evolution/`；发布后通过 `skm install mocikadev/mocika-samsara --subpath skills/self-evolution` 安装（需先创建 GitHub Release）
- [x] `self-evolution` skill v0.4.0 与最终 CLI 命令对齐（2026-04-24）：frontmatter version 0.4.0，命令速查全部标 ✅，MCP 配置格式已修正

---

## v1.0+ Roadmap（待探讨）

> 以下功能已识别，尚未进入设计阶段，需要专项讨论后再定方案。

| 功能 | 描述 | 来源 |
|------|------|------|
| **从知识库自动生成 SKILL.md** | 积累足够的同类 lesson 后，自动提炼生成可复用的 SKILL.md 文件；参考 Hermes Agent 的轨迹提炼思路 | `samsara-design.md §3.6.3` |
| **Layer C：代码扫描层** | 集成 CodeMap/agentmd，支持 `samsara generate-agents-md` 从代码库自动生成 AGENTS.md | `docs/research/layer-c/` |

---

## 阶段切换记录

| 时间 | 事件 |
|------|------|
| 2026-04-22 | 阶段 1（产品设计）开始 |
| 2026-04-23 | 设计文档 v0.7、实现文档 v0.3 完成主要结构 |
| 2026-04-23 | 阶段 1（产品设计）完成：`samsara-design.md` v0.8 通过全部退出标准 |
| 2026-04-23 | 阶段 2（技术规格）完成：`samsara-cli-impl.md` v0.4 通过全部退出标准 |
| 2026-04-23 | 阶段 3（工程准备）开始 |
| 2026-04-23 | 阶段 3（工程准备）完成：骨架验证通过，所有 17 子命令可 --help 输出 |
| 2026-04-23 | 阶段 4 v0.1 完成：13/13 集成测试通过，clippy/fmt 全绿（手动验收待确认）|
| 2026-04-23 | 阶段 4 v0.2 完成：26/26 集成测试通过，clippy/fmt 全绿（手动验收待确认）|
| 2026-04-24 | 阶段 4 v0.2 手动验收通过：init/domain/write/skill-note/promote/lint/reflect 全流程无报错 |
| 2026-04-24 | 阶段 4 v0.3 开始 |
| 2026-04-24 | 阶段 4 v0.3 完成：36/36 集成测试通过，clippy/fmt 全绿，手动验收通过（archive/prime/demote/log rotate）|
| 2026-04-24 | 阶段 4 v0.4 完成：43/43 集成测试通过，clippy/fmt 全绿，手动验收通过（remote/push/pull/self-update/mcp serve）|
| 2026-04-24 | 阶段 5 集成验收开始 |
| 2026-04-24 | 阶段 5 主要项目通过：write→promote→lint→reflect 流程✅；MCP tools/call✅；OpenCode opencode.json 已配置✅；P-03 决策✅（Gemini @import 支持，Windsurf 不支持需同步脚本）|
| 2026-04-24 | MCP bug 修复：notification id=None 静默处理 + InitializeResult 补 protocolVersion；OpenCode 握手验证通过 |
| 2026-04-24 | 阶段 6 进入：AGENTS.md/process.md 阶段标注更新，SKILL.md v0.4.0 同步到 ~/.agents/skills/self-evolution/ |
| 2026-04-24 | 阶段 6 完成：三项退出标准全部满足（cargo install ✅ / skm 方案确认 ✅ / skill 对齐 ✅）；待创建 GitHub Release v0.1.0 标签 |
