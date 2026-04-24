# Samsara Layer C（代码扫描层）决策指南

**日期**: 2026-04-23  
**状态**: 待决策  
**优先级**: 高（影响 v0.2 及后续版本规划）

---

## 快速决策表

| 问题 | 答案 | 理由 | 下一步 |
|------|------|------|--------|
| **是否需要 Layer C？** | ✅ 是 | 代码图谱生成已成熟，AGENTS.md 自动生成工具已出现 | 进行 P-07 决策 |
| **是否自己实现？** | ❌ 否 | 维护成本高（66+ 语言），MCP 标准化使集成成本低 | 选择集成方案 |
| **采用哪个工具？** | CodeMap（首选）+ agentmd（生成） | 轻量级、MCP 原生、支持多语言 | 原型实现 |
| **何时实现？** | v0.2（2-3 周） | 与 Samsara 的产品路线图对齐 | 启动 Phase 1 |

---

## 三个待决策项（P-07 ~ P-09）

### P-07：是否实现 Layer C（代码扫描层）？

**选项 A**：实现（推荐）
- ✅ 自动生成初始 AGENTS.md
- ✅ 支持多语言代码库
- ✅ 与 Samsara 的"被动积累"模式对齐
- ⚠️ 需要集成 3-4 个外部工具

**选项 B**：不实现
- ✅ 减少开发工作量
- ❌ 用户需要手动维护 AGENTS.md
- ❌ 无法支持大型代码库的自动化

**建议**：选项 A（实现）

**理由**：
1. 现有工具已成熟（生产级，社区活跃）
2. MCP 标准化使集成成本低（2-3 周）
3. 与 Samsara 的核心价值对齐（自我进化）
4. 用户期望（AGENTS.md 已成为行业标准）

---

### P-08：AGENTS.md 自动生成的质量保证？

**选项 A**：生成 + 评分 + 漂移检测（推荐）
- ✅ 三层质量保证
- ✅ 自动检测过时内容
- ✅ 支持 CI 集成
- ⚠️ 需要集成 agentmd 工具

**选项 B**：仅生成，不检测
- ✅ 实现简单
- ❌ 无法保证质量
- ❌ AGENTS.md 容易过时

**选项 C**：完全手动
- ✅ 完全控制
- ❌ 不符合 Samsara 的自动化目标

**建议**：选项 A（生成 + 评分 + 漂移检测）

**质量目标**：
- 总分 ≥ 80/100（agentmd score）
- 新鲜度 ≥ 15/20
- 月度漂移检测（自动 PR 提醒）

---

### P-09：分布式 AGENTS.md 网络的组织模式？

**选项 A**：嵌套 + 分层 + 中央仓库（推荐）
```
~/.agents/AGENTS.md                 # 全局规则
  ↓
project-root/AGENTS.md              # 项目规则
  ↓
project-root/.agents/*.md           # 分层上下文
  ↓
agent-standards/ repo               # 跨组织规则
```

**选项 B**：仅嵌套（简化）
```
~/.agents/AGENTS.md
  ↓
project-root/AGENTS.md
  ↓
project-root/packages/*/AGENTS.md
```

**选项 C**：仅中央仓库
```
agent-standards/ repo
  ↓ (CI 生成)
各项目的 AGENTS.md
```

**建议**：选项 A（嵌套 + 分层 + 中央仓库）

**理由**：
1. 与 OpenAI、Datadog 等大型组织的实践对齐
2. 支持个人、项目、组织三个层级
3. 灵活性最高（支持多种使用场景）
4. 向后兼容（现有 AGENTS.md 标准）

---

## 实现路线图

### Phase 1：代码图谱集成（v0.2，2-3 周）

**目标**：集成 CodeMap，支持代码扫描

**交付物**：
- [ ] MCP 客户端适配器
- [ ] `samsara scan` 命令
- [ ] INDEX.md 扩展（code_graphs 字段）
- [ ] 文档 + 示例

**验收标准**：
- 支持 Go, Python, TypeScript 代码库
- 增量扫描（基于文件 hash）
- 输出 JSON 格式的代码图谱

---

### Phase 2：AGENTS.md 自动生成（v0.3，2-3 周）

**目标**：集成 agentmd，支持自动生成 AGENTS.md

