use crate::{
    cli::PromoteArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::{
        aaak::{self, AaakEntry},
        index,
        lesson::{find_lesson, Lesson},
        log::{append_log, LogAction, LogEntry},
        rules,
    },
};
use chrono::Local;
use std::{
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

pub fn run(args: PromoteArgs, config: &Config) -> Result<(), SamsaraError> {
    let PromoteArgs {
        domain,
        keyword,
        aaak,
        layer0,
        yes,
    } = args;

    let lesson_path = find_lesson(&config.knowledge_home, &domain, &keyword)
        .ok_or_else(|| SamsaraError::LessonNotFound(domain.clone(), keyword.clone()))?;
    let lesson = Lesson::load(&lesson_path)?;

    if layer0 {
        return promote_layer0(lesson, &domain, &keyword, aaak, yes, config);
    }

    promote_to_rules(lesson, &domain, &keyword, aaak, config)
}

fn promote_to_rules(
    mut lesson: Lesson,
    domain: &str,
    keyword: &str,
    include_aaak: bool,
    config: &Config,
) -> Result<(), SamsaraError> {
    let occurrences = lesson.frontmatter.occurrences.len();
    if occurrences < 3 {
        return Err(SamsaraError::InsufficientOccurrences(
            format!("{domain}/{keyword}"),
            occurrences,
        ));
    }

    rules::append_to_rules(
        &config.knowledge_home,
        domain,
        keyword,
        &lesson.body,
        occurrences,
        config.dry_run,
    )?;

    lesson.frontmatter.promoted = true;
    lesson.save(config.dry_run)?;

    let today = Local::now().date_naive();
    append_log(
        &config.knowledge_home.join("log.md"),
        &LogEntry {
            date: today,
            action: LogAction::Promote,
            target: format!("{domain}/{keyword}"),
            note: Some(format!("→ rules/{domain}.md")),
        },
        config.dry_run,
    )?;

    index::rebuild(&config.knowledge_home, config.dry_run)?;

    if config.auto_commit && !config.dry_run {
        git::auto_commit(
            &config.knowledge_home,
            &format!("samsara: promote {domain}/{keyword}"),
        )?;
    }

    if include_aaak {
        handle_aaak_prompt(
            &config.agents_home.join("AGENTS.md"),
            &lesson.body,
            config.dry_run,
        )?;
    }

    println!("✅ {domain}/{keyword} 已晋升到 rules/{domain}.md");
    println!("如需晋升到 AGENTS.md，运行 samsara promote {domain} {keyword} --layer0");

    Ok(())
}

fn promote_layer0(
    lesson: Lesson,
    domain: &str,
    keyword: &str,
    include_aaak: bool,
    yes: bool,
    config: &Config,
) -> Result<(), SamsaraError> {
    let agents_md_path = config.agents_home.join("AGENTS.md");
    if !agents_md_path.exists() {
        return Err(SamsaraError::AgentsMdNotFound(agents_md_path));
    }

    let rule_line = format!(
        "- {domain}/{keyword}: {}",
        first_non_empty_line(&lesson.body)
    );
    println!("将在 AGENTS.md 写入：\n{rule_line}");

    if config.dry_run {
        return Ok(());
    }

    if !yes && !confirm_layer0_write()? {
        return Ok(());
    }

    let backup_path = backup_agents_md(&config.agents_home, &agents_md_path)?;
    let content = fs::read_to_string(&agents_md_path)?;
    let had_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(ToString::to_string).collect();

    let current_rule_lines = count_substantive_rule_lines(&lines);
    let added_rule_lines = rule_line.lines().count();
    let total_rule_lines = current_rule_lines + added_rule_lines;

    if total_rule_lines > 100 {
        println!(
            "拒绝写入：AGENTS.md 实质规则已有 {current_rule_lines} 行，新增后将超过 100 行上限。"
        );
        fs::remove_file(&backup_path)?;
        return Err(SamsaraError::AgentsMdTooLong(current_rule_lines));
    }

    insert_layer0_rule(&mut lines, rule_line);
    fs::write(&agents_md_path, render_lines(&lines, had_trailing_newline))?;

    let today = Local::now().date_naive();
    append_log(
        &config.knowledge_home.join("log.md"),
        &LogEntry {
            date: today,
            action: LogAction::Promote,
            target: format!("{domain}/{keyword}"),
            note: Some("→ AGENTS.md --layer0".to_string()),
        },
        false,
    )?;

    if include_aaak {
        handle_aaak_prompt(&agents_md_path, &lesson.body, false)?;
    }

    if config.auto_commit {
        git::auto_commit(
            &config.knowledge_home,
            &format!("samsara: promote --layer0 {domain}/{keyword}"),
        )?;
    }

    fs::remove_file(&backup_path)?;
    println!("✅ {domain}/{keyword} 已晋升到 AGENTS.md");
    Ok(())
}

fn handle_aaak_prompt(
    agents_md_path: &Path,
    rule_body: &str,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    if !agents_md_path.exists() {
        return Err(SamsaraError::AgentsMdNotFound(agents_md_path.to_path_buf()));
    }

    println!("规则内容：\n{}", rule_body.trim());
    println!("请输入 AAAK（entity/relation/value），留空跳过：");

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(());
    }

    let parts: Vec<&str> = trimmed.split('/').map(str::trim).collect();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        eprintln!("⚠️  AAAK 输入格式无效，已跳过。请使用 entity/relation/value。");
        return Ok(());
    }

    let entry = AaakEntry {
        entity: parts[0].to_string(),
        relation: parts[1].to_string(),
        value: parts[2].to_string(),
        date: Local::now().date_naive(),
    };

    aaak::append_entry(agents_md_path, &entry, dry_run)?;
    if !dry_run {
        let _ = aaak::load_and_trim(agents_md_path, 480)?;
    }

    Ok(())
}

