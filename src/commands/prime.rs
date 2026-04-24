use crate::{
    cli::PrimeArgs,
    config::Config,
    error::SamsaraError,
    knowledge::lesson::{find_lesson, Lesson, LessonType},
};

use chrono::{Local, NaiveDate};
use std::{fs, path::Path};

const SEPARATOR: &str = "────────────────────────────────────────────────";

pub fn run(args: PrimeArgs, config: &Config) -> Result<(), SamsaraError> {
    let sort = normalize_sort(&args.sort);
    let actionable_agents = read_actionable_agents(&config.agents_home.join("AGENTS.md"))?;
    let mut candidates = collect_candidates(
        &config.knowledge_home,
        actionable_agents.as_str(),
        args.domain.as_deref(),
    )?;

    sort_candidates(&mut candidates, sort);

    let top_n: Vec<_> = candidates.into_iter().take(args.limit).collect();
    if top_n.is_empty() {
        println!("暂无可推荐的规则（知识库为空或所有规则已在 AGENTS.md 中）");
        return Ok(());
    }

    println!("{SEPARATOR}");
    println!(
        "samsara prime: Top {} 推荐规则 (sorted: {sort})",
        top_n.len()
    );
    println!("{SEPARATOR}");

    for (index, candidate) in top_n.iter().enumerate() {
        println!(
            " #{:<2} [{}]  [{}] {}",
            index + 1,
            candidate.rule_path(),
            candidate.lesson_type_label(),
            candidate.summary
        );
        println!(
            "    来源: {} | occurrences: {} | verified: {} | last: {} | score: {}",
            candidate.source_path,
            candidate.occurrences,
            candidate.verified,
            candidate.last_occurrence,
            candidate.score
        );
        println!(
            "    → samsara promote --layer0 {} {}",
            candidate.domain, candidate.keyword
        );
    }

    println!("{SEPARATOR}");
    println!("提示: 直接复制上方 samsara promote --layer0 命令执行晋升");

    Ok(())
}

#[derive(Clone)]
struct CandidateRule {
    domain: String,
    keyword: String,
    summary: String,
    source_path: String,
    lesson_type: Option<LessonType>,
    occurrences: usize,
    verified: u32,
    last_occurrence: NaiveDate,
    score: i32,
}

impl CandidateRule {
    fn rule_path(&self) -> String {
        format!("{}/{}", self.domain, self.keyword)
    }

    fn lesson_type_label(&self) -> &'static str {
        match self.lesson_type.as_ref() {
            Some(LessonType::Error) => "error",
            Some(LessonType::Skill) => "skill",
            Some(LessonType::Pattern) => "pattern",
            Some(LessonType::Insight) => "insight",
            None => "未分类",
        }
    }
}

struct ParsedRule {
    keyword: String,
    summary: String,
}

fn collect_candidates(
    knowledge_home: &Path,
    actionable_agents: &str,
    domain_filter: Option<&str>,
) -> Result<Vec<CandidateRule>, SamsaraError> {
    let rules_dir = knowledge_home.join("rules");
    if !rules_dir.exists() {
        return Ok(Vec::new());
    }

    let today = Local::now().date_naive();
    let mut entries = fs::read_dir(&rules_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut candidates = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(domain) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if domain_filter.is_some_and(|filter| filter != domain) {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let parsed_rules = parse_rules(&content);

        for parsed_rule in parsed_rules {
            let Some(lesson_path) = find_lesson(knowledge_home, domain, &parsed_rule.keyword)
            else {
                continue;
            };

            let lesson = Lesson::load(&lesson_path)?;
            let Some(last_occurrence) = lesson.frontmatter.occurrences.iter().copied().max() else {
                continue;
            };

            let score = calculate_score(
                &lesson,
                last_occurrence,
                today,
                actionable_agents.contains(&format!("{domain}/{}", parsed_rule.keyword)),
            );

            candidates.push(CandidateRule {
                domain: domain.to_string(),
                keyword: parsed_rule.keyword,
                summary: parsed_rule.summary,
                source_path: format!("rules/{domain}.md"),
                lesson_type: lesson.frontmatter.lesson_type.clone(),
                occurrences: lesson.frontmatter.occurrences.len(),
                verified: lesson.frontmatter.verified,
                last_occurrence,
                score,
            });
        }
    }

    Ok(candidates)
}

fn parse_rules(content: &str) -> Vec<ParsedRule> {
    let mut rules = Vec::new();
    let mut current_keyword: Option<String> = None;
    let mut section_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(keyword) = line.strip_prefix("## ") {
            if let Some(keyword) = current_keyword.take() {
                rules.push(ParsedRule {
                    keyword,
                    summary: extract_summary(&section_lines),
                });
            }

            current_keyword = Some(keyword.trim().to_string());
            section_lines.clear();
        } else if current_keyword.is_some() {
            section_lines.push(line.to_string());
        }
    }

    if let Some(keyword) = current_keyword {
        rules.push(ParsedRule {
            keyword,
            summary: extract_summary(&section_lines),
        });
    }

    rules
}