**交付物**：
- [ ] agentmd 集成
- [ ] `samsara generate-agents-md` 命令
- [ ] 评分 + 漂移检测
- [ ] GitHub Action 集成
- [ ] 文档 + 示例

**验收标准**：
- 支持 minimal/tiered 模式
- 自动评分（5 个维度）
- 月度漂移检测

---

### Phase 3：分布式网络管理（v0.4，1-2 周）

**目标**：支持多项目 AGENTS.md 网络

**交付物**：
- [ ] 中央仓库同步机制
- [ ] `samsara sync-network` 命令
- [ ] 嵌套 AGENTS.md 加载器
- [ ] 文档 + 示例

**验收标准**：
- 支持嵌套 AGENTS.md 加载
- 支持中央仓库分发
- 支持 AGENTS.override.md

---

## 工具选型详情

### 代码图谱工具：CodeMap

**为什么选 CodeMap？**
- ✅ Go 实现（与 Samsara 同语言）
- ✅ MCP 原生支持
- ✅ LSP 增强（精确类型信息）
- ✅ 实时更新（文件监视）
- ✅ SQLite 存储（持久化）

**集成成本**：低（已是 MCP 服务）

**备选方案**：
- StakGraph（框架感知，16 语言）
- Codebase-Memory（66 语言，学术论文支持）

---

### AGENTS.md 生成工具：agentmd

**为什么选 agentmd？**
- ✅ Python CLI（易于集成）
- ✅ Minimal 模式（减少 token 20%）
- ✅ Tiered 模式（支持分层）
- ✅ 评分系统（5 个维度）
- ✅ 漂移检测（自动检查过时）
- ✅ GitHub Action（CI 集成）

**集成成本**：低（子进程调用）

**备选方案**：
- agents-md-generator（MCP 服务，增量扫描）
- agentseed（简单但功能少）

---

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 外部工具更新不兼容 | 中 | 定期测试，维护适配层 |
| 代码图谱生成性能差 | 中 | 使用增量扫描，后台守护进程 |
| AGENTS.md 质量不稳定 | 低 | 评分 + 漂移检测 + 人工审查 |
| 多项目网络同步冲突 | 低 | git hooks + 原子提交 |

---

## 成功指标

### 短期（v0.2-v0.3）
- [ ] 支持 3+ 语言的代码扫描
- [ ] AGENTS.md 自动生成成功率 ≥ 90%
- [ ] 评分系统准确性 ≥ 85%

### 中期（v0.4）
- [ ] 支持 monorepo 场景
- [ ] 分布式网络同步成功率 ≥ 95%
- [ ] 用户采用率 ≥ 50%

### 长期（v1.0+）
- [ ] 支持 10+ 语言
- [ ] 与 20+ AI 工具兼容
- [ ] 成为行业标准（如 AGENTS.md 本身）

---

## 下一步行动

### 立即（本周）
- [ ] 确认 P-07 ~ P-09 决策
- [ ] 启动 Phase 1 原型实现
- [ ] 建立 CodeMap 集成测试环境

### 短期（2-3 周）
- [ ] 完成 Phase 1（代码图谱集成）
- [ ] 发布 v0.2 alpha
- [ ] 收集用户反馈

### 中期（4-6 周）
- [ ] 完成 Phase 2（AGENTS.md 生成）
- [ ] 完成 Phase 3（分布式网络）
- [ ] 发布 v0.3 / v0.4 正式版

---

## 参考资源

### 完整调研报告
- 📄 `samsara_layer_c_research_2026.md`（752 行，详细分析）

### 工具文档
- CodeMap：https://github.com/asd-noor/codemap
- agentmd：https://github.com/mikiships/agentmd
- agents-md-generator：https://github.com/nushey/agents-md-generator

### 最佳实践
- AGENTS.md 标准：https://agents.md/
- AgentPatterns.ai：https://agentpatterns.ai/
- Codified Context 论文：arXiv 2602.20478

---

## 决策记录

**决策者**: [待填]  
**决策日期**: [待填]  
**P-07 决策**: [待填]  
**P-08 决策**: [待填]  
**P-09 决策**: [待填]  

---

**文档版本**: v0.1  
**最后更新**: 2026-04-23  
**状态**: 待审批
