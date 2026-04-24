use crate::{
    cli::LogArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::log::{self, LogAction},
};

pub fn run(args: LogArgs, config: &Config) -> Result<(), SamsaraError> {
    let log_path = config.knowledge_home.join("log.md");

    if args.rotate {
        let result = log::rotate_log(&log_path, args.keep, config.dry_run)?;
        if result.archived == 0 {
            println!("无需轮转（{} 天内共 {} 条）", args.keep, result.kept);
        } else {
            println!("已归档 {} 条，保留 {} 条", result.archived, result.kept);
            if config.auto_commit && !config.dry_run {
                git::auto_commit(&config.knowledge_home, "samsara: log rotate")?;
            }
        }
        return Ok(());
    }

    let entries = log::read_last_n(&log_path, args.tail)?;
    let action_filter = args
        .action
        .as_deref()
        .and_then(|action| log::action_from_str(&action.to_ascii_uppercase()));

    let filtered: Vec<_> = entries
        .into_iter()
        .rev()
        .filter(|entry| match &action_filter {
            Some(action) => same_action(action, &entry.action),
            None => true,
        })
        .collect();

    if filtered.is_empty() {
        println!("暂无操作记录");
        return Ok(());
    }

    for entry in filtered {
        println!(
            "{} {:<12} {}",
            entry.date,
            action_label(&entry.action),
            entry.target
        );
    }

    Ok(())
}

fn same_action(left: &LogAction, right: &LogAction) -> bool {
    matches!(
        (left, right),
        (LogAction::Write, LogAction::Write)
            | (LogAction::Update, LogAction::Update)
            | (LogAction::Promote, LogAction::Promote)
            | (LogAction::Archive, LogAction::Archive)
            | (LogAction::Lint, LogAction::Lint)
            | (LogAction::SkillUse, LogAction::SkillUse)
            | (LogAction::SkillFail, LogAction::SkillFail)
            | (LogAction::Demote, LogAction::Demote)
            | (LogAction::Layer0, LogAction::Layer0)
            | (LogAction::LogRotate, LogAction::LogRotate)
    )
}

fn action_label(action: &LogAction) -> &'static str {
    match action {
        LogAction::Write => "WRITE",
        LogAction::Update => "UPDATE",
        LogAction::Promote => "PROMOTE",
        LogAction::Archive => "ARCHIVE",
        LogAction::Lint => "LINT",
        LogAction::SkillUse => "SKILL_USE",
        LogAction::SkillFail => "SKILL_FAIL",
        LogAction::Demote => "DEMOTE",
        LogAction::Layer0 => "LAYER0",
        LogAction::LogRotate => "LOG_ROTATE",
    }
}
