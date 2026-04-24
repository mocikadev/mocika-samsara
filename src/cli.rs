use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "samsara",
    about = "AI Agent 知识管理 CLI — 写入、晋升、校验和反思学习教训",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// 覆盖知识库路径（默认 ~/.agents/knowledge/）
    #[arg(long, global = true)]
    pub home: Option<std::path::PathBuf>,

    /// 预览操作，不修改任何文件
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// 初始化知识库目录和工具映射
    Init(InitArgs),
    /// 写入或更新一条教训
    Write(WriteArgs),
    /// 按相关性搜索 lessons 和 rules
    Search(SearchArgs),
    /// 将教训晋升为规则（或晋升到 AGENTS.md）
    Promote(PromoteArgs),
    /// 管理 domain（列出 / 添加）
    Domain(DomainArgs),
    /// 将教训归档
    Archive(ArchiveArgs),
    /// 检查知识库健康度（13 项）
    Lint(LintArgs),
    /// 显示知识库统计摘要
    Status(StatusArgs),
    /// 查看操作日志
    Log(LogArgs),
    /// 按推荐分输出 Top N 规则候选
    Prime(PrimeArgs),
    /// 将规则从 AGENTS.md 降级
    Demote(DemoteArgs),
    /// 管理 git 远端地址
    Remote(RemoteArgs),
    /// 分析学习模式和待晋升候选（静态分析，无 LLM）
    Reflect(ReflectArgs),
    /// 记录 skill 使用成功或失败
    #[command(name = "skill-note")]
    SkillNote(SkillNoteArgs),
    /// 推送 knowledge/ 到远端 git
    Push,
    /// 从远端 git 拉取并重建 INDEX
    Pull,
    /// 升级 samsara 到最新版本
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateArgs),
    /// 启动 MCP 服务
    Mcp(McpArgs),
}

// ─── 各命令参数结构体 ─────────────────────────────────────────

#[derive(Parser)]
pub struct InitArgs {
    /// 跳过交互确认
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Parser)]
pub struct WriteArgs {
    /// 知识域（如 rust、git、docker）
    pub domain: String,
    /// 关键词（文件名，无 .md 后缀）
    pub keyword: String,
    /// 直接写入正文摘要（跳过编辑器）
    #[arg(long)]
    pub summary: Option<String>,
    /// 记录类型（error | skill | pattern | insight）
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,
    /// verified 次数 +1（规则已验证有效）
    #[arg(long)]
    pub verify: bool,
    /// 更新正文（打开 $EDITOR 或配合 --summary）
    #[arg(long)]
    pub update: bool,
    /// domain 不存在时自动创建，跳过交互提示
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Parser)]
pub struct SearchArgs {
    /// 搜索关键词
    pub query: String,
    /// 限制搜索范围到指定 domain
    #[arg(long)]
    pub domain: Option<String>,
    /// 仅搜索指定类型（error | skill | pattern | insight）
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,
    /// 仅搜索 rules/
    #[arg(long)]
    pub rules_only: bool,
    /// 仅搜索 lessons/
    #[arg(long)]
    pub lessons_only: bool,
    /// 最多显示 N 个结果（默认 10）
    #[arg(long, default_value = "10")]
    pub limit: usize,
}

#[derive(Parser)]
pub struct PromoteArgs {
    /// 知识域
    pub domain: String,
    /// 关键词
    pub keyword: String,
    /// 同时写入 AGENTS.md 的 ## AAAK section
    #[arg(long)]
    pub aaak: bool,
    /// 晋升到 AGENTS.md 实质规则区（有 100 行安全检查）
    #[arg(long)]
    pub layer0: bool,
    /// 跳过交互确认
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Parser)]
pub struct DomainArgs {
    #[command(subcommand)]
    pub action: DomainAction,
}

#[derive(Subcommand)]
pub enum DomainAction {
    /// 列出所有 domain
    List,
    /// 添加新 domain
    Add {
        /// domain 名称
        name: String,
    },
}

#[derive(Parser)]
pub struct ArchiveArgs {
    /// 知识域
    pub domain: String,
    /// 关键词
    pub keyword: String,
}

#[derive(Parser)]
pub struct LintArgs {
    /// 自动修复可修复项（⑤⑥⑧⑪）
    #[arg(long)]
    pub fix: bool,
    /// 批量确认修复，跳过交互
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Parser)]
pub struct StatusArgs {}

#[derive(Parser)]
pub struct LogArgs {
    /// 显示最后 N 条（默认 20）
    #[arg(long, default_value = "20")]
    pub tail: usize,
    /// 按操作类型过滤（write | promote | lint | archive）
    #[arg(long)]
    pub action: Option<String>,
    /// 轮转旧日志到归档文件
    #[arg(long)]
    pub rotate: bool,
    /// 保留最近 N 天（与 --rotate 配合，默认 90）
    #[arg(long, value_name = "DAYS", default_value = "90")]
    pub keep: u32,
}

#[derive(Parser)]
pub struct PrimeArgs {
    /// 最多显示 N 条（默认 10）
    #[arg(long, default_value = "10")]
    pub limit: usize,
    /// 排序方式（recent | occurrences | domain，默认 recent）
    #[arg(long, default_value = "recent")]
    pub sort: String,
    /// 仅输出指定 domain
    #[arg(long)]
    pub domain: Option<String>,
}

#[derive(Parser)]
pub struct DemoteArgs {
    /// 匹配模式（不区分大小写，支持部分词匹配）
    pub pattern: String,
    /// 跳过逐条确认
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Parser)]
pub struct RemoteArgs {
    #[command(subcommand)]
    pub action: RemoteAction,
}

#[derive(Subcommand)]
pub enum RemoteAction {
    /// 添加远端地址
    Add {
        /// git 远端 URL
        url: String,
    },
    /// 更新远端地址
    Set {
        /// git 远端 URL
        url: String,
    },
    /// 显示当前远端配置
    Show,
}

#[derive(Parser)]
pub struct ReflectArgs {}

#[derive(Parser)]
pub struct SkillNoteArgs {
    /// skill 名称
    pub name: String,
    /// 记录为失败（默认记录为成功）
    #[arg(long)]
    pub fail: bool,
    /// 附加备注
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Parser)]
pub struct SelfUpdateArgs {
    /// 仅检查新版本，不下载
    #[arg(long)]
    pub check: bool,
}

#[derive(Parser)]
pub struct McpArgs {
    #[command(subcommand)]
    pub action: McpAction,
}

#[derive(Subcommand)]
pub enum McpAction {
    /// 以 stdio 模式启动 MCP 服务
    Serve {
        /// 预留参数：未来可扩展为 HTTP 模式
        #[arg(long)]
        port: Option<u16>,
    },
}
