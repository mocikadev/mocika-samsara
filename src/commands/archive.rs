use chrono::Local;
use std::fs;

use crate::{
    cli::ArchiveArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::{
        index,
        lesson::find_lesson,
        log::{self, LogAction, LogEntry},
    },
};

pub fn run(args: ArchiveArgs, config: &Config) -> Result<(), SamsaraError> {
    let ArchiveArgs { domain, keyword } = args;

    let lesson_path = find_lesson(&config.knowledge_home, &domain, &keyword)
        .ok_or_else(|| SamsaraError::LessonNotFound(domain.clone(), keyword.clone()))?;

    if config.dry_run {
        println!("[dry-run] 将归档 lessons/{domain}/{keyword}.md → archive/{domain}/{keyword}.md");
        return Ok(());
    }

    let archive_dir = config.knowledge_home.join("archive").join(&domain);
    fs::create_dir_all(&archive_dir)?;

    let archive_path = archive_dir.join(format!("{keyword}.md"));
    fs::rename(&lesson_path, &archive_path)?;

    index::rebuild(&config.knowledge_home, false)?;

    let entry = LogEntry {
        date: Local::now().date_naive(),
        action: LogAction::Archive,
        target: format!("{domain}/{keyword}"),
        note: None,
    };
    log::append_log(&config.knowledge_home.join("log.md"), &entry, false)?;

    if config.auto_commit {
        git::auto_commit(
            &config.knowledge_home,
            &format!("samsara: archive {domain}/{keyword}"),
        )?;
    }

    println!("✅ {domain}/{keyword} 已归档到 archive/{domain}/{keyword}.md");
    Ok(())
}