fn extract_summary(section_lines: &[String]) -> String {
    section_lines
        .iter()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && !line.starts_with("来源：") && *line != "---")
        .unwrap_or("")
        .to_string()
}

fn calculate_score(
    lesson: &Lesson,
    last_occurrence: NaiveDate,
    today: NaiveDate,
    already_in_agents: bool,
) -> i32 {
    let days_since_last = today.signed_duration_since(last_occurrence).num_days();
    let recency_bonus = ((30 - days_since_last).max(0) as i32) * 5;
    let error_bonus = if matches!(
        lesson.frontmatter.lesson_type.as_ref(),
        Some(LessonType::Error)
    ) {
        20
    } else {
        0
    };
    let conflict_penalty = if lesson
        .frontmatter
        .conflicts_with
        .as_ref()
        .is_some_and(|items| !items.is_empty())
    {
        10
    } else {
        0
    };

    let mut score = lesson.frontmatter.occurrences.len() as i32 * 10
        + recency_bonus
        + error_bonus
        + lesson.frontmatter.verified as i32 * 15
        - conflict_penalty;

    if already_in_agents {
        score /= 2;
    }

    score
}

fn sort_candidates(candidates: &mut [CandidateRule], sort: &str) {
    match sort {
        "occurrences" => candidates.sort_by(|left, right| {
            right
                .occurrences
                .cmp(&left.occurrences)
                .then_with(|| right.last_occurrence.cmp(&left.last_occurrence))
                .then_with(|| right.score.cmp(&left.score))
                .then_with(|| left.domain.cmp(&right.domain))
                .then_with(|| left.keyword.cmp(&right.keyword))
        }),
        "domain" => candidates.sort_by(|left, right| {
            left.domain
                .cmp(&right.domain)
                .then_with(|| right.score.cmp(&left.score))
                .then_with(|| right.last_occurrence.cmp(&left.last_occurrence))
                .then_with(|| left.keyword.cmp(&right.keyword))
        }),
        _ => candidates.sort_by(|left, right| {
            right
                .last_occurrence
                .cmp(&left.last_occurrence)
                .then_with(|| right.score.cmp(&left.score))
                .then_with(|| right.occurrences.cmp(&left.occurrences))
                .then_with(|| left.domain.cmp(&right.domain))
                .then_with(|| left.keyword.cmp(&right.keyword))
        }),
    }
}

fn normalize_sort(sort: &str) -> &str {
    match sort.trim().to_ascii_lowercase().as_str() {
        "occurrences" => "occurrences",
        "domain" => "domain",
        _ => "recent",
    }
}

fn read_actionable_agents(agents_md_path: &Path) -> Result<String, SamsaraError> {
    if !agents_md_path.exists() {
        return Ok(String::new());
    }

    let content = fs::read_to_string(agents_md_path)?;
    let mut actionable = Vec::new();
    let mut protected = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("## ") {
            protected = trimmed.starts_with("## 自我进化协议") || trimmed.starts_with("## AAAK");
        }

        if !protected {
            actionable.push(line);
        }
    }

    Ok(actionable.join("\n"))
}
