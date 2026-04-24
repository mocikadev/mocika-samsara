use chrono::Local;
use std::{
    collections::HashSet,
    fs,
    io::{self, BufRead, Write},
};

use crate::{
    cli::DemoteArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::log::{self, LogAction, LogEntry},
};

pub fn run(args: DemoteArgs, config: &Config) -> Result<(), SamsaraError> {
    let agents_md_path = config.agents_home.join("AGENTS.md");
    if !agents_md_path.exists() {
        return Err(SamsaraError::AgentsMdNotFound(agents_md_path));
    }

    let content = fs::read_to_string(config.agents_home.join("AGENTS.md"))?;
    let had_trailing_newline = content.ends_with('\n');
    let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
    let matches = find_matching_lines(&lines, &args.pattern);

    if matches.is_empty() {
        println!("未找到可降级规则，请检查 pattern");
        return Ok(());
    }

    for (line_number, line) in &matches {
        println!("L{line_number}: {line}");
    }

    if config.dry_run {
        println!("[dry-run] 将从 AGENTS.md 删除以上规则");
        return Ok(());
    }

    if !args.yes && !confirm_demote()? {
        println!("已取消");
        return Ok(());
    }

    let matched_line_numbers: HashSet<usize> = matches
        .iter()
        .map(|(line_number, _)| *line_number)
        .collect();
    let remaining_lines: Vec<String> = lines
        .into_iter()
        .enumerate()
        .filter_map(|(index, line)| (!matched_line_numbers.contains(&(index + 1))).then_some(line))
        .collect();

    fs::write(
        &agents_md_path,
        render_lines(&remaining_lines, had_trailing_newline),
    )?;

    println!("规则已从 AGENTS.md 移除。规则应存在于 rules/<domain>.md，请运行 samsara lint 验证");

    let entry = LogEntry {
        date: Local::now().date_naive(),
        action: LogAction::Demote,
        target: args.pattern.clone(),
        note: None,
    };
    log::append_log(&config.knowledge_home.join("log.md"), &entry, false)?;

    if config.auto_commit {
        git::auto_commit(
            &config.knowledge_home,
            "samsara: demote rule from AGENTS.md",
        )?;
    }

    Ok(())
}

fn find_matching_lines(lines: &[String], pattern: &str) -> Vec<(usize, String)> {
    let pattern = pattern.to_lowercase();
    let mut in_protected_section = false;
    let mut matches = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("## 自我进化协议") || trimmed.starts_with("## AAAK") {
            in_protected_section = true;
            continue;
        }

        if trimmed.starts_with("## ") {
            in_protected_section = false;
            continue;
        }

        if in_protected_section {
            continue;
        }

        if line.to_lowercase().contains(&pattern) {
            matches.push((index + 1, line.clone()));
        }
    }

    matches
}

fn confirm_demote() -> Result<bool, SamsaraError> {
    print!("确认删除以上规则？[y/N]");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn render_lines(lines: &[String], had_trailing_newline: bool) -> String {
    let mut rendered = lines.join("\n");
    if had_trailing_newline && !rendered.is_empty() {
        rendered.push('\n');
    }
    rendered
}
