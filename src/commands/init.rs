use crate::{cli::InitArgs, config::Config, error::SamsaraError};
use std::path::Path;
use std::process::Command;
use serde_json::Value;

const SEED_DOMAINS: &[&str] = &[
    "rust",
    "python",
    "typescript",
    "javascript",
    "go",
    "java",
    "kotlin",
    "swift",
    "cpp",
    "c",
    "dart",
    "flutter",
    "android",
    "ios",
    "git",
    "ci",
    "docker",
    "k8s",
    "infra",
    "makefile",
    "cmake",
    "cargo",
    "windows",
    "linux",
    "macos",
    "api",
    "database",
    "auth",
    "testing",
    "perf",
    "security",
    "ml",
    "samsara",
    "skm",
    "opencode",
    "vscode",
    "terminal",
];

const AGENTS_MD_TEMPLATE: &str = r#"# 自我进化协议（Samsara）

> 知识库路径：`~/.agents/knowledge/`

## 操作协议

遇到可归纳的错误/教训时：
```bash
samsara write <domain> <keyword> --summary "..." [--type error|skill|pattern|insight]
```

再次遇到同一问题：`samsara write <domain> <keyword>`（occurrences +1）

occurrences ≥ 3 时：`samsara promote <domain> <keyword>`（晋升到 rules/<domain>.md）

## 常用命令

| 命令 | 用途 |
|------|------|
| `samsara write <d> <k> --summary "..."` | 写入 / 更新教训 |
| `samsara write <d> <k> --verify` | 验证规则有效（verified +1）|
| `samsara promote <d> <k>` | 晋升到 rules/<domain>.md |
| `samsara promote <d> <k> --layer0` | 晋升到 AGENTS.md（100 行安全检查）|
| `samsara lint [--fix]` | 检查 / 修复知识库 |
| `samsara prime` | Top 10 推荐晋升规则 |
| `samsara search <query>` | 搜索 lesson/rules |
| `samsara status` | 知识库统计 |
| `samsara reflect` | 分析学习模式 |

## AAAK
<!-- AAAK 条目（entity|relation|value|date），由 samsara 自动维护，请勿手动编辑 -->
"#;

const PROTOCOL_INJECT: &str = r#"

---

## 自我进化协议（Samsara）

> 知识库路径：`~/.agents/knowledge/`

遇到可归纳的错误/教训时：

```bash
samsara write <domain> <keyword> --summary "..." [--type error|skill|pattern|insight]
```

再次遇到同一问题：`samsara write <domain> <keyword>`（occurrences +1）
occurrences ≥ 3：`samsara promote <domain> <keyword>`

| 命令 | 用途 |
|------|------|
| `samsara write <d> <k> --summary "..."` | 写入 / 更新教训 |
| `samsara write <d> <k> --verify` | 验证规则有效 |
| `samsara promote <d> <k>` | 晋升到 rules/<domain>.md |
| `samsara promote <d> <k> --layer0` | 晋升到 AGENTS.md |
| `samsara search <query>` | 搜索教训 / 规则 |
| `samsara prime` | Top 10 推荐晋升规则 |
"#;

const GITIGNORE_LINES: &[&str] = &[".DS_Store", "*.tmp", "*.bak"];

const GITATTRIBUTES_LINES: &[&str] = &[
    "knowledge/log.md merge=union",
    "knowledge/INDEX.md merge=ours",
];

pub fn run(args: InitArgs, config: &Config) -> Result<(), SamsaraError> {
    let agents_home = &config.agents_home;
    let knowledge_home = &config.knowledge_home;

    println!("🔄 初始化知识库：{}", agents_home.display());

    ensure_dirs(agents_home, knowledge_home, args.yes)?;
    upsert_agents_md(agents_home, config.dry_run)?;
    create_if_absent(&knowledge_home.join("INDEX.md"), "", config.dry_run)?;
    create_if_absent(&knowledge_home.join("log.md"), "", config.dry_run)?;
    init_git(knowledge_home, config.dry_run)?;
    setup_tool_mappings(agents_home, config.dry_run)?;
    maybe_install_skill(args.yes)?;

    println!("✅ 初始化完成");
    Ok(())
}

