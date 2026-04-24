# samsara · 工程开发参考

> **文档版本**：v1.0  
> **状态**：阶段 3（工程准备）起效，随迭代持续更新  
> **关联**：`samsara-cli-impl.md` v0.4，参考实现 `mocika-skills-cli`

---

## 目录

1. [技术选型](#1-技术选型)
2. [项目骨架结构](#2-项目骨架结构)
3. [CI/CD 配置](#3-cicd-配置)
4. [self-evolution SKILL.md 设计](#4-self-evolution-skillmd-设计)
5. [自升级策略](#5-自升级策略)
6. [关键实现模式](#6-关键实现模式)
7. [开发规范](#7-开发规范)

---

## 1. 技术选型

### 1.1 依赖清单

与 `skm`（mocika-skills-cli）保持技术栈对齐，精简为 samsara 最小需求：

```toml
[package]
name = "samsara"
version = "0.1.0"
edition = "2021"
rust-version = "1.88"
license = "MIT OR Apache-2.0"

[[bin]]
name = "samsara"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive", "color"] }
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"          # frontmatter 解析（YAML 1.1，与 lesson 格式兼容）
toml = "0.8"                # samsara.toml 配置文件
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"                  # ~ 路径解析
regex = "1"                 # lint ⑬ Jaccard 计算 / search 高亮

[dev-dependencies]
tempfile = "3"              # 集成测试临时目录
assert_cmd = "2"            # CLI 命令测试
predicates = "3"            # 断言辅助

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev.package."*"]
opt-level = 3               # 依赖编译加速，不影响调试

[profile.release-with-debug]
inherits = "release"
strip = false
debug = true
```

### 1.2 刻意不引入的依赖

| 依赖 | 理由 |
|------|------|
| `anyhow` | samsara 命令层统一用 `SamsaraError`，无需多层包装；避免混用两套错误处理模式 |
| `git2` | 通过 `std::process::Command` 调用系统 git，零编译依赖；与 skm 一致（impl §3.1） |
| `reqwest` | v0.1-v0.3 无 HTTP 需求；`self-update` 命令在 v0.4+ 引入时再加 |
| `rayon` | lesson 数量（百级）不足以触发并行收益；搜索是 I/O bound |
| `indicatif` / `console` | 初期命令无长耗时操作；`init` 阶段 3 骨架中先用简单 eprintln |
| `sha2` | 同 reqwest，延到自升级命令实现时再引入 |

### 1.3 Rust 版本策略

- **MSRV**：`1.88`（与 skm 对齐，CI 强制校验）
- **edition**：2021
- 使用 `OnceLock`（stable since 1.70）、`let-else`（stable since 1.65）等稳定特性

---

## 2. 项目骨架结构

完整目录布局（来源：impl §1，参考 skm 模块分层）：

```
mocika-samsara/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── src/
│   ├── main.rs              ← ~9 行：调用 lib::run()，格式化顶层错误后 exit(1)
│   ├── lib.rs               ← run()：i18n 初始化 + clap 路由
│   ├── cli.rs               ← 所有子命令 enum（clap derive）
│   ├── config.rs            ← Config 结构体，SAMSARA_HOME / --home flag 解析
│   ├── error.rs             ← SamsaraError（thiserror）
│   ├── i18n.rs              ← Lang enum + OnceLock，直接复用 skm 模式
│   ├── knowledge/
│   │   ├── mod.rs
│   │   ├── lesson.rs        ← LessonFrontmatter, Lesson 数据模型 + 解析/写回
│   │   ├── rules.rs         ← rules/<domain>.md 读写
│   │   ├── index.rs         ← INDEX.md 全量重建（rebuild / scan）
│   │   ├── log.rs           ← log.md 追加 / read_last_n
│   │   └── aaak.rs          ← AGENTS.md ## AAAK section 读写
│   └── commands/
│       ├── mod.rs
│       ├── init.rs          ← samsara init
│       ├── write.rs         ← samsara write
│       ├── search.rs        ← samsara search
│       ├── promote.rs       ← samsara promote（含 --aaak / --layer0）
│       ├── domain.rs        ← samsara domain list|add
│       ├── archive.rs       ← samsara archive
│       ├── lint.rs          ← samsara lint（13 项检查，--fix）
│       ├── status.rs        ← samsara status
│       ├── log.rs           ← samsara log + log rotate
│       ├── prime.rs         ← samsara prime
│       ├── demote.rs        ← samsara demote
│       ├── remote.rs        ← samsara remote add|set|show
│       ├── reflect.rs       ← samsara reflect
│       └── skill_note.rs    ← samsara skill-note
├── tests/
│   ├── fixtures/
│   │   ├── empty_knowledge/
│   │   ├── existing_lesson/
│   │   ├── promotable/
│   │   ├── expired/
│   │   ├── skill_notes/
│   │   ├── high_freq/
│   │   ├── new_domain/
│   │   ├── search_mixed/
│   │   └── stale_rules_ref/
│   └── integration/
├── docs/
│   ├── samsara-design.md       ← 产品设计 v0.8（已冻结）
│   ├── samsara-cli-impl.md     ← 实现规格 v0.4（已冻结）
│   ├── samsara-engineering.md  ← 本文档
│   └── research/layer-c/       ← Layer C 专项调研归档
├── skills/
│   └── self-evolution/
│       └── SKILL.md            ← AI agent 操作手册（见 §4）
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

### 2.1 main.rs / lib.rs 模式

直接参考 skm 的分层：`main.rs` 极薄（~9 行），所有逻辑在 `lib.rs` 的 `run()` 中：

```rust
// main.rs
fn main() {
    if let Err(e) = samsara::run() {
        eprintln!("{}: {e}", samsara::i18n::t("error"));
        std::process::exit(1);
    }
}

// lib.rs
pub fn run() -> Result<(), SamsaraError> {
    i18n::init_from_env();
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args)      => commands::init::run(args),
        Command::Write(args)     => commands::write::run(args),
        Command::Search(args)    => commands::search::run(args),
        Command::Promote(args)   => commands::promote::run(args),
        Command::Domain(args)    => commands::domain::run(args),
        Command::Archive(args)   => commands::archive::run(args),
        Command::Lint(args)      => commands::lint::run(args),
        Command::Status(args)    => commands::status::run(args),
        Command::Log(args)       => commands::log::run(args),
        Command::Prime(args)     => commands::prime::run(args),
        Command::Demote(args)    => commands::demote::run(args),
        Command::Remote(args)    => commands::remote::run(args),
        Command::Reflect(args)   => commands::reflect::run(args),
        Command::SkillNote(args) => commands::skill_note::run(args),
        Command::Push            => commands::log::run_push(),
        Command::Pull            => commands::log::run_pull(),
        Command::SelfUpdate(a)   => commands::self_update::run(a),
    }
}
```

---

## 3. CI/CD 配置

### 3.1 CI（`.github/workflows/ci.yml`）

直接对齐 skm 的 ci.yml 结构：

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true

jobs:
  check:
    name: Check (fmt + clippy + test)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test

  msrv:
    name: MSRV (1.88)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.88"
      - uses: Swatinem/rust-cache@v2
        with:
          key: msrv
      - run: cargo check

  build:
    name: Build · ${{ matrix.target }}
    needs: check
    strategy:
      matrix:
        include:
          - { target: x86_64-unknown-linux-musl,  runner: ubuntu-latest, use-cross: true }
          - { target: aarch64-unknown-linux-musl,  runner: ubuntu-latest, use-cross: true }
          - { target: x86_64-apple-darwin,         runner: macos-latest,  use-cross: false }
          - { target: aarch64-apple-darwin,         runner: macos-latest,  use-cross: false }
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}
      - name: Install cross
        if: matrix.use-cross
        uses: taiki-e/install-action@v2
        with:
          tool: cross
      - name: Build
        run: |
          if [ "${{ matrix.use-cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
```

### 3.2 Release（`.github/workflows/release.yml`）

触发：`push tags v*`

步骤：
1. **verify**：检查 tag 与 `Cargo.toml version` 是否一致（`grep '^version' Cargo.toml`）
2. **build**：4 个 target 矩阵，产出 `samsara-{artifact}` 二进制
3. **package**：打包 `samsara-linux-amd64.tar.gz` 等
4. **checksum**：生成 `SHA256SUMS.txt`
5. **release**：`gh release create $TAG` 上传所有 artifacts

---

## 4. self-evolution SKILL.md 设计

### 4.1 定位与交付方式

`skills/self-evolution/SKILL.md` 是 samsara CLI 的配套 AI 操作手册，随工程仓库一起维护。

发布后通过 skm 安装：
```bash
skm install mocikadev/mocika-samsara --subpath skills/self-evolution
```

`samsara init` 也会自动通过 `skm install` 拉取（见 impl §4.7 step 5）。

### 4.2 frontmatter 规范

```yaml
---
name: self-evolution
version: 0.1.0
description: Guide AI agents to use samsara CLI for knowledge management
tags: [samsara, knowledge, lesson, rules, agents-md]
compatible_agents: [opencode, claude-code, codex, gemini]
---
```

版本号与 samsara CLI `Cargo.toml version` 保持一致（每次命令接口变更同步更新）。

### 4.3 内容结构骨架

```markdown
# self-evolution · AI 自我进化操作手册

## 何时使用本 skill
- 遇到可归纳的错误或教训时
- 学到新技能/模式/洞察后想持久化时
- 需要将高频规则晋升到 AGENTS.md 时
- 需要记录 skill 使用结果（成功 / 失败）时

## 基本工作流

### 1. 写入教训
```bash
samsara write <domain> <keyword> --summary "简明教训" [--type error|skill|pattern|insight]
```

### 2. 验证规则有效性
```bash
samsara write <domain> <keyword> --verify   # verified 字段 +1
```

### 3. 晋升规则
```bash
samsara promote <domain> <keyword>           # → rules/<domain>.md
samsara promote <domain> <keyword> --layer0  # → AGENTS.md（有 100 行安全检查）
```

### 4. 检查知识库健康度
```bash
samsara lint          # 13 项检查
samsara lint --fix    # 自动修复 ⑤⑥⑧⑪ 四项
```

### 5. 提炼 Top N 推荐规则
```bash
samsara prime [--limit 10] [--domain rust]
```

## 命令速查

| 命令 | 场景 |
|------|------|
| `samsara write <d> <k> --summary "..."` | 写入 / 更新教训（无需打开编辑器） |
| `samsara write <d> <k> --verify` | 验证规则有效（verified +1） |
| `samsara promote <d> <k>` | 晋升到 rules/<domain>.md |
| `samsara promote <d> <k> --layer0` | 晋升到 AGENTS.md（需确认） |
| `samsara lint [--fix]` | 检查 / 修复知识库 |
| `samsara prime` | Top 10 推荐晋升规则 |
| `samsara reflect` | 分析学习模式和待晋升候选 |
| `samsara search <query>` | 按相关性搜索 lesson/rules |
| `samsara skill-note <name>` | 记录 skill 使用成功 |
| `samsara skill-note <name> --fail --note "..."` | 记录 skill 失败 |
| `samsara status` | 知识库统计 |

## 注意事项

- AI 读取 lesson 文件**不计入** occurrences，不重置 90 天计时器
- `write` 有 upsert 语义：keyword 已存在则追加 occurrence，不覆盖正文
- `promote --layer0` 有 100 行安全检查，超出时拒绝并引导 `samsara demote`
- `lint --fix` 仅自动修复 ⑤⑥⑧⑪ 四项，其余需人工处理
- type 字段可选（error / skill / pattern / insight），影响 prime 评分（Error +20）
```

---

## 5. 自升级策略

> 计划在 v0.4+ 引入（阶段 4 后期），阶段 3 骨架中保留 `Command::SelfUpdate` 占位即可。

参考 skm 的 `core/updater.rs`，完整流程：

1. **检查**：`GET https://api.github.com/repos/mocikadev/mocika-samsara/releases/latest`，解析 `tag_name`
2. **比较**：semver 比较 `env!("CARGO_PKG_VERSION")` 与 latest tag
3. **下载**：按 `(OS, ARCH)` 映射 asset 名，同时下载 `SHA256SUMS.txt`
   - `x86_64-linux` → `samsara-linux-amd64.tar.gz`
   - `aarch64-linux` → `samsara-linux-arm64.tar.gz`
   - `x86_64-macos` → `samsara-macos-amd64.tar.gz`
   - `aarch64-macos` → `samsara-macos-arm64.tar.gz`
4. **校验**：`sha2` 计算下载文件的 SHA256，与 `SHA256SUMS.txt` 比对
5. **替换**：写入 `{exe_path}.tmp`，`std::fs::rename` 原子替换
6. **命令**：`samsara self-update [--check]`

引入时机依赖：
```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
sha2 = "0.10"
```

安全约束（继承 skm 经验）：
- 使用 `rustls-tls`，不依赖系统 OpenSSL
- Windows 暂不支持（可执行文件锁定），`cfg!(target_os = "windows")` 早期返回并提示手动下载

---

## 6. 关键实现模式

### 6.1 原子写文件

```rust
/// 写入临时文件后 rename，确保原子性（电源中断不留损坏文件）
fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

### 6.2 frontmatter 解析

```rust
/// 解析 ---\n...\n---\n 格式，返回 (frontmatter, body)
fn parse_frontmatter(content: &str) -> Result<(LessonFrontmatter, String), SamsaraError> {
    let rest = content
        .strip_prefix("---\n")
        .ok_or_else(|| SamsaraError::FrontmatterParse("missing opening ---".into()))?;
    let end = rest
        .find("\n---\n")
        .ok_or_else(|| SamsaraError::FrontmatterParse("missing closing ---".into()))?;
    let yaml = &rest[..end];
    let body = rest[end + 5..].to_string();  // skip "\n---\n"
    let fm: LessonFrontmatter = serde_yaml::from_str(yaml)?;
    Ok((fm, body))
}
```

### 6.3 i18n（直接复用 skm 模式）

```rust
// i18n.rs
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang { En, Zh }

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn init_from_env() {
    // samsara 默认中文（面向中文用户）；可通过 LANG=en 切换
    let lang = std::env::var("LANG")
        .ok()
        .and_then(|v| Lang::from_code(v.split('.').next().unwrap_or("")))
        .unwrap_or(Lang::Zh);
    let _ = LANG.set(lang);
}

pub fn t(key: &str) -> &'static str {
    match *LANG.get().unwrap_or(&Lang::Zh) {
        Lang::En => match key { "error" => "error", _ => key },
        Lang::Zh => match key { "error" => "错误", _ => key },
    }
}
```

### 6.4 git 封装

```rust
// 不引入 git2，通过 Command 调用系统 git（零编译依赖）
fn run_git(args: &[&str], cwd: &Path) -> Result<(), SamsaraError> {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| SamsaraError::GitNotFound(e.to_string()))?;
    if !status.success() {
        anyhow::bail!("git {:?} exited with: {}", args, status);
    }
    Ok(())
}

pub fn auto_commit(knowledge_home: &Path, message: &str) -> Result<(), SamsaraError> {
    // 若不是 git repo 则静默跳过（不报错），提示用户运行 samsara init
    if !knowledge_home.join(".git").exists() {
        eprintln!("提示：knowledge/ 未初始化 git，运行 `samsara init` 启用自动提交");
        return Ok(());
    }
    run_git(&["add", "-A"], knowledge_home)?;
    run_git(&["commit", "-m", message], knowledge_home)?;
    Ok(())
}
```

---

## 7. 开发规范

### 7.1 提交格式

```
<type>: <中文描述>
```

type 限于：`feat / fix / docs / style / refactor / perf / test / build / ci / chore`

示例：
- `feat: 实现 samsara write 命令（upsert 语义）`
- `test: 补充 lint ⑬ Jaccard 检查集成测试`
- `ci: 初始化 GitHub Actions（fmt + clippy + test）`

### 7.2 提交前必执行

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

### 7.3 阶段 3 验收清单

按顺序执行，每步可独立验证：

```bash
# 1. 初始化项目骨架
cargo init --name samsara

# 2. 编写 cli.rs（所有 17 个子命令定义）
# 3. 编写 commands/ 骨架（每个命令 fn run(...) { unimplemented!() }）

# 4. 验证编译与静态检查
cargo build                      # 必须 0 error
cargo clippy -- -D warnings      # 必须 0 warning
cargo fmt --check                # 必须通过

# 5. 验证 help 输出
./target/debug/samsara --help    # 必须输出所有 17 个子命令

# 6. 验证 CI 配置
# 推送后 GitHub Actions 全绿（check + msrv + build 三个 job）
```

### 7.4 阶段 4 里程碑分工

| 版本 | 命令 |
|------|------|
| v0.1 | `init`, `write`, `search`, `status`, `log` |
| v0.2 | `lint`, `promote`（含 --aaak / --layer0）, `reflect`, `skill-note`, `domain` |
| v0.3 | `archive`, `prime`, `demote`, `--dry-run`, `log rotate` |
| v0.4 | `remote`, `push`, `pull`, `self-update` |

每个版本独立满足：`cargo test` 全绿 + `cargo clippy -- -D warnings` 无告警 + 手动走通一次真实场景。

---

*文档结束。阶段 3 完成后更新 §2 骨架目录（补充实际创建的文件列表）。*