fn confirm_layer0_write() -> Result<bool, SamsaraError> {
    print!("确认写入？[y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn backup_agents_md(agents_home: &Path, agents_md_path: &Path) -> Result<PathBuf, SamsaraError> {
    let backup_dir = agents_home.join(".backup");
    fs::create_dir_all(&backup_dir)?;
    let backup_path = backup_dir.join("AGENTS.md.bak");
    fs::copy(agents_md_path, &backup_path)?;
    Ok(backup_path)
}

fn first_non_empty_line(body: &str) -> &str {
    body.lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("")
}

fn insert_layer0_rule(lines: &mut Vec<String>, rule_line: String) {
    if let Some((section_start, _)) = find_aaak_bounds(lines) {
        if section_start > 0 && !lines[section_start - 1].is_empty() {
            lines.insert(section_start, String::new());
        }
        lines.insert(section_start, rule_line);
        return;
    }

    if !lines.is_empty() && lines.last().is_some_and(|line| !line.is_empty()) {
        lines.push(String::new());
    }
    lines.push(rule_line);
}

fn count_substantive_rule_lines(lines: &[String]) -> usize {
    let aaak_bounds = find_aaak_bounds(lines);

    lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            let in_aaak = aaak_bounds.is_some_and(|(start, end)| *index >= start && *index < end);

            !in_aaak && !line.trim().is_empty() && !line.trim_start().starts_with('#')
        })
        .count()
}

fn find_aaak_bounds(lines: &[String]) -> Option<(usize, usize)> {
    let start = lines
        .iter()
        .position(|line| line.trim_start().starts_with("## AAAK"))?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| line.trim_start().starts_with("##").then_some(index))
        .unwrap_or(lines.len());

    Some((start, end))
}

fn render_lines(lines: &[String], had_trailing_newline: bool) -> String {
    let mut rendered = lines.join("\n");
    if had_trailing_newline && !rendered.is_empty() {
        rendered.push('\n');
    }
    rendered
}
