# Samsara 代码扫描与 AGENTS.md 自动生成调研报告

**日期**: 2026-04-23  
**调研范围**: 大型代码库自动化扫描、AGENTS.md 生成工具、分布式 AGENTS.md 网络组织  
**目标**: 评估 Samsara 是否需要新增"代码扫描层"（Layer C）及技术选型

---

## 执行摘要

### 关键发现

1. **代码图谱生成已成熟生态**：Tree-sitter + LSP 组合已成为行业标准，多个生产级工具可用
2. **AGENTS.md 自动生成工具已出现**：至少 5 个专门工具可从代码库自动生成 AGENTS.md
3. **分布式 AGENTS.md 网络已有最佳实践**：嵌套 AGENTS.md + 路由表模式已被 OpenAI、Datadog 等大型组织验证
4. **MCP 成为事实标准**：所有新工具都通过 MCP 暴露功能，与 Samsara 的跨工具兼容目标完全对齐

### 建议

**Samsara 应新增 Layer C（代码扫描层）**，但**不需要自己实现**。而是：
- 集成现有的 MCP 服务（CodeMap、StakGraph、Codebase-Memory）
- 在 Samsara 的 INDEX.md 中记录这些服务的输出
- 让 Samsara CLI 自动调用这些工具生成初始 AGENTS.md

---

## 第一部分：大型代码库自动化扫描工具

### 1.1 核心技术栈

所有现代工具都采用相同的三层架构：

```
Layer 1: 解析
  ├─ Tree-sitter（AST 解析，66+ 语言）
  ├─ LSP（类型解析、交叉引用）
  └─ 语言特定查询（S-expression）

Layer 2: 图构建
  ├─ 节点提取（函数、类、接口、端点）
  ├─ 边构建（调用、实现、导入）
  └─ 增量更新（基于文件 mtime/hash）

Layer 3: 查询与暴露
  ├─ SQLite 存储（持久化、递归 CTE）
  ├─ MCP 工具（标准化接口）
  └─ CLI（人类友好的输出）
```

### 1.2 主要工具对比

| 工具 | 语言支持 | 核心特性 | 输出格式 | 与 Samsara 兼容性 |
|------|---------|---------|---------|------------------|
| **CodeMap** | Go, Python, JS/TS, Lua, Zig, Templ | LSP 增强、实时更新、递归 CTE | MCP + CLI | ⭐⭐⭐⭐⭐ 完美 |
| **StakGraph** | 16 语言（TS, JS, Python, Go, Rust, Ruby, Java, Kotlin, Swift, C#, PHP, C/C++, Angular, Svelte, Bash, TOML） | 框架感知、端点提取、变更追踪 | MCP + CLI + Neo4j | ⭐⭐⭐⭐⭐ 完美 |
| **Codebase-Memory** | 66 语言 | 知识图谱、MCP 工具集（14 个）、零依赖 C 二进制 | MCP + SQLite | ⭐⭐⭐⭐⭐ 完美 |
| **CodeSift** | 8 语言 | 146 个 MCP 工具、BM25F 搜索、LSP 桥接 | MCP | ⭐⭐⭐⭐⭐ 完美 |
| **CodeLens** | JS/TS, Python | 轻量级、13 个 MCP 工具、死代码检测 | MCP + JSON | ⭐⭐⭐⭐ 很好 |
| **codegraph** | JS/TS, Python, Terraform | 本地 SQLite、函数级追踪、diff 影响分析 | CLI + JSON | ⭐⭐⭐⭐ 很好 |
| **mcp-codebase-intelligence** | 8 语言 | 架构图生成、自然语言查询、18 个 MCP 工具 | MCP | ⭐⭐⭐⭐ 很好 |
| **AstBasedContext-rs** | 多语言 | 102 项代码检查、冗余检测、架构分析 | MCP + CLI | ⭐⭐⭐⭐ 很好 |

### 1.3 推荐选择

**对于 Samsara**：

1. **首选：CodeMap**（Go 实现）
   - 原因：轻量级、LSP 增强、实时更新、MCP 原生
   - 集成成本：低（已是 MCP 服务）
   - 适用场景：多语言项目、需要精确类型信息

