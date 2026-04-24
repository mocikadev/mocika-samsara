use crate::{
    cli::ReflectArgs,
    config::Config,
    error::SamsaraError,
    knowledge::{
        lesson::{Lesson, LessonType},
        log::{self, LogAction, LogEntry},
    },
};

use chrono::{Duration, Local};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::Path,
};

pub fn run(_args: ReflectArgs, config: &Config) -> Result<(), SamsaraError> {
    let today = Local::now().date_naive();
    let lesson_candidates = collect_promotion_candidates(&config.knowledge_home.join("lessons"))?;
    let log_entries = read_all_log_entries(&config.knowledge_home.join("log.md"))?;
    let hot_domains = collect_hot_domains(&log_entries, today);
    let skill_failures = collect_skill_failures(&log_entries);
    let aaak_entities = read_aaak_entities(&config.agents_home.join("AGENTS.md"))?;
    let aaak_enabled = aaak_entities.is_some();
    let aaak_candidates = aaak_entities
        .as_ref()
        .map(|entities| collect_aaak_candidates(&log_entries, entities))
        .unwrap_or_default();

    print_candidates(&lesson_candidates);
    print_hot_domains(&hot_domains);
    print_skill_failures(&skill_failures);
    print_aaak_candidates(&aaak_candidates, aaak_enabled);

    Ok(())
}

#[derive(Clone)]
struct PromotionCandidate {
    path: String,
    occurrences: usize,
    verified: u32,
}

fn collect_promotion_candidates(
    lessons_root: &Path,
) -> Result<BTreeMap<&'static str, Vec<PromotionCandidate>>, SamsaraError> {
    let mut groups: BTreeMap<&'static str, Vec<PromotionCandidate>> = BTreeMap::from([
        ("error", Vec::new()),
        ("skill", Vec::new()),
        ("pattern", Vec::new()),
        ("insight", Vec::new()),
        ("未分类", Vec::new()),
    ]);

    if !lessons_root.exists() {
        return Ok(groups);
    }

    let mut domains: Vec<_> = fs::read_dir(lessons_root)?.collect::<Result<Vec<_>, _>>()?;
    domains.sort_by_key(|entry| entry.file_name());

    for domain_entry in domains {
        if !domain_entry.file_type()?.is_dir() {
            continue;
        }

        let domain = domain_entry.file_name().to_string_lossy().to_string();
        let mut files: Vec<_> =
            fs::read_dir(domain_entry.path())?.collect::<Result<Vec<_>, _>>()?;
        files.sort_by_key(|entry| entry.file_name());

        for file_entry in files {
            let path = file_entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let Some(keyword) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };

            let lesson = match Lesson::load(&path) {
                Ok(lesson) => lesson,
                Err(error) => {
                    eprintln!(
                        "[WARN]  {}: Lesson::load 失败，已跳过（{}）",
                        path.to_string_lossy(),
                        error
                    );
                    continue;
                }
            };

            if lesson.frontmatter.occurrences.len() < 3 || lesson.frontmatter.promoted {
                continue;
            }

            let group = lesson_type_label(lesson.frontmatter.lesson_type.as_ref());
            if let Some(entries) = groups.get_mut(group) {
                entries.push(PromotionCandidate {
                    path: format!("{domain}/{keyword}"),
                    occurrences: lesson.frontmatter.occurrences.len(),
                    verified: lesson.frontmatter.verified,
                });
            }
        }
    }

    for entries in groups.values_mut() {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
    }

    Ok(groups)
}

fn lesson_type_label(lesson_type: Option<&LessonType>) -> &'static str {
    match lesson_type {
        Some(LessonType::Error) => "error",
        Some(LessonType::Skill) => "skill",
        Some(LessonType::Pattern) => "pattern",
        Some(LessonType::Insight) => "insight",
        None => "未分类",
    }
}

fn read_all_log_entries(log_path: &Path) -> Result<Vec<LogEntry>, SamsaraError> {
    let total = log::count_lines(log_path);
    if total == 0 {
        return Ok(Vec::new());
    }
    log::read_last_n(log_path, total)
}

