use crate::{
    cli::WriteArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::{
        index,
        lesson::{find_lesson, Lesson, LessonFrontmatter, LessonType},
        log::{self, LogAction, LogEntry},
    },
};
use chrono::Local;
use std::{
    io::{self, Write},
    path::Path,
    process::Command,
};

pub fn run(args: WriteArgs, config: &Config) -> Result<(), SamsaraError> {
    let WriteArgs {
        domain,
        keyword,
        summary,
        r#type,
        verify,
        update,
        yes,
    } = args;

    let today = Local::now().date_naive();
    let lessons_dir = config.knowledge_home.join("lessons");

    ensure_domain_exists(&lessons_dir, &domain, yes, config.dry_run)?;

    let lesson_type = r#type.as_deref().and_then(parse_lesson_type);

    let (lesson, action) = match find_lesson(&config.knowledge_home, &domain, &keyword) {
        Some(path) => {
            let mut lesson = Lesson::load(&path)?;
            lesson.add_occurrence(today);

            if verify {
                lesson.frontmatter.verified += 1;
            }

            if let Some(ref lesson_type) = lesson_type {
                lesson.frontmatter.lesson_type = Some(lesson_type.clone());
            }

            match (update, summary.as_deref()) {
                (true, Some(summary)) => lesson.body = summary.to_string(),
                (true, None) => open_editor(&mut lesson, config.dry_run)?,
                (false, _) => {}
            }

            lesson.save(config.dry_run)?;
            (lesson, LogAction::Update)
        }
        None => {
            let mut lesson = Lesson {
                frontmatter: LessonFrontmatter {
                    date: today,
                    domain: domain.clone(),
                    lesson_type,
                    tags: Vec::new(),
                    occurrences: vec![today],
                    promoted: false,
                    verified: 0,
                    valid_until: None,
                    conflicts_with: None,
                },
                body: String::new(),
                path: lessons_dir.join(&domain).join(format!("{keyword}.md")),
            };

            if let Some(summary) = summary {
                lesson.body = summary;
            } else {
                open_editor(&mut lesson, config.dry_run)?;
            }

            lesson.save(config.dry_run)?;
            (lesson, LogAction::Write)
        }
    };

    let entry = LogEntry {
        date: today,
        action,
        target: format!("{domain}/{keyword}"),
        note: verify.then(|| format!("verified={}", lesson.frontmatter.verified)),
    };

    log::append_log(
        &config.knowledge_home.join("log.md"),
        &entry,
        config.dry_run,
    )?;
    index::rebuild(&config.knowledge_home, config.dry_run)?;

    if config.auto_commit && !config.dry_run {
        git::auto_commit(
            &config.knowledge_home,
            &format!("samsara: write {domain}/{keyword}"),
        )?;
    }

    if lesson.should_promote() {
        println!(
            "⚡ occurrences 已达 3 次，可执行 samsara promote {} {}",
            domain, keyword
        );
    }

    Ok(())
}

fn ensure_domain_exists(
    lessons_dir: &Path,
    domain: &str,
    yes: bool,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    let domain_dir = lessons_dir.join(domain);
    if domain_dir.exists() {
        return Ok(());
    }

    if yes || confirm_domain_creation(domain)? {
        if !dry_run {
            std::fs::create_dir_all(&domain_dir)?;
        }
        return Ok(());
    }

    print_existing_domains(lessons_dir)?;
    Err(SamsaraError::UnknownDomain(domain.to_string()))
}

fn confirm_domain_creation(domain: &str) -> Result<bool, SamsaraError> {
    print!("⚠️  domain '{}' 不存在，是否创建？ [y/N]: ", domain);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn print_existing_domains(lessons_dir: &Path) -> Result<(), SamsaraError> {
    if !lessons_dir.exists() {
        println!("可用 domain：<无>");
        return Ok(());
    }

    let mut domains = Vec::new();
    for entry in std::fs::read_dir(lessons_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            domains.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    domains.sort_unstable();

    if domains.is_empty() {
        println!("可用 domain：<无>");
    } else {
        println!("可用 domain：{}", domains.join(", "));
    }

    Ok(())
}

fn parse_lesson_type(raw: &str) -> Option<LessonType> {
    match raw.to_ascii_lowercase().as_str() {
        "error" => Some(LessonType::Error),
        "skill" => Some(LessonType::Skill),
        "pattern" => Some(LessonType::Pattern),
        "insight" => Some(LessonType::Insight),
        _ => {
            eprintln!("⚠️  无效的 lesson type: {raw}，已忽略。");
            None
        }
    }
}

fn open_editor(lesson: &mut Lesson, dry_run: bool) -> Result<(), SamsaraError> {
    if dry_run {
        return Ok(());
    }

    lesson.save(false)?;

    let editor = std::env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("VISUAL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "vi".to_string());

    match Command::new(&editor).arg(&lesson.path).status() {
        Ok(status) if status.success() => {
            *lesson = Lesson::load(&lesson.path)?;
        }
        Ok(_) => {
            eprintln!("⚠️  编辑器退出码非零，继续保留当前内容。");
        }
        Err(error) => {
            eprintln!("⚠️  启动编辑器失败（{editor}）：{error}");
        }
    }

    Ok(())
}