fn ensure_dirs(agents_home: &Path, knowledge_home: &Path, _yes: bool) -> Result<(), SamsaraError> {
    for sub in &["lessons", "rules", "archive"] {
        std::fs::create_dir_all(knowledge_home.join(sub))?;
    }
    for domain in SEED_DOMAINS {
        std::fs::create_dir_all(knowledge_home.join("lessons").join(domain))?;
    }
    for adapter in &["claude-code", "gemini", "windsurf"] {
        std::fs::create_dir_all(agents_home.join("adapters").join(adapter))?;
    }
    println!("  ✅ 目录结构创建完毕");
    Ok(())
}

fn upsert_agents_md(agents_home: &Path, dry_run: bool) -> Result<(), SamsaraError> {
    let path = agents_home.join("AGENTS.md");
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        if !content.contains("## AAAK") {
            if !dry_run {
                let appended = format!("{content}\n## AAAK\n<!-- AAAK 条目（entity|relation|value|date），由 samsara 自动维护，请勿手动编辑 -->\n");
                std::fs::write(&path, appended)?;
            }
            println!("  ✅ AGENTS.md：追加了缺失的 ## AAAK section");
        } else {
            println!("  ⚠️  AGENTS.md 已存在，内容未修改");
        }
    } else {
        if !dry_run {
            std::fs::write(&path, AGENTS_MD_TEMPLATE)?;
        }
        println!("  ✅ AGENTS.md：写入协议模板");
    }
    Ok(())
}

fn create_if_absent(path: &Path, content: &str, dry_run: bool) -> Result<(), SamsaraError> {
    if !path.exists() {
        if !dry_run {
            std::fs::write(path, content)?;
        }
        println!("  ✅ 创建：{}", path.display());
    }
    Ok(())
}

fn init_git(knowledge_home: &Path, dry_run: bool) -> Result<(), SamsaraError> {
    if crate::git::is_git_repo(knowledge_home) {
        println!("  ⚠️  knowledge/ 已是 git 仓库，跳过 git init");
    } else {
        if !dry_run {
            let ok = Command::new("git")
                .args(["init", &knowledge_home.to_string_lossy()])
                .status()
                .map_err(|e| SamsaraError::GitNotFound(e.to_string()))?
                .success();
            if !ok {
                return Err(SamsaraError::GitFailed);
            }
        }
        println!("  ✅ git init knowledge/");
    }

    upsert_file_lines(&knowledge_home.join(".gitignore"), GITIGNORE_LINES, dry_run)?;
    upsert_file_lines(
        &knowledge_home.join(".gitattributes"),
        GITATTRIBUTES_LINES,
        dry_run,
    )?;

    Ok(())
}

fn upsert_file_lines(path: &Path, lines: &[&str], dry_run: bool) -> Result<(), SamsaraError> {
    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut content = existing.clone();
    let mut added = false;
    for line in lines {
        if !existing.lines().any(|l| l.trim() == *line) {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(line);
            content.push('\n');
            added = true;
        }
    }
    if added && !dry_run {
        std::fs::write(path, &content)?;
    }
    Ok(())
}

fn setup_tool_mappings(agents_home: &Path, dry_run: bool) -> Result<(), SamsaraError> {
    let home = dirs::home_dir().unwrap_or_default();

    let opencode_dir = home.join(".config").join("opencode");
    if opencode_dir.exists() {
        inject_protocol(&opencode_dir.join("AGENTS.md"), dry_run)?;
    }

    let codex_dir = home.join(".codex");
    if codex_dir.exists() {
        inject_protocol(&codex_dir.join("AGENTS.md"), dry_run)?;
    }

    let claude_dir = home.join(".claude");
    if claude_dir.exists() {
        let claude_md = claude_dir.join("CLAUDE.md");
        let agents_md = agents_home.join("AGENTS.md");
        let import_line = format!("@{}", agents_md.display());
        if !claude_md.exists() {
            if !dry_run {
                std::fs::write(&claude_md, format!("{import_line}\n"))?;
            }
            println!("  ✅ ~/.claude/CLAUDE.md：写入 @import 行");
        } else {
            let content = std::fs::read_to_string(&claude_md)?;
            if !content.contains(&import_line) {
                if !dry_run {
                    let appended = format!("{}\n{import_line}\n", content.trim_end());
                    std::fs::write(&claude_md, appended)?;
                }
                println!("  ✅ ~/.claude/CLAUDE.md：追加 @import 行");
            } else {
                println!("  ⚠️  ~/.claude/CLAUDE.md 已包含 @import，跳过");
            }
        }
    }

    println!("  ⏭️  Gemini / Windsurf 映射：待 P-03 确认后实现");
    inject_mcp_configs(dry_run)?;
    Ok(())
}