fn collect_hot_domains(log_entries: &[LogEntry], today: chrono::NaiveDate) -> Vec<(String, usize)> {
    let cutoff = today - Duration::days(30);
    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in log_entries {
        if !matches!(entry.action, LogAction::Update) || entry.date < cutoff {
            continue;
        }

        if let Some(domain) = extract_domain(&entry.target) {
            *counts.entry(domain).or_insert(0) += 1;
        }
    }

    let mut result = counts
        .into_iter()
        .filter(|(_, count)| *count > 5)
        .collect::<Vec<_>>();
    result.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    result
}

fn collect_skill_failures(log_entries: &[LogEntry]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in log_entries {
        if matches!(entry.action, LogAction::SkillFail) {
            *counts.entry(entry.target.clone()).or_insert(0) += 1;
        }
    }

    let mut result = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect::<Vec<_>>();
    result.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    result
}

fn read_aaak_entities(agents_md_path: &Path) -> Result<Option<BTreeSet<String>>, SamsaraError> {
    if !agents_md_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(agents_md_path)?;
    let mut entities = BTreeSet::new();
    let mut in_aaak = false;

    for line in content.lines().map(str::trim) {
        if line.starts_with("## AAAK") {
            in_aaak = true;
            continue;
        }

        if in_aaak && line.starts_with("## ") {
            break;
        }

        if !in_aaak || line.is_empty() {
            continue;
        }

        if let Some(entity) = parse_aaak_entity(line) {
            entities.insert(entity);
        }
    }

    Ok(Some(entities))
}

fn parse_aaak_entity(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let entity = inner.split('|').next()?.trim();
    if entity.is_empty() {
        None
    } else {
        Some(entity.to_string())
    }
}

fn collect_aaak_candidates(
    log_entries: &[LogEntry],
    aaak_entities: &BTreeSet<String>,
) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in log_entries {
        if let Some(keyword) = extract_keyword(&entry.target) {
            *counts.entry(keyword).or_insert(0) += 1;
        }
    }

    let mut result = counts
        .into_iter()
        .filter(|(keyword, count)| *count > 3 && !aaak_entities.contains(keyword))
        .collect::<Vec<_>>();
    result.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    result
}

fn extract_domain(target: &str) -> Option<String> {
    target.split('/').next().and_then(|domain| {
        let domain = domain.trim();
        (!domain.is_empty()).then(|| domain.to_string())
    })
}

fn extract_keyword(target: &str) -> Option<String> {
    let first_token = target.split_whitespace().next()?.trim();
    let keyword = first_token
        .rsplit('/')
        .next()?
        .trim_end_matches(".md")
        .trim();
    (!keyword.is_empty()).then(|| keyword.to_string())
}

fn print_candidates(groups: &BTreeMap<&'static str, Vec<PromotionCandidate>>) {
    println!("=== 待晋升候选 ===");
    let ordered_labels = ["error", "skill", "pattern", "insight", "未分类"];
    let total = ordered_labels
        .iter()
        .map(|label| groups.get(label).map_or(0, Vec::len))
        .sum::<usize>();

    if total == 0 {
        println!("  （暂无）");
        println!();
        return;
    }

    for label in ordered_labels {
        println!("[{label}]");
        match groups.get(label) {
            Some(entries) if !entries.is_empty() => {
                for entry in entries {
                    println!(
                        "  {}  occurrences: {}  verified: {}",
                        entry.path, entry.occurrences, entry.verified
                    );
                }
            }
            _ => println!("  （暂无）"),
        }
    }
    println!();
}

fn print_hot_domains(entries: &[(String, usize)]) {
    println!("=== 高频 domain（最近 30 天 > 5 次更新）===");
    if entries.is_empty() {
        println!("  （暂无）");
        println!();
        return;
    }

    for (domain, count) in entries {
        println!("  {domain}: {count} 次更新，建议安装 skill: {domain}");
    }
    println!();
}

fn print_skill_failures(entries: &[(String, usize)]) {
    println!("=== skill 失败统计 ===");
    if entries.is_empty() {
        println!("  （暂无）");
        println!();
        return;
    }

    for (skill, count) in entries {
        println!("  ⚠️ {skill}: {count} 次失败，建议修复");
    }
    println!();
}

fn print_aaak_candidates(entries: &[(String, usize)], aaak_enabled: bool) {
    println!("=== AAAK 候选 ===");
    if !aaak_enabled || entries.is_empty() {
        println!("  （暂无）");
        return;
    }

    for (keyword, count) in entries {
        println!("  {keyword} (出现 {count} 次)");
    }
}
