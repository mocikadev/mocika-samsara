# samsara CLI · 软件实现文档

> **文档版本**：v0.4  
> **状态**：草稿，待评审  
> **上次更新**：2026-04-23  
> **关联产品文档**：`samsara-design.md` §18（同目录 `docs/`）

---

## 目录

1. [项目概述](#1-项目概述)
2. [数据模型](#2-数据模型)
3. [模块结构](#3-模块结构)
4. [各命令实现](#4-各命令实现)
5. [错误处理](#5-错误处理)
6. [测试策略](#6-测试策略)
7. [构建与发布](#7-构建与发布)

---

## 1. 项目概述

### 定位

`samsara` 是 Samsara 知识系统的 Layer 2 管理工具，负责 `~/.agents/knowledge/` 目录下所有文件的**写入、格式校验、晋升和归档**。

与 `skm`（管理 `~/.agents/skills/`，Layer 1）共同构成 Samsara 的工具层。

### 目录结构（Rust 项目）

```
samsara/
├── Cargo.toml
├── src/
│   ├── main.rs          ← CLI 入口，clap 路由
│   ├── cli.rs           ← 命令定义（clap derive）
│   ├── config.rs        ← 读取 SAMSARA_HOME 等环境变量
│   ├── knowledge/
│   │   ├── mod.rs
│   │   ├── lesson.rs    ← Lesson 数据模型 + frontmatter 解析
│   │   ├── rules.rs     ← Rules 文件操作
│   │   ├── index.rs     ← INDEX.md 读写
│   │   ├── log.rs       ← log.md 追加
│   │   └── aaak.rs      ← AaakEntry 数据模型 + AGENTS.md ## AAAK section 读写
│   └── commands/
│       ├── mod.rs
│       ├── init.rs      ← samsara init（目录创建 + git init + symlink + .gitattributes）
│       ├── write.rs
│       ├── search.rs    ← samsara search（全文搜索 lessons/ + rules/）
│       ├── promote.rs
│       ├── domain.rs    ← samsara domain list|add
│       ├── archive.rs
│       ├── lint.rs
│       ├── status.rs
│       ├── log.rs       ← samsara log + log rotate
│       ├── prime.rs     ← samsara prime（Top N 规则提炼，stdout 输出）
│       ├── demote.rs    ← samsara demote（AGENTS.md 规则降级）
│       ├── remote.rs    ← samsara remote add/set/show
│       ├── reflect.rs   ← samsara reflect（静态分析）
│       └── skill_note.rs← samsara skill-note
└── tests/
    ├── fixtures/        ← 测试用的 lesson/index/rules 文件
    └── integration/     ← 端到端命令测试
```

---

## 2. 数据模型

### 2.1 Lesson Frontmatter

对应 `knowledge/lessons/[domain]/[keyword].md` 文件头部的 YAML。

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 记忆类型（来源：DNA Memory 调研，§10 设计）
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LessonType {
    Error,    // 踩坑 / 错误教训
    Skill,    // 学到的技能 / 方法
    Pattern,  // 归纳的模式
    Insight,  // 深层洞察
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LessonFrontmatter {
    pub date: NaiveDate,
    pub domain: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub lesson_type: Option<LessonType>,        // 可选；不填时 prime/reflect 按"未分类"处理
    pub tags: Vec<String>,
    pub occurrences: Vec<NaiveDate>,            // 时间戳数组，只追加不覆盖
    pub promoted: bool,
    #[serde(default)]
    pub verified: u32,                           // 规则被验证有效的次数；初始 0
    pub valid_until: Option<NaiveDate>,          // 可选，超期自动归档
    pub conflicts_with: Option<Vec<String>>,     // 可选，格式 "<domain>/<keyword>"
}

pub struct Lesson {
    pub frontmatter: LessonFrontmatter,
    pub body: String,   // frontmatter 之后的 Markdown 正文
    pub path: PathBuf,  // 文件路径（用于写回）
}
```

**frontmatter 解析方式**：文件以 `---\n` 开头，找到第二个 `---\n`，中间内容用 `serde_yaml` 解析，其余为 body。

### 2.2 INDEX 条目

```rust
pub struct IndexDomainEntry {
    pub domain: String,
    pub tags: Vec<String>,
    pub lesson_count: usize,
    pub has_rules: bool,
    pub related_skill: Option<String>,
    pub last_written: Option<NaiveDate>,
}
```

INDEX.md 是**衍生数据**，不是权威数据。权威数据是 `lessons/` 目录下的文件本身。
每次写操作完成后，`index.rs` 扫描整个 `lessons/` 和 `rules/` 目录，**整体重建** INDEX.md，
而非行级更新。这消除了正则替换的脆弱性，并保证 INDEX 和文件系统永远一致。

### 2.3 Log 条目

```rust
pub enum LogAction {
    Write,      // 新建 lesson
    Update,     // 追加 occurrence
    Promote,    // lesson → rules
    Archive,    // 移入 archive/
    Lint,       // lint 检查发现问题
    SkillUse,   // skill 成功使用（来源：daerwen §3.7.4）
    SkillFail,  // skill 报错/失效（来源：daerwen §3.7.4）
}

pub struct LogEntry {
    pub date: NaiveDate,
    pub action: LogAction,
    pub target: String,   // 如 "rust/cargo-fmt-vs-clippy.md" 或 skill 名称
    pub note: Option<String>,
}
```

写入格式（append-only，一行一条）：
```
2026-04-22 WRITE      rust/cargo-fmt-vs-clippy.md (occurrences: 1)
2026-04-28 UPDATE     rust/cargo-fmt-vs-clippy.md (occurrences: 2)
2026-05-10 PROMOTE    rust/cargo-fmt-vs-clippy.md → rules/rust.md
2026-04-22 SKILL_USE  rust-skills (task: cargo build 优化)
2026-04-28 SKILL_FAIL rust-skills (load 报错: SKILL.md 缺少 tags 字段)
```

---

### 2.4 AAAK 条目（`~/.agents/AGENTS.md` 的 `## AAAK` section）

对应 AGENTS.md 末尾 `## AAAK` section 中每一行的结构化数据（来源：daerwen §3.7.1；存储位置决策：v0.7——独立 aaak.md 无工具自动加载保证，废弃）：

```rust
/// 解析自 AGENTS.md ## AAAK section 的单条记录，格式：[entity|relation|value|date]
pub struct AaakEntry {
    pub entity: String,
    pub relation: String,
    pub value: String,
    pub date: NaiveDate,
}

impl AaakEntry {
    /// 序列化为 ## AAAK section 中的一行
    pub fn to_line(&self) -> String {
        format!("[{}|{}|{}|{}]", self.entity, self.relation, self.value, self.date)
    }

    /// 从一行文本解析，格式不符返回 None
    pub fn from_line(line: &str) -> Option<Self>;
}

/// 从 AGENTS.md 的 ## AAAK section 加载所有条目
/// 剔除超出预算（480 字符 ≈ 120 tokens）的最旧条目后写回 AGENTS.md
pub fn load_and_trim(agents_md_path: &Path, budget_chars: usize) -> Result<Vec<AaakEntry>>;

/// 在 AGENTS.md 的 ## AAAK section 追加条目
/// 写前检查同 entity+relation 是否已存在，存在则更新 value/date
/// 若 ## AAAK section 不存在，则在文件末尾追加 section
pub fn append_entry(agents_md_path: &Path, entry: &AaakEntry, dry_run: bool) -> Result<()>;

/// 解析 AGENTS.md，定位 ## AAAK section 的开始和结束行号
/// 返回 (section_start_line, entries) 或 None（section 不存在）
pub fn find_aaak_section(agents_md_path: &Path) -> Result<Option<(usize, Vec<AaakEntry>)>>;
```

---

## 3. 模块结构

```rust
pub struct Config {
    pub knowledge_home: PathBuf,  // 默认 ~/.agents/knowledge/
    pub agents_home: PathBuf,     // 默认 ~/.agents/
    pub dry_run: bool,
    pub auto_commit: bool,        // 默认 true，每次写操作后 git commit
}
```

优先级：`--home` flag > `SAMSARA_HOME` 环境变量 > `~/.agents/knowledge/`

**git 集成**（来源：daerwen §3.7.3）：

`config.auto_commit = true` 时，每个写命令（write/promote/archive/skill-note）在操作完成后执行：

```rust
// git 操作通过 std::process::Command 调用，不引入 git2 crate（保持零重依赖）
fn auto_commit(knowledge_home: &Path, message: &str) -> Result<()> {
    Command::new("git").args(["add", "-A"]).current_dir(knowledge_home).status()?;
    Command::new("git").args(["commit", "-m", message]).current_dir(knowledge_home).status()?;
    Ok(())
}
// 调用示例：auto_commit(&config.knowledge_home, "samsara: write rust/cargo-fmt")?;
```

若 knowledge_home 不是 git repo，`auto_commit` 静默跳过（不报错），提示用户运行 `samsara init`。

### 3.2 knowledge/lesson.rs — 核心操作

```rust
impl Lesson {
    /// 从文件路径加载
    pub fn load(path: &Path) -> Result<Self>;

    /// 追加一条 occurrence（今日日期）
    pub fn add_occurrence(&mut self, date: NaiveDate);

    /// 判断是否满足晋升条件
    pub fn should_promote(&self) -> bool {
        self.frontmatter.occurrences.len() >= 3 && !self.frontmatter.promoted
    }

    /// 判断是否过期（valid_until 已过）
    pub fn is_expired(&self, today: NaiveDate) -> bool {
        self.frontmatter.valid_until
            .map(|d| d < today)
            .unwrap_or(false)
    }

    /// 写回文件（frontmatter 序列化 + body）
    pub fn save(&self, dry_run: bool) -> Result<()>;
}

/// 在指定 domain 目录下查找 keyword.md
pub fn find_lesson(knowledge_home: &Path, domain: &str, keyword: &str) -> Option<PathBuf>;
```

### 3.3 knowledge/index.rs

INDEX.md **全量重建**，不做增量更新：

```rust
/// 扫描 knowledge_home 下所有 lessons/ 和 rules/ 目录，重新生成完整 INDEX.md
/// 任何写操作（write/promote/archive）执行完毕后调用此函数
pub fn rebuild(knowledge_home: &Path, dry_run: bool) -> Result<()>;

/// 仅用于 status 命令的只读扫描，不写文件
pub fn scan(knowledge_home: &Path) -> Result<Vec<IndexDomainEntry>>;
```

**rebuild 的生成逻辑**：
```
1. 遍历 lessons/ 子目录 → 按 domain 分组，统计 lesson_count、last_written
2. 检查 rules/<domain>.md 是否存在 → 填充 has_rules
3. 从各 lesson 的 tags 聚合该 domain 的 tag 列表
4. 按 domain 名字母序排列
5. 整体写入 INDEX.md（覆盖旧文件）
```

### 3.4 knowledge/log.rs

```rust
pub fn append_log(log_path: &Path, entry: &LogEntry, dry_run: bool) -> Result<()>;
pub fn read_last_n(log_path: &Path, n: usize) -> Result<Vec<LogEntry>>;
```

---

## 4. 各命令实现

### 4.1 `samsara write <domain> <keyword>`

```
Algorithm:
  1. Domain 验证（filesystem-based）：
     检查 lessons/<domain>/ 目录是否存在
     ├─ 存在 → 继续
     └─ 不存在：
         ├─ 有 --yes flag → 静默 mkdir lessons/<domain>/
         └─ 无 --yes flag → 交互提示 "'<domain>' 是新 domain，是否创建？[y/N]"
                            N → 打印已有 domain 列表，退出
                            Y → mkdir lessons/<domain>/
   2. find_lesson(domain, keyword)
      ├─ Some(path) → Lesson::load(path)
      │               lesson.add_occurrence(today)
      │               若有 --verify flag：lesson.frontmatter.verified += 1
      │               若有 --update flag：
      │                 打开 $EDITOR 修改 body（或 --summary 直接覆盖 body）
      │               若有 --type flag：lesson.frontmatter.lesson_type = Some(type)
      │               lesson.save()
      │               log WRITE/UPDATE（--verify 时附注 verified=N）
      └─ None → 创建 lessons/<domain>/<keyword>.md
                frontmatter 初始化（occurrences: [today], verified: 0）
                若有 --type flag：lesson_type = Some(type)
                如有 --summary flag → 直接写 body
                否则 → 打开 $EDITOR（跳过则留空，提示用户后续手动填）
                log WRITE

  3. index::rebuild()   ← 统一在最后重建，无论新建还是更新
  4. auto_commit("samsara: write <domain>/<keyword>")
  5. 检查 lesson.should_promote() → 若是，打印提示：
     "⚡ occurrences 已达 3 次，可执行 samsara promote <domain> <keyword>"
```

### 4.2 `samsara promote <domain> <keyword>`

```
Algorithm:
  1. find_lesson → 找不到则报错
  2. 验证 occurrences.len() >= 3，否则拒绝（打印当前次数）
  3. 读取 lesson body，提取"规则"部分
  4. 追加到 rules/<domain>.md（文件不存在则创建）
     格式：
       ## <keyword>
       来源：lessons/<domain>/<keyword>.md（occurrences: N）
       <规则内容>
       ---
   5. lesson.frontmatter.promoted = true → lesson.save()
   6. log PROMOTE
   7. index::rebuild()
   8. 若 --aaak flag：
      a. 提示用户输入 AAAK 条目（entity/relation/value），或从规则中自动提炼
      b. aaak::append_entry(&agents_home/"AGENTS.md", entry)
         ← 写入 AGENTS.md 的 ## AAAK section（若 section 不存在则追加到文件末尾）
      c. aaak::load_and_trim(&agents_home/"AGENTS.md", 480)
         ← 检查预算，超出时按 date 升序删除最旧条目
    9. auto_commit("samsara: promote <domain>/<keyword>")
   10. 询问：是否为"绝不/必须"级别？→ 若是，提示用户执行 samsara promote --layer0

若 --layer0 flag：（晋升到 AGENTS.md 实质规则区，不走上述步骤 4-9，独立安全算法）
   1. dry-run 预览：打印将写入 AGENTS.md 的行内容，询问"确认写入？[y/N]"
      （--yes flag 跳过确认；--dry-run flag 仅输出预览后退出，不继续后续步骤）
   2. 备份：cp ~/.agents/AGENTS.md ~/.agents/.backup/AGENTS.md.bak
      （覆盖式，只保留最近 1 份）
   3. 行数检查：统计 AGENTS.md 实质规则行数（排除 ## AAAK section、空行、纯注释行）
      当前行数 + 新增行数 ≤ 100 → 继续
      超出 → 拒绝并打印：
        "AGENTS.md 实质规则已有 N 行，新增后将超过 100 行上限。
         请先运行 `samsara demote <domain> <keyword>` 降级低优先级规则后重试。"
        删除 .bak 文件，退出
   4. 写入：追加规则行到 AGENTS.md 实质规则区（## AAAK section 之前）
   5. 写入 log.md：LAYER0 <domain>/<keyword>.md → AGENTS.md
   6. git commit -m "samsara: promote --layer0 <domain>/<keyword>"
      删除 AGENTS.md.bak

注：--aaak 与 --layer0 可同时使用。若同时提供两个 flag，step 6（--aaak 处理）在 --layer0 流程全部完成后额外执行。
```

### 4.3 `samsara lint`

```
检查项（共 13 项，编号与设计文档 §18.2 对应）：

ERROR:
  ① lesson 文件 > 30 行                                      ❌ 需人工拆分
  ③ frontmatter 缺必填字段（date/domain/tags/occurrences/promoted）❌ 需人工补全
  ④ occurrences 非数组或含非 ISO-8601 日期格式               ❌ 需人工修正

WARN:
  ② rules/<domain>.md 超过 100 行                            ❌ 需人工整理
  ⑤ valid_until 已过期（今日 > valid_until）                 ✅ --fix 移入 archive/
  ⑥ lesson 90 天无新 write（promoted=false 的孤立记录）       ✅ --fix 移入 archive/
     *注：occurrences 追加 = 引用；AI 读取文件不计入，不重置计时器*
  ⑦ promoted=true 但 rules/<domain>.md 中无对应条目          ❌ 需人工确认
  ⑧ INDEX.md 中的 domain 与实际 lessons/ 子目录不一致        ✅ --fix 重建 INDEX.md
  ⑨ AGENTS.md 实质规则行数（不含 ## AAAK section）> 100     ❌ 需人工（建议 samsara demote）
  ⑩ rules/ 文件中引用的 lesson 路径不存在（死链）            ❌ 需人工
  ⑫ conflicts_with 列出的 keyword 在 lessons/ 或 rules/ 中
     均不存在（悬空引用）                                    ❌ 仅报告

INFO:
  ⑪ log.md 行数 > 1000                                       ✅ --fix 执行 samsara log rotate
  ⑬ 同 domain 内存在 tags 高度重叠的 lesson 对               ❌ 建议人工合并
     （Jaccard 相似度 ≥ 0.7，可能是同一根因的不同描述）
     实现：对每对 lesson 计算 tags 集合 Jaccard = |A∩B| / |A∪B|；
           仅输出候选对，不自动合并

输出格式：
  [ERROR] lessons/rust/cargo-fmt-vs-clippy.md: 缺少必填字段 'occurrences'
  [WARN]  lessons/git/rebase-conflict.md: 文件 38 行，超过 30 行上限
  [WARN]  AGENTS.md 实质规则 105 行，已超过 100 行上限
  [INFO]  蒸馏候选: rust/cargo-fmt + rust/fmt-check (tags Jaccard: 0.83)

退出码：有 ERROR → 1，仅 WARN/INFO → 0

--fix 自动修复项（⑤⑥⑧⑪），执行顺序：
  1. 收集 ⑤⑥ 过期/孤立 lesson 列表 → 逐条询问（--yes 批量确认）→ 移入 archive/
  2. ⑧ 重建 INDEX.md
  3. 步骤 1+2 合并一次 git commit: "samsara: lint --fix (archive N lessons, rebuild INDEX)"
  4. ⑪ 调用 log rotate 逻辑（含独立 git commit: "samsara: log rotate"）
  5. 输出最终报告，未修复项标记 [skipped]
其余检查项只报告不修改文件。
```

### 4.4 `samsara status`

遍历 `knowledge/` 目录，聚合统计后格式化输出（见产品文档 §18.2）。不修改任何文件。

### 4.5 `samsara log [--tail N]`

读取 `log.md`，按行解析，`--tail N`（默认 20）控制显示条数。支持 `--action write/promote/lint` 过滤。

### 4.6 `samsara archive <domain> <keyword>`

```
Algorithm:
  1. find_lesson → 找不到则报错
  2. 移动文件：lessons/<domain>/<keyword>.md → archive/<domain>/<keyword>.md
  3. index::rebuild()
  4. log ARCHIVE
  5. auto_commit("samsara: archive <domain>/<keyword>")
```

### 4.7 `samsara init`

```
Algorithm:
  1. 创建目录结构（若已存在则跳过，不报错）：
     $AGENTS_HOME/knowledge/{lessons/,rules/,archive/}
     种子 domain 目录（37 个，来源：§14）：
     $AGENTS_HOME/knowledge/lessons/{rust/,python/,typescript/,javascript/,go/,java/,kotlin/,swift/,cpp/,c/,dart/,flutter/,android/,ios/,git/,ci/,docker/,k8s/,infra/,makefile/,cmake/,cargo/,windows/,linux/,macos/,api/,database/,auth/,testing/,perf/,security/,ml/,samsara/,skm/,opencode/,vscode/,terminal/}
     $AGENTS_HOME/skills/self-evolution/
     $AGENTS_HOME/adapters/{claude-code/,gemini/,windsurf/}
   2. 创建/更新文件（幂等，upsert 策略）：
      $AGENTS_HOME/AGENTS.md       → 不存在：写入协议模板（含 ## AAAK 占位 section）
                                     已存在：仅在末尾追加缺失的 ## AAAK section，不修改现有内容
      $AGENTS_HOME/knowledge/INDEX.md → 不存在则创建；已存在则跳过
      $AGENTS_HOME/knowledge/log.md   → 不存在则创建；已存在则跳过
   3. git 初始化：
      若 $AGENTS_HOME/knowledge/ 不是 git repo → git init
      写入 .gitignore（upsert：检查并追加缺失行，不覆盖现有内容）
      写入 .gitattributes（upsert：检查并追加缺失属性行，不覆盖现有内容）：
         knowledge/log.md merge=union      ← 多设备 merge：log.md 条目自动合并（union）
         knowledge/INDEX.md merge=ours     ← 多设备 merge：INDEX.md 保留本地版本（pull 后重建）
      注意：若已是 git repo，跳过 git init
  4. 工具映射（所有操作仅在目标不存在时执行，已存在则打印 ⚠️）：
     A 类 symlink（目标工具配置目录存在时）：
       ln -sf $AGENTS_HOME/AGENTS.md ~/.config/opencode/AGENTS.md
       ln -sf $AGENTS_HOME/AGENTS.md ~/.codex/AGENTS.md
     B 类 @import（~/.claude 存在时）：
       if ~/.claude/CLAUDE.md 不存在：写入 "@$AGENTS_HOME/AGENTS.md"
       else：打印 ⚠️ 提示用户手动在 CLAUDE.md 中添加 @import 行
     C 类（Gemini/Windsurf）：打印 ⏭️ 待 P-03 确认后实现
  5. 若 skm 在 PATH 中：
     skm install mocikadev/self-evolution --link-to all（若未安装）
  6. 输出初始化报告（每步 ✅/⚠️/⏭️ 状态）
```

**注意**：init 本身不触发 git commit（空仓库），首次 write 时才产生第一个 commit。

### 4.8 `samsara reflect`

```
Algorithm（纯静态分析，无 LLM 调用）：

  1. 解析 log.md，构建事件序列
  2. 计算各 domain 的 UPDATE 频率（最近 30 天）
     → 高频 domain（> 5 次）→ 建议安装 skill
  3. 扫描 lessons/ 目录，找 occurrences.len() >= 3 且 promoted=false
     → 按 lesson_type 分组（error / skill / pattern / insight / 未分类）输出待晋升候选列表
     → 每条附带 verified 次数
  4. 统计 SKILL_FAIL 事件，按 skill 名称聚合失败次数
     → 失败次数 > 1 → 标记为需修复
  5. 扫描 log.md 中所有 target 字段（keyword 部分），统计出现频率
     → 高频 keyword（> 3 次）且 AGENTS.md ## AAAK section 中无对应 entity → 输出 AAAK 候选条目
  6. 格式化输出报告（见产品文档 §18.2 reflect 示例）
```

reflect 不修改任何文件，只输出报告。

### 4.9 `samsara skill-note <name>`

```
Algorithm:
  1. 根据是否有 --fail flag 确定 action: SkillUse or SkillFail
  2. 构建 LogEntry { action, target: name, note: --note 参数 }
  3. log::append_log(log_path, &entry)
  4. auto_commit("samsara: skill-note <name>")
  5. 输出：✅ skill 使用记录已写入 / ⚠️ skill 失败已记录
```

---

### 4.10 `samsara search <query>`

```
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
     [路径]  标签... 出现次数（lesson 的话）
       匹配行预览（最多 2 行，高亮匹配词）

flags:
  --domain <name>    限制搜索范围
  --type <type>      仅搜索指定类型（error|skill|pattern|insight）
  --rules-only       仅搜索 rules/
  --lessons-only     仅搜索 lessons/
  --limit N          最多显示 N 个结果（默认 10）
```

不修改任何文件。

### 4.11 `samsara domain list` / `samsara domain add <name>`

```
domain list:
  遍历 lessons/ 子目录，格式化输出（domain 名、lesson 数量、是否有 rules 文件）
  不修改任何文件

domain add <name>:
  1. 验证 name 不含非法字符（/、空格等）
  2. 检查 lessons/<name>/ 是否已存在 → 存在则打印已有并退出
  3. mkdir lessons/<name>/
  4. 输出：✅ domain '<name>' 已注册（lessons/<name>/）
```

### 4.12 `samsara log rotate [--keep 90d]`

```
Algorithm:
  1. 解析 log.md 全部条目（跳过非法行）
  2. cutoff_date = today - keep_days（默认 90）
  3. 分割：
     recent = entries where date >= cutoff_date
     old    = entries where date < cutoff_date
  4. 若 old 为空 → 打印 "无需轮转" 并退出
  5. 按年份分组 old，追加到 log.archive-YYYY.md（append-only）
  6. 将 recent 写回 log.md（覆盖）
  7. auto_commit("samsara: log rotate")
   8. 输出：归档 N 条到 log.archive-YYYY.md，保留 M 条
```

### 4.13 `samsara push`

```
Algorithm:
  1. git -C $AGENTS_HOME/knowledge add -A
  2. git -C $AGENTS_HOME/knowledge commit -m "samsara: sync $(date +%Y-%m-%d)" --allow-empty
  3. git -C $AGENTS_HOME/knowledge push origin main
  4. 输出：✅ 推送成功 / ❌ 推送失败（打印 git 错误）
```

### 4.14 `samsara pull`

```
Algorithm:
  1. git -C $AGENTS_HOME/knowledge fetch origin
  2. git -C $AGENTS_HOME/knowledge merge --no-ff origin/main
     冲突处理策略（由 .gitattributes 驱动）：
       lesson/*.md / rules/*.md   → last-push-wins（git 默认 merge 策略）
       INDEX.md                   → merge=ours（保留本地版本）
       log.md                     → merge=union（自动合并，不产生冲突）
  3. 若 merge 退出码 != 0（lesson/rules 产生真实冲突）：
       打印冲突文件列表，提示用户手动解决冲突（git add + git merge --continue）后重新运行 samsara pull
  4. 若 merge 成功 → index::rebuild()（重建 INDEX.md，覆盖 merge=ours 保留的旧版本）
   5. 输出：✅ 拉取并重建 INDEX 成功 / ⚠️ 合并冲突，需手动解决
```

### 4.15 `samsara prime [--limit N] [--sort <recent|occurrences|domain>] [--domain <d>]`

```
Algorithm（只读，不修改任何文件）：
  1. 收集所有 rules/*.md 的规则条目（按 "## 规则标题" 解析）
  2. 收集 occurrences >= 3 且 promoted=true 的 lessons 的核心规则行
  3. 对每条规则计算"推荐分"：
     - occurrences 总数 × 10
     - 最近 occurrence 距今天数 d → score += max(0, 30 - d) × 5（越近越高）
     - lesson_type == Error → +20（错误教训优先晋升到 AGENTS.md）
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
```

flags:
- `--limit N` — 最多显示 N 条（默认 10）
- `--sort recent` — 按最近 occurrence 排序（默认）
- `--sort occurrences` — 按总次数排序
- `--sort domain` — 按 domain 分组输出
- `--domain <d>` — 仅输出指定 domain 的规则

### 4.16 `samsara demote <pattern>`

```
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
```

flags:
- `--yes` — 跳过逐条确认
- `--dry-run` — 预览匹配行，不执行删除

### 4.17 `samsara remote add <url>` / `samsara remote set <url>` / `samsara remote show`

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

---

## 5. 错误处理

使用 `thiserror` crate 定义统一错误类型：

```rust
#[derive(thiserror::Error, Debug)]
pub enum SamsaraError {
    #[error("domain '{0}' 不存在，使用 --yes 自动创建或 `samsara domain add '{0}'`")]
    UnknownDomain(String),

    #[error("lesson '{0}/{1}' 不存在")]
    LessonNotFound(String, String),

    #[error("lesson '{0}' occurrences 不足 3 次（当前 {1} 次），无法晋升")]
    InsufficientOccurrences(String, usize),

    #[error("frontmatter 解析失败：{0}")]
    FrontmatterParse(#[from] serde_yaml::Error),

    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),

    #[error("AGENTS.md 不存在：{0}，请先运行 `samsara init`")]
    AgentsMdNotFound(std::path::PathBuf),
}
```

所有命令返回 `Result<(), SamsaraError>`，main.rs 统一格式化输出错误信息后 `process::exit(1)`。

---

## 6. 测试策略

### 6.1 单元测试（`#[cfg(test)]` 内嵌）

| 测试目标 | 测试内容 |
|---------|---------|
| `lesson.rs` | frontmatter 序列化/反序列化，`add_occurrence`，`should_promote`，`is_expired` |
| `index.rs` | `rebuild` 生成的 INDEX.md 与实际 lessons/ 目录一致；`scan` 返回正确统计 |
| `log.rs` | append 格式正确，read_last_n 边界 |

### 6.2 集成测试（`tests/integration/`）

每个测试用例：
1. 创建临时目录（`tempdir` crate）
2. 复制 `tests/fixtures/` 中的预置文件
3. 调用命令函数（`commands::write::run(config, args)`）
4. 断言文件内容、INDEX 变化、log 条目

```
fixtures/
├── empty_knowledge/      ← 空知识库（测试 init + 首次写入）
├── existing_lesson/      ← occurrences: 2（测试 update + 晋升提示）
├── promotable/           ← occurrences: 3, promoted: false（测试 promote --aaak → AGENTS.md section）
├── expired/              ← valid_until 已过期（测试 lint ③）
├── skill_notes/          ← SKILL_USE/SKILL_FAIL 日志（测试 reflect skill 分析）
├── high_freq/            ← 高频 UPDATE domain + 多个待晋升（测试 reflect 完整输出）
├── new_domain/           ← write 时 domain 不存在（测试 --yes 自动创建 + 交互拒绝）
├── search_mixed/         ← 多文件多域（测试 search 相关性排序）
└── stale_rules_ref/      ← rules 文件引用已归档的 lesson（测试 lint ⑦ 引用失效检查）
```

### 6.3 不测试的内容

- 编辑器交互（`$EDITOR` 调起）：集成测试通过 `--summary` flag 绕过
- 用户交互确认（promote 的"是否绝不级别"）：单元测试直接传参绕过

---

## 7. 构建与发布

### 依赖

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
dirs = "5"           # 获取 ~ 路径

# 注意：git 操作通过 std::process::Command 调用 git binary，
# 不引入 git2 crate，保持零重依赖原则（来源：daerwen §3.7.3）

[dev-dependencies]
tempfile = "3"       # 集成测试临时目录
```

### 安装方式

```bash
# 方式 1：cargo 直接安装
cargo install --path .

# 方式 2（规划中）：通过 skm 安装
skm install samsara-cli
```

### 发布路线

| Phase | 内容 | 验收标准 |
|-------|------|---------|
| v0.1 | `init`、`write`、`search`、`status`、`log` | init 完成目录+git+symlink+工具映射（upsert 幂等）；write 支持 --update/--yes；search 按相关性返回结果 |
| v0.2 | `lint`、`promote`（含 --aaak、--layer0）、`reflect`、`skill-note`、`domain` | lint 覆盖 13 项检查（含引用失效、promoted 状态一致性、蒸馏候选⑬）；promote --layer0 安全算法完整；domain list/add 可用 |
| v0.3 | `archive`、`prime`、`demote`、`--dry-run`、`log rotate` | archive --stale 批量归档；prime 输出含可执行 promote 命令；demote 无匹配时安全退出 |
| v0.4 | `remote`、`push`、`pull` | 多设备同步完整；冲突策略（last-push-wins / union / ours）通过 .gitattributes 验证；pull 冲突时引导人工解决 |

---

*文档结束。实现时以本文档为准，产品行为以 `samsara-design.md §18` 为准。*