fn inject_protocol(path: &Path, dry_run: bool) -> Result<(), SamsaraError> {
    const MARKER: &str = "## 自我进化协议（Samsara）";
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        if content.contains(MARKER) {
            println!("  ⚠️  自进化协议已存在，跳过：{}", path.display());
            return Ok(());
        }
        if !dry_run {
            let appended = format!("{}\n{}", content.trim_end(), PROTOCOL_INJECT);
            std::fs::write(path, appended)?;
        }
        println!("  ✅ 注入自进化协议：{}", path.display());
    } else {
        if !dry_run {
            std::fs::write(path, PROTOCOL_INJECT.trim_start())?;
        }
        println!("  ✅ 创建并写入自进化协议：{}", path.display());
    }
    Ok(())
}

fn maybe_install_skill(_yes: bool) -> Result<(), SamsaraError> {
    let skm_available = Command::new("skm").arg("--version").status().is_ok();
    if skm_available {
        println!("  🔍 检测到 skm，安装 self-evolution skill...");
        let status = Command::new("skm")
            .args([
                "install",
                "mocikadev/mocika-samsara:skills/self-evolution",
                "--link-to",
                "all",
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("  ✅ self-evolution skill 已安装并链接到所有 agent");
                println!("     后续升级：skm update self-evolution");
            }
            Ok(_) => println!("  ⚠️  skm install 返回非零退出码，请手动执行："),
            Err(e) => println!("  ⚠️  skm 调用失败（{e}），请手动执行："),
        }
    } else {
        println!("  ⚠️  未检测到 skm，跳过 skill 安装");
        println!("     安装 skm 后手动执行：skm install mocikadev/mocika-samsara:skills/self-evolution --link-to all");
    }
    Ok(())
}

fn inject_mcp_configs(dry_run: bool) -> Result<(), SamsaraError> {
    let home = dirs::home_dir().unwrap_or_default();

    let opencode_dir = home.join(".config").join("opencode");
    if opencode_dir.exists() {
        let entry = serde_json::json!({
            "type": "local",
            "command": ["samsara", "mcp", "serve"]
        });
        inject_json_mcp_entry(&opencode_dir.join("opencode.json"), "mcp", entry, dry_run)?;
    }

    let claude_dir = home.join(".claude");
    if claude_dir.exists() {
        let entry = serde_json::json!({
            "command": "samsara",
            "args": ["mcp", "serve"]
        });
        inject_json_mcp_entry(
            &claude_dir.join("claude_desktop_config.json"),
            "mcpServers",
            entry,
            dry_run,
        )?;
    }

    Ok(())
}

fn inject_json_mcp_entry(
    path: &Path,
    section_key: &str,
    entry: Value,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    let mut root: Value = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => {
                println!("  ⚠️  {} JSON 解析失败，跳过 MCP 配置注入", path.display());
                return Ok(());
            }
        }
    } else {
        Value::Object(serde_json::Map::new())
    };

    let root_obj = match root.as_object_mut() {
        Some(obj) => obj,
        None => {
            println!("  ⚠️  {} 格式非对象，跳过 MCP 配置注入", path.display());
            return Ok(());
        }
    };

    let section = root_obj
        .entry(section_key)
        .or_insert_with(|| Value::Object(serde_json::Map::new()));

    let section_obj = match section.as_object_mut() {
        Some(obj) => obj,
        None => {
            println!(
                "  ⚠️  {} 中 {} 格式非对象，跳过",
                path.display(),
                section_key
            );
            return Ok(());
        }
    };

    if section_obj.contains_key("samsara") {
        println!("  ⚠️  {} 已含 samsara MCP 配置，跳过", path.display());
        return Ok(());
    }

    section_obj.insert("samsara".to_string(), entry);

    if !dry_run {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let rendered = serde_json::to_string_pretty(&root)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(path, rendered + "\n")?;
    }

    println!("  ✅ MCP 配置已注入：{}", path.display());
    Ok(())
}
