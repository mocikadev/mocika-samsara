use chrono::Local;

use crate::{
    cli::SkillNoteArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::log::{self, LogAction, LogEntry},
};

pub fn run(args: SkillNoteArgs, config: &Config) -> Result<(), SamsaraError> {
    let action = if args.fail {
        LogAction::SkillFail
    } else {
        LogAction::SkillUse
    };

    let entry = LogEntry {
        date: Local::now().date_naive(),
        action,
        target: args.name.clone(),
        note: args.note.clone(),
    };

    log::append_log(
        &config.knowledge_home.join("log.md"),
        &entry,
        config.dry_run,
    )?;

    if config.auto_commit && !config.dry_run {
        git::auto_commit(
            &config.knowledge_home,
            &format!("samsara: skill-note {}", args.name),
        )?;
    }

    if args.fail {
        println!("⚠️  skill 失败已记录：{}", args.name);
    } else {
        println!("✅ skill 使用记录已写入：{}", args.name);
    }

    Ok(())
}