2. **备选：StakGraph**（Rust 实现）
   - 原因：框架感知、16 语言支持、变更追踪
   - 集成成本：中等（需要 Neo4j 或使用 CLI 模式）
   - 适用场景：大型单体应用、需要端点映射

3. **备选：Codebase-Memory**（C 实现）
   - 原因：66 语言支持、零依赖、学术论文支持
   - 集成成本：低（单个二进制）
   - 适用场景：超大型代码库、需要最大语言覆盖

---

## 第二部分：AGENTS.md 自动生成工具

### 2.1 现有工具生态

#### 工具 1：agents-md-generator（MCP 服务）

**GitHub**: nushey/agents-md-generator  
**语言**: Python 3.11+  
**核心特性**:
- Tree-sitter 解析 + 增量扫描
- 三个 MCP 工具：`generate_agents_md`、`scan_codebase`、`read_payload_chunk`
- 支持语言：Python, C#, TypeScript, JavaScript, Go
- 项目大小配置：small/medium/large（自动调整压缩）
- 缓存位置：`~/.cache/agents-md-generator/<project-hash>/`

**生成内容**:
```
- Project Overview
- Architecture & Data Flow
- Conventions & Patterns
- Environment Variables
- Setup Commands
- Development Workflow
- Testing Instructions
- Code Style
- Build and Deployment
```

**与 Samsara 的兼容性**: ⭐⭐⭐⭐⭐ 完美
- 已是 MCP 服务，可直接集成
- 支持增量扫描（与 Samsara 的"被动积累"模式兼容）
- 输出格式完全符合 agents.md 标准

**集成方式**:
```bash
# 作为 MCP 服务
pip install agents-md-generator
agents-md-generator setup  # 自动配置到 Claude Code / Cursor / Windsurf

# 或通过 uvx（无需安装）
uvx agents-md-generator
```

---

#### 工具 2：agentmd（CLI + GitHub Action）

