# mocika-samsara · 工作区说明

本目录是 **Samsara（轮回）AI 自我进化知识系统** 的工程工作区。

---

## 🔄 当前阶段：阶段 6 发布（进行中）

> **AI agent 行为约束**：阶段 5 全部退出条件已满足（2026-04-24）。  
> ✅ 可以：进行阶段 6 发布工作（skm 接入、skill 对齐、版本标签）  
> ❌ 不可以：跳过阶段 6 退出标准直接宣告发布完成  
> 完整流程定义与阶段门控规则见 → `docs/process.md`

### 阶段 5 退出标准（✅ 已满足，2026-04-24）

- [x] `~/.agents/AGENTS.md` 已就绪（含自我进化协议 + 常用命令表）
- [x] P-03 实测：Gemini CLI ✅ 支持 `@import` 绝对路径；Windsurf ❌ 不支持，需同步脚本
- [x] 完整流程 write → promote → lint → reflect 无报错（真实 `~/.agents/knowledge/` 验证）
- [x] OpenCode MCP 握手验证：`initialize` + `tools/list` + `tools/call` 全程正常
- [x] MCP bug 修复：notification 静默处理 + `protocolVersion` 字段补全（2026-04-24）
- [ ] Claude Code / Gemini CLI 兼容性（工具未安装，不阻塞主流程）

### 阶段总览

- ✅ **阶段 1**：产品设计（`docs/samsara-design.md` v0.8）
- ✅ **阶段 2**：技术规格（`docs/samsara-cli-impl.md` v0.4）
- ✅ **阶段 3**：工程准备（Rust 骨架，2026-04-23 完成）
- ✅ **阶段 4**：迭代开发（v0.1 → v0.2 → v0.3 → v0.4，43/43 测试通过）
- ✅ **阶段 5**：集成验收（真实场景验证、P-03 实测，2026-04-24 完成）
- 🔄 **阶段 6**：发布 ← 当前

---

## 关键文件

| 文件 | 说明 | 状态 |
|------|------|------|
| `docs/samsara-design.md` | 产品设计 v0.8 | 🔒 已冻结 |
| `docs/samsara-cli-impl.md` | 软件实现规格 v0.4 | 🔒 已冻结 |
| `docs/samsara-engineering.md` | 工程开发参考（技术选型、骨架、SKILL.md、自升级） | 🟢 活跃 |
| `docs/research/layer-c/` | Layer C 代码图谱调研（待专项研究） | 🔵 归档 |
| `skills/self-evolution/SKILL.md` | AI agent 操作手册（随 CLI 版本同步） | 🟢 活跃 |

---

## 关联代码仓库

| 项目 | 路径 | 说明 |
|------|------|------|
| **skm CLI** | `/home/shanying/WorkSpace/MocikaSpace/mocika-skills-cli` | 参考实现：Rust 架构、i18n、CI/CD、自升级策略 |

---

## 产品设计关键决策（速查）

| 决策 | 结论 |
|------|------|
| 中央仓库位置 | `~/.agents/`（skm 原生目录） |
| knowledge/ symlink | 否，仅通过 AGENTS.md 路径引用（P-02） |
| 管理程序 | 独立 Rust CLI（`samsara`），风格对齐 skm |
| INDEX.md 维护 | 全量重建（衍生数据，无增量更新） |
| 数据库 / embedding | 否，纯文件 + git |
| AAAK 存储 | AGENTS.md 末尾 `## AAAK` section |
| Domain 注册 | 文件系统即注册表（`lessons/<domain>/` 目录） |
| git 集成 | `std::process::Command`（不引入 git2） |

---

## 待决策项

- **P-03**：✅ 已决策（2026-04-24）：Gemini CLI 支持 `@import` 绝对路径；Windsurf 不支持，需同步脚本
- **P-07/P-08/P-09**：Layer C 专项（暂不纳入主系统，见 `docs/research/layer-c/`）
