use crate::error::SamsaraError;
use chrono::NaiveDate;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AaakEntry {
    pub entity: String,
    pub relation: String,
    pub value: String,
    pub date: chrono::NaiveDate,
}

impl AaakEntry {
    pub fn to_line(&self) -> String {
        format!(
            "[{}|{}|{}|{}]",
            self.entity, self.relation, self.value, self.date
        )
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
        let parts: Vec<&str> = inner.split('|').map(str::trim).collect();

        if parts.len() != 4 {
            return None;
        }

        let date = NaiveDate::parse_from_str(parts[3], "%Y-%m-%d").ok()?;

        Some(Self {
            entity: parts[0].to_string(),
            relation: parts[1].to_string(),
            value: parts[2].to_string(),
            date,
        })
    }
}

pub fn find_aaak_section(
    agents_md_path: &Path,
) -> Result<Option<(usize, Vec<AaakEntry>)>, SamsaraError> {
    let content = std::fs::read_to_string(agents_md_path)?;
    Ok(find_aaak_section_in_content(&content))
}

fn find_aaak_section_in_content(content: &str) -> Option<(usize, Vec<AaakEntry>)> {
    let lines = collect_lines(content);

    find_section_bounds(&lines).map(|(section_start, section_end)| {
        let entries = lines[section_start + 1..section_end]
            .iter()
            .filter_map(|line| AaakEntry::from_line(line))
            .collect();

        (section_start, entries)
    })
}

pub fn append_entry(
    agents_md_path: &Path,
    entry: &AaakEntry,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    let content = std::fs::read_to_string(agents_md_path)?;
    let had_trailing_newline = content.ends_with('\n');
    let mut lines = collect_lines(&content);

    if let Some((section_start, section_end)) = find_section_bounds(&lines) {
        let mut replaced = false;

        for line in &mut lines[section_start + 1..section_end] {
            if let Some(existing) = AaakEntry::from_line(line) {
                if existing.entity == entry.entity && existing.relation == entry.relation {
                    *line = entry.to_line();
                    replaced = true;
                    break;
                }
            }
        }

        if !replaced {
            lines.insert(section_end, entry.to_line());
        }
    } else {
        if !lines.is_empty() && lines.last().is_some_and(|line| !line.is_empty()) {
            lines.push(String::new());
        }
        lines.push("## AAAK".to_string());
        lines.push(entry.to_line());
    }

    if dry_run {
        println!("将在 AGENTS.md 的 ## AAAK 写入：\n{}", entry.to_line());
        return Ok(());
    }

    std::fs::write(agents_md_path, render_lines(&lines, had_trailing_newline))?;
    Ok(())
}

pub fn load_and_trim(
    agents_md_path: &Path,
    budget_chars: usize,
) -> Result<Vec<AaakEntry>, SamsaraError> {
    let content = std::fs::read_to_string(agents_md_path)?;
    let had_trailing_newline = content.ends_with('\n');
    let mut lines = collect_lines(&content);

    let Some((section_start, section_end)) = find_section_bounds(&lines) else {
        return Ok(vec![]);
    };

    let entry_lines: Vec<(usize, AaakEntry)> = lines[section_start + 1..section_end]
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| {
            AaakEntry::from_line(line).map(|entry| (section_start + 1 + offset, entry))
        })
        .collect();

    let mut total_chars: usize = entry_lines
        .iter()
        .map(|(_, entry)| entry.to_line().len())
        .sum();

    if total_chars <= budget_chars {
        return Ok(entry_lines.into_iter().map(|(_, entry)| entry).collect());
    }

    let mut sorted = entry_lines.clone();
    sorted.sort_by_key(|(line_index, entry)| (entry.date, *line_index));

    let mut removed_lines = std::collections::BTreeSet::new();
    for (line_index, entry) in sorted {
        if total_chars <= budget_chars {
            break;
        }

        total_chars = total_chars.saturating_sub(entry.to_line().len());
        removed_lines.insert(line_index);
    }

    lines = lines
        .into_iter()
        .enumerate()
        .filter_map(|(line_index, line)| {
            if removed_lines.contains(&line_index) && AaakEntry::from_line(&line).is_some() {
                None
            } else {
                Some(line)
            }
        })
        .collect();

    std::fs::write(agents_md_path, render_lines(&lines, had_trailing_newline))?;

    Ok(entry_lines
        .into_iter()
        .filter_map(|(line_index, entry)| (!removed_lines.contains(&line_index)).then_some(entry))
        .collect())
}

fn collect_lines(content: &str) -> Vec<String> {
    content.lines().map(ToString::to_string).collect()
}

fn find_section_bounds(lines: &[String]) -> Option<(usize, usize)> {
    let section_start = lines
        .iter()
        .position(|line| line.trim_start().starts_with("## AAAK"))?;

    let section_end = lines
        .iter()
        .enumerate()
        .skip(section_start + 1)
        .find_map(|(index, line)| line.trim_start().starts_with("##").then_some(index))
        .unwrap_or(lines.len());

    Some((section_start, section_end))
}

fn render_lines(lines: &[String], had_trailing_newline: bool) -> String {
    let mut rendered = lines.join("\n");
    if had_trailing_newline && !rendered.is_empty() {
        rendered.push('\n');
    }
    rendered
}
