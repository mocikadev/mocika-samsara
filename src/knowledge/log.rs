use crate::error::SamsaraError;
use chrono::{Datelike, Duration, Local, NaiveDate};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

pub enum LogAction {
    Write,
    Update,
    Promote,
    Archive,
    Lint,
    SkillUse,
    SkillFail,
    Demote,
    Layer0,
    LogRotate,
}

pub struct LogEntry {
    pub date: NaiveDate,
    pub action: LogAction,
    pub target: String,
    pub note: Option<String>,
}

pub struct RotateResult {
    pub archived: usize,
    pub kept: usize,
}

pub fn append_log(log_path: &Path, entry: &LogEntry, dry_run: bool) -> Result<(), SamsaraError> {
    if dry_run {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(f, "{}", format_entry(entry))?;
    Ok(())
}

pub fn read_last_n(log_path: &Path, n: usize) -> Result<Vec<LogEntry>, SamsaraError> {
    if !log_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(log_path)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(n);
    let entries = lines[start..]
        .iter()
        .filter_map(|l| parse_log_line(l))
        .collect();
    Ok(entries)
}

pub fn read_all_entries(log_path: &Path) -> Result<Vec<LogEntry>, SamsaraError> {
    if !log_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(log_path)?;
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_log_line)
        .collect())
}

pub fn rotate_log(
    log_path: &Path,
    keep_days: u32,
    dry_run: bool,
) -> Result<RotateResult, SamsaraError> {
    let entries = read_all_entries(log_path)?;
    if entries.is_empty() {
        return Ok(RotateResult {
            archived: 0,
            kept: 0,
        });
    }

    let today = Local::now().date_naive();
    let cutoff = today - Duration::days(i64::from(keep_days));
    let mut recent = Vec::new();
    let mut old = Vec::new();

    for entry in entries {
        if entry.date < cutoff {
            old.push(entry);
        } else {
            recent.push(entry);
        }
    }

    if old.is_empty() {
        return Ok(RotateResult {
            archived: 0,
            kept: recent.len(),
        });
    }

    let archived = old.len();

    if !dry_run {
        let archive_root = archive_root(log_path);
        let mut old_by_year = std::collections::BTreeMap::<i32, Vec<LogEntry>>::new();
        for entry in old {
            old_by_year
                .entry(entry.date.year())
                .or_default()
                .push(entry);
        }

        for (year, entries) in old_by_year {
            let archive_path = archive_root.join(format!("log.archive-{year}.md"));
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(archive_path)?;

            for entry in entries {
                writeln!(file, "{}", format_entry(&entry))?;
            }
        }

        std::fs::write(log_path, render_entries(&recent))?;
    }

    Ok(RotateResult {
        archived,
        kept: recent.len(),
    })
}

pub fn count_lines(log_path: &Path) -> usize {
    if !log_path.exists() {
        return 0;
    }
    std::fs::read_to_string(log_path)
        .map(|c| c.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0)
}

fn action_str(action: &LogAction) -> &'static str {
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

pub fn action_from_str(s: &str) -> Option<LogAction> {
    match s.trim() {
        "WRITE" => Some(LogAction::Write),
        "UPDATE" => Some(LogAction::Update),
        "PROMOTE" => Some(LogAction::Promote),
        "ARCHIVE" => Some(LogAction::Archive),
        "LINT" => Some(LogAction::Lint),
        "SKILL_USE" => Some(LogAction::SkillUse),
        "SKILL_FAIL" => Some(LogAction::SkillFail),
        "DEMOTE" => Some(LogAction::Demote),
        "LAYER0" => Some(LogAction::Layer0),
        "LOG_ROTATE" => Some(LogAction::LogRotate),
        _ => None,
    }
}

fn format_entry(entry: &LogEntry) -> String {
    let a = action_str(&entry.action);
    match &entry.note {
        Some(note) => format!("{} {:<12} {} ({})", entry.date, a, entry.target, note),
        None => format!("{} {:<12} {}", entry.date, a, entry.target),
    }
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let mut parts = line.splitn(3, ' ');
    let date_str = parts.next()?;
    let action_raw = parts.next()?.trim();
    let rest = parts.next()?.trim();

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let action = action_from_str(action_raw)?;

    let (target, note) = if let Some(note_start) = rest.rfind(" (") {
        if let Some(note_end) = rest.rfind(')') {
            let target = rest[..note_start].trim().to_string();
            let note = rest[note_start + 2..note_end].to_string();
            (target, Some(note))
        } else {
            (rest.to_string(), None)
        }
    } else {
        (rest.to_string(), None)
    };

    Some(LogEntry {
        date,
        action,
        target,
        note,
    })
}

fn archive_root(log_path: &Path) -> PathBuf {
    log_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn render_entries(entries: &[LogEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut content = entries
        .iter()
        .map(format_entry)
        .collect::<Vec<_>>()
        .join("\n");
    content.push('\n');
    content
}
