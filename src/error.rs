#[derive(thiserror::Error, Debug)]
pub enum SamsaraError {
    #[error("domain '{0}' 不存在，使用 --yes 自动创建或 `samsara domain add '{0}'`")]
    UnknownDomain(String),

    #[error("lesson '{0}/{1}' 不存在")]
    LessonNotFound(String, String),

    #[error("lesson '{0}' occurrences 不足 3 次（当前 {1} 次），无法晋升")]
    InsufficientOccurrences(String, usize),

    #[error("frontmatter 解析失败：{0}")]
    FrontmatterParse(String),

    #[error("YAML 解析失败：{0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),

    #[error("AGENTS.md 不存在：{0}，请先运行 `samsara init`")]
    AgentsMdNotFound(std::path::PathBuf),

    #[error("AGENTS.md 实质规则已有 {0} 行，超过 100 行上限，请先运行 `samsara demote` 降级规则")]
    AgentsMdTooLong(usize),

    #[error("git 未安装或不在 PATH 中：{0}")]
    GitNotFound(String),

    #[error("git 操作失败（退出码非零）")]
    GitFailed,

    #[error("git remote 操作失败：{0}")]
    RemoteFailed(String),

    #[error("推送失败：{0}")]
    PushFailed(String),

    #[error("merge 冲突，请手动解决后再运行 `samsara pull`：{0:?}")]
    PullConflict(Vec<String>),

    #[error("自升级失败：{0}")]
    UpdateError(String),

    #[error("网络请求失败：{0}")]
    NetworkError(String),
}

impl From<reqwest::Error> for SamsaraError {
    fn from(value: reqwest::Error) -> Self {
        Self::NetworkError(value.to_string())
    }
}