**GitHub**: mikiships/agentmd  
**语言**: Python 3.10+  
**核心特性**:
- 支持多个代理格式：CLAUDE.md, AGENTS.md, .cursorrules, .github/copilot-instructions.md
- **Minimal 模式**：研究表明冗长的 AGENTS.md 会降低代理性能 20%，minimal 模式只保留最高价值内容
- **Tiered 模式**：自动检测子系统边界，生成 Tier 1 (CLAUDE.md) + Tier 2 (.agents/*.md)
- **Drift 检测**：自动检测 AGENTS.md 是否过时
- **评分系统**：5 个维度评分（完整性、特异性、清晰度、代理感知、新鲜度）
- **GitHub Action**：自动在 PR 中检测 drift

**生成流程**:
```
scan → detect languages/frameworks/commands
  ↓
generate → create context files (minimal/full/tiered)
  ↓
eval → measure performance impact (需要 coderace)
  ↓
drift → detect staleness
```

**与 Samsara 的兼容性**: ⭐⭐⭐⭐⭐ 完美
- 支持多格式输出（与跨工具兼容目标一致）
- Minimal 模式符合 Samsara 的"高信噪比"设计哲学
- Tiered 模式与 Samsara 的分层知识结构对齐

**集成方式**:
```bash
pip install agentmd-gen

# 生成
agentmd generate --minimal --agent codex  # AGENTS.md

# 检测 drift
agentmd drift --agent codex

# 评分
agentmd score AGENTS.md
```

---

#### 工具 3：agentseed（CLI）

**GitHub**: avinshe/agentseed  
**语言**: TypeScript  
**核心特性**:
- 一行命令生成 AGENTS.md
- 自动检测 monorepo 子项目
- 支持 20+ AI 工具
- 跟踪 git SHA，增量重新生成

**与 Samsara 的兼容性**: ⭐⭐⭐⭐ 很好
- 简单易用，但功能不如 agentmd 丰富

---

#### 工具 4：Caliber（CLI）

**特性**:
- 自动发现 MCP 服务并配置
- Git hook 自动更新 AGENTS.md
- 支持 Claude Code, Cursor, OpenAI Codex

**与 Samsara 的兼容性**: ⭐⭐⭐⭐ 很好
- Git hook 集成与 Samsara 的"被动积累"模式兼容

---

#### 工具 5：AgentBrain（CLI）

**特性**:
- 生成三份文档：context.md, dependency-map.md, patterns.md
- 自动 git hook 后台重新生成
- 智能过滤（仅在源代码变化时重新生成）

**与 Samsara 的兼容性**: ⭐⭐⭐⭐ 很好
- 后台自动更新符合 Samsara 的被动模式

---

### 2.2 AGENTS.md 生成的最佳实践

#### 关键发现（来自 Russell Clare 的研究）

1. **Next.js v16.2 现已内置 AGENTS.md 生成**
   - 自动生成，版本匹配的文档捆绑在 node_modules 中
   - 解决了"代理读过时训练数据"的问题

2. **LLM 生成的 AGENTS.md 质量问题**
   - 纯 LLM 生成的文件相比人工编写降低 20% 的成功率
   - 最有价值的内容：精确的构建/测试/lint 命令
   - 最无价值的内容：通用提示、风格指南、反模式

3. **Minimal 模式的优势**
   - 研究表明冗长的 AGENTS.md 会增加推理成本 20%
   - 最优策略：只包含代理无法自己推断的内容
   - 推荐内容：
     - 一行项目描述
     - 构建/测试/lint 命令（最高价值）
     - 源代码和测试目录根
     - 一个 `/compact` 提示（对于 Claude）

---

## 第三部分：分布式 AGENTS.md 网络组织

### 3.1 嵌套 AGENTS.md 模式（已验证）

**已被采用的组织**：OpenAI（88 个 AGENTS.md 文件）、Datadog、Google

#### 基本原则

```
repo-root/
├── AGENTS.md                    # 全局规则（始终加载）
├── packages/
│   ├── api/
│   │   └── AGENTS.md            # API 特定规则（最近的文件优先）
│   ├── web/
│   │   └── AGENTS.md            # Web 特定规则
│   └── shared/
│       └── AGENTS.md            # 共享库规则
└── infrastructure/
    └── AGENTS.md                # IaC 规则
```

**加载顺序**（从 Codex 的实现）：
1. 全局作用域：`~/.codex/AGENTS.md` 或 `~/.codex/AGENTS.override.md`
2. 项目作用域：从 git root 到当前目录的所有 AGENTS.md
3. 合并顺序：从根到当前目录，**后面的文件覆盖前面的**

#### 高级模式：分层上下文（Tiered Context）

**来源**：Codified Context 论文（arXiv 2602.20478）

单文件 AGENTS.md 不能扩展到 1000+ 行。解决方案：

```
project/
├── AGENTS.md                    # Tier 1: 路由表 + 全局规则（~30 行）
└── .agents/
    ├── api.md                   # Tier 2: API 子系统（~100 行）
    ├── database.md              # Tier 2: 数据库层（~100 行）
    └── web.md                   # Tier 2: Web 前端（~100 行）
```

**Tier 1 AGENTS.md 的路由表**：
```markdown
## Context Files (load when working in these areas)
| Directory | Context File |
|-----------|-------------|
| api/      | .agents/api.md |
| db/       | .agents/database.md |
| web/      | .agents/web.md |
```

**优势**：
- 代理只加载相关的上下文（节省 token）
- 每个子系统可由其所有者维护
- 全局规则与本地规则分离

---

### 3.2 跨项目 AGENTS.md 网络

#### 模式 1：中央仓库 + 分发（Canonicalize-then-Fan-Out）

**已被采用的组织**：Nx monorepo、Datadog

```
agent-standards/                # 中央仓库
├── AGENTS.md                   # 自描述
├── skills/                      # 共享 skills
├── templates/                   # 新项目模板
└── conventions/                 # 标准化规则

# CI 作业：
# 1. 从中央仓库读取规则
# 2. 为每个下游项目生成 AGENTS.md
# 3. 提交到各项目
```

**关键决策**：
- 集中化什么：跨组织统一的标准（CI 命令、命名规范、禁止模式）
- 保持本地化什么：项目特定的架构决策

#### 模式 2：全局配置 + 项目覆盖

**来源**：Codex 的实现

```
~/.codex/AGENTS.md              # 全局默认值
  ↓
repo-root/AGENTS.md             # 项目级覆盖
  ↓
repo-root/packages/api/AGENTS.md # 目录级覆盖
```

**优先级**：
1. 全局配置：多仓库上下文、个人工作流偏好
2. 项目级：项目范围的执行规则
3. 目录级：特定区域的约束

**特殊文件**：
- `AGENTS.override.md`：完全替换（不继承）
- `AGENTS.local.md`：用户本地覆盖（.gitignore）

---

### 3.3 多代理协调模式

#### 问题：多个代理同时写入同一文件会导致数据损坏

#### 解决方案（来自 The Prompt Shelf）

1. **角色分离**：在 AGENTS.md 中明确定义每个代理的职责
   ```markdown
   ## Agent Roles
   - **Architect**: Modifies architecture/ and design/
   - **Backend**: Modifies services/ and api/
   - **Frontend**: Modifies web/ and components/
   ```

2. **共享状态约定**：
   ```markdown
   ## Shared State Conventions
   - Never modify files outside your role's directory
   - Use atomic commits (one file per commit)
   - Always run tests before committing
   ```

3. **冲突检测**：
   - 使用 git hooks 检测并拒绝跨角色修改
   - 在 AGENTS.md 中记录最后修改的代理和时间戳

---

### 3.4 与 Samsara 的对齐

**Samsara 的分布式 AGENTS.md 网络设计**：

```
~/.agents/                      # Layer 1: 全局知识库
├── AGENTS.md                   # 全局规则
├── skills/                      # 共享 skills
├── lessons-learned.md           # 错误日志
└── INDEX.md                     # 全量索引

project-root/                   # Layer 2: 项目级
├── AGENTS.md                   # 项目规则
├── .agents/                     # 子系统文档
│   ├── api.md
│   ├── database.md
│   └── web.md
└── docs/                        # 项目文档

monorepo/                        # Layer 3: Monorepo
├── AGENTS.md                   # Monorepo 规则
├── packages/
│   ├── package-a/
│   │   └── AGENTS.md           # 包特定规则
│   └── package-b/
│       └── AGENTS.md
```

**Samsara 的优势**：
- 与现有 AGENTS.md 标准完全兼容
- 支持嵌套和分层
- 可与中央仓库模式集成
- 自动生成工具已成熟

---

## 第四部分：Samsara Layer C 设计建议

### 4.1 架构决策

**决策 1：是否自己实现代码扫描？**

**答案**：否。理由：
- 现有工具已成熟且生产级
- 维护成本高（66+ 语言支持）
- MCP 标准化使集成成本低
- 社区活跃，持续更新

**决策 2：采用哪个工具？**

**推荐**：多工具支持策略
```
Samsara Layer C = 工具适配层

用户可选：
- CodeMap（默认，轻量级）
- StakGraph（框架感知）
- Codebase-Memory（最大语言覆盖）
- 用户自定义 MCP 服务
```

**决策 3：AGENTS.md 生成工具？**

**推荐**：agentmd（CLI）+ agents-md-generator（MCP）
- agentmd：本地 CLI，支持 minimal/tiered 模式
- agents-md-generator：MCP 服务，支持增量扫描

---

### 4.2 Samsara Layer C 的三阶段实现

#### Phase 1：代码图谱集成（v0.2）

```rust
// samsara/src/layer_c/mod.rs

pub trait CodeGraphProvider {
    fn scan(&self, path: &Path) -> Result<CodeGraph>;
    fn get_symbols(&self, file: &Path) -> Result<Vec<Symbol>>;
    fn get_dependencies(&self, symbol: &Symbol) -> Result<Vec<Dependency>>;
}

// 实现：MCP 客户端适配器
pub struct MCPCodeGraphAdapter {
    mcp_client: MCPClient,
}

impl CodeGraphProvider for MCPCodeGraphAdapter {
    fn scan(&self, path: &Path) -> Result<CodeGraph> {
        // 调用 MCP 工具：codemap::index 或 stakgraph::parse
        self.mcp_client.call_tool("index", json!({ "path": path }))
    }
}
```

**输出**：
- `~/.agents/code-graphs/<project-hash>/graph.json`
- 在 INDEX.md 中记录：`code_graph_provider: "codemap"`, `last_scanned: "2026-04-23T10:30:00Z"`

#### Phase 2：AGENTS.md 自动生成（v0.3）

```rust
// samsara/src/layer_c/agents_md_generator.rs

pub struct AgentsMdGenerator {
    code_graph: CodeGraph,
    config: AgentsMdConfig,
}

impl AgentsMdGenerator {
    pub fn generate(&self) -> Result<String> {
        // 1. 从 code_graph 提取架构
        let architecture = self.extract_architecture();
        
        // 2. 调用 agentmd 或 agents-md-generator
        let agents_md = self.call_generator(architecture)?;
        
        // 3. 在 INDEX.md 中记录
        self.record_generation_metadata()?;
        
        Ok(agents_md)
    }
}
```

**输出**：
- `project-root/AGENTS.md`（或 `.agents/` 中的分层文件）
- INDEX.md 记录：`agents_md_generated: true`, `generator: "agentmd"`, `mode: "minimal"`

#### Phase 3：分布式网络管理（v0.4）

```rust
// samsara/src/layer_c/network.rs

pub struct DistributedAgentsMdNetwork {
    central_repo: Path,
    projects: Vec<ProjectRef>,
}

impl DistributedAgentsMdNetwork {
    pub fn sync(&self) -> Result<()> {
        // 1. 从中央仓库读取规则
        let central_rules = self.load_central_rules()?;
        
        // 2. 为每个项目生成 AGENTS.md
        for project in &self.projects {
            let agents_md = self.generate_for_project(project, &central_rules)?;
            project.write_agents_md(agents_md)?;
        }
        
        // 3. 更新 INDEX.md
        self.update_network_index()?;
    }
}
```

---

### 4.3 与现有 Samsara 组件的集成

#### INDEX.md 扩展

```yaml
# INDEX.md

code_graphs:
  - provider: "codemap"
    project_hash: "abc123def456"
    last_scanned: "2026-04-23T10:30:00Z"
    node_count: 1234
    edge_count: 5678
    languages: ["go", "python", "typescript"]

agents_md:
  - path: "AGENTS.md"
    generator: "agentmd"
    mode: "minimal"
    generated_at: "2026-04-23T10:35:00Z"
    version: "0.6.0"
    dimensions:
      completeness: 18
      specificity: 17
      clarity: 16
      agent_awareness: 18
      freshness: 15
    total_score: 84

distributed_network:
  central_repo: "https://github.com/org/agent-standards"
  sync_status: "in_sync"
  last_sync: "2026-04-23T09:00:00Z"
  projects_managed: 12
```

#### Lesson 文件集成

```markdown
# code-scanning.md

## 最佳实践

### 代码图谱生成
- 使用 CodeMap 处理多语言项目（Go, Python, TS）
- 使用 StakGraph 处理框架感知场景（Express, FastAPI）
- 使用 Codebase-Memory 处理超大型代码库（66+ 语言）

### AGENTS.md 生成
- 优先使用 minimal 模式（减少 token 消耗 20%）
- 对于 monorepo，使用 tiered 模式（Tier 1 + .agents/）
- 每周运行 `agentmd drift` 检测过时内容

### 分布式网络
- 中央仓库存储跨组织规则
- 项目级 AGENTS.md 存储项目特定规则
- 使用 AGENTS.override.md 处理例外情况
```

---

## 第五部分：技术选型总结

### 5.1 推荐的工具组合

| 组件 | 推荐工具 | 备选方案 | 集成方式 |
|------|---------|---------|---------|
| 代码图谱 | CodeMap | StakGraph / Codebase-Memory | MCP 客户端 |
| AGENTS.md 生成 | agentmd (CLI) | agents-md-generator (MCP) | 子进程调用 |
| 分布式管理 | 自定义（基于 git + INDEX.md） | Nx monorepo 工具 | git hooks |
| 增量更新 | 基于文件 hash | 基于 git diff | 后台守护进程 |

### 5.2 实现成本估算

| 阶段 | 工作量 | 时间 | 依赖 |
|------|--------|------|------|
| Phase 1: 代码图谱集成 | 中等 | 2-3 周 | MCP 客户端库 |
| Phase 2: AGENTS.md 生成 | 中等 | 2-3 周 | agentmd 集成 |
| Phase 3: 分布式网络 | 低 | 1-2 周 | git + INDEX.md |

### 5.3 与 Samsara 现有设计的兼容性

✅ **完全兼容**：
- MCP 标准与 Samsara 的跨工具兼容目标一致
- 增量扫描与"被动积累"模式对齐
- INDEX.md 全量重建方案可扩展支持代码图谱元数据
- 分层 AGENTS.md 与 Samsara 的分层知识结构对齐

⚠️ **需要注意**：
- 代码图谱存储位置：建议 `~/.agents/code-graphs/` 而非项目内
- 缓存管理：需要定期清理过期的图谱数据
- 权限管理：多项目场景下的访问控制

---

## 第六部分：决策清单

### P-07：是否实现 Layer C（代码扫描层）？

**建议**：是，但采用"集成而非实现"策略

**理由**：
1. 现有工具已成熟（生产级，社区活跃）
2. MCP 标准化使集成成本低
3. 维护成本高（66+ 语言支持）
4. 与 Samsara 的跨工具兼容目标完全对齐

**实现方式**：
- 不自己实现代码扫描
- 集成 CodeMap / StakGraph / Codebase-Memory
- 在 INDEX.md 中记录扫描元数据
- 提供 CLI 命令自动调用这些工具

---

### P-08：AGENTS.md 自动生成的质量保证？

**建议**：采用"生成 + 评分 + 漂移检测"三层策略

**实现**：
1. 使用 agentmd 生成（支持 minimal/tiered 模式）
2. 使用 agentmd score 评分（5 个维度）
3. 使用 agentmd drift 检测过时（CI 集成）

**质量目标**：
- 总分 ≥ 80/100
- 新鲜度 ≥ 15/20
- 月度漂移检测（自动 PR 提醒）

---

### P-09：分布式 AGENTS.md 网络的组织模式？

**建议**：采用"嵌套 + 分层 + 中央仓库"混合模式

**实现**：
```
全局层：~/.agents/AGENTS.md（个人偏好）
  ↓
项目层：project-root/AGENTS.md（项目规则）
  ↓
子系统层：project-root/.agents/*.md（分层上下文）
  ↓
中央仓库：agent-standards/ repo（跨组织规则）
```

**优势**：
- 支持个人、项目、组织三个层级
- 后面的文件覆盖前面的（优先级清晰）
- 与 OpenAI、Datadog 等大型组织的实践对齐

---

## 附录 A：工具快速参考

### CodeMap
```bash
# 安装
go build -o codemap .

# 作为 MCP 服务
codemap serve

# 作为 CLI
codemap index
codemap symbols internal/graph/store.go
codemap impact NodeID
```

### agentmd
```bash
# 安装
pip install agentmd-gen

# 生成（minimal 模式推荐）
agentmd generate --minimal --agent codex

# 评分
agentmd score AGENTS.md

# 漂移检测
agentmd drift --agent codex
```

### agents-md-generator
```bash
# 安装
pip install agents-md-generator
agents-md-generator setup

# 或通过 uvx
uvx agents-md-generator
```

---

## 附录 B：参考资源

### 学术论文
- Codebase-Memory: Tree-Sitter-Based Knowledge Graphs for LLM Agents (arXiv 2603.27277)
- Codified Context: Tiered Context Architecture (arXiv 2602.20478)
- Evaluating AGENTS.md (arXiv 2602.11988)

### 官方文档
- agents.md 标准：https://agents.md/
- Next.js AI Agents 指南：https://nextjs.org/docs/app/guides/ai-agents
- Aider Repository Map：https://aider.chat/docs/repomap.html

### 最佳实践指南
- AgentPatterns.ai：https://agentpatterns.ai/
- The Prompt Shelf：https://thepromptshelf.dev/
- Addy Osmani 的 AGENTS.md 指南：https://addyosmani.com/agents/

---

## 结论

Samsara 应该新增 Layer C（代码扫描层），但**不需要自己实现**。通过集成现有的 MCP 服务（CodeMap、StakGraph、Codebase-Memory）和 AGENTS.md 生成工具（agentmd、agents-md-generator），Samsara 可以：

1. **自动生成初始 AGENTS.md**（从代码库自动提取）
2. **维护代码图谱**（支持多语言、增量更新）
3. **管理分布式 AGENTS.md 网络**（嵌套 + 分层 + 中央仓库）
4. **保证质量**（评分 + 漂移检测）

这样既能充分利用社区成果，又能保持 Samsara 的核心价值：**跨工具兼容、被动积累、自我进化**。

