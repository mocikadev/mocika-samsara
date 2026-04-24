use crate::{
    cli::LintArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::{
        index,
        lesson::{find_lesson, Lesson},
        log::{self, LogAction, LogEntry},
    },
};

use chrono::{Datelike, Duration, Local, NaiveDate};
use serde_yaml::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    io::{self, Write},
    path::Path,
};

pub fn run(args: LintArgs, config: &Config) -> Result<(), SamsaraError> {
    let today = Local::now().date_naive();
    let mut report = collect_report(config, today)?;

    if args.fix {
        apply_fixes(&mut report, config, args.yes, today)?;
    }

    print_report(&report);

    if report.has_error() {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Severity {
    Error,
    Warn,
    Info,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum IssueStatus {
    Open,
    Fixed,
    Skipped,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CheckId {
    LessonTooLong,
    RulesTooLong,
    MissingFrontmatterFields,
    InvalidOccurrences,
    ExpiredLesson,
    StaleLesson,
    MissingPromotedRuleSection,
    IndexDomainMismatch,
    AgentsRuleOverflow,
    MissingLessonReference,
    LogTooLong,
    MissingConflictTarget,
    DuplicateLessonCandidate,
}

struct Issue {
    check: CheckId,
    severity: Severity,
    location: Option<String>,
    message: String,
    status: IssueStatus,
}

struct LintReport {
    issues: Vec<Issue>,
}

impl LintReport {
    fn new() -> Self {
        Self { issues: Vec::new() }
    }

    fn push(
        &mut self,
        check: CheckId,
        severity: Severity,
        location: Option<String>,
        message: impl Into<String>,
    ) {
        self.issues.push(Issue {
            check,
            severity,
            location,
            message: message.into(),
            status: IssueStatus::Open,
        });
    }

    fn has_error(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == Severity::Error && issue.status != IssueStatus::Fixed)
    }

    fn active_counts(&self) -> (usize, usize, usize) {
        self.issues
            .iter()
            .filter(|issue| issue.status != IssueStatus::Fixed)
            .fold((0, 0, 0), |(errors, warns, infos), issue| {
                match issue.severity {
                    Severity::Error => (errors + 1, warns, infos),
                    Severity::Warn => (errors, warns + 1, infos),
                    Severity::Info => (errors, warns, infos + 1),
                }
            })
    }

    fn mark_by_location_and_checks(
        &mut self,
        location: &str,
        checks: &[CheckId],
        status: IssueStatus,
    ) {
        for issue in &mut self.issues {
            if issue.location.as_deref() == Some(location) && checks.contains(&issue.check) {
                issue.status = status;
            }
        }
    }

    fn mark_by_check(&mut self, check: CheckId, status: IssueStatus) {
        for issue in &mut self.issues {
            if issue.check == check {
                issue.status = status;
            }
        }
    }
}

struct LoadedLesson {
    lesson: Lesson,
    domain: String,
    keyword: String,
}

fn collect_report(config: &Config, today: NaiveDate) -> Result<LintReport, SamsaraError> {
    let knowledge_home = &config.knowledge_home;
    let lessons_dir = knowledge_home.join("lessons");
    let rules_dir = knowledge_home.join("rules");

    let rules_snapshot = scan_rules(knowledge_home, &rules_dir)?;
    let mut report = LintReport::new();
    let mut lessons_by_domain: BTreeMap<String, Vec<LoadedLesson>> = BTreeMap::new();

    if lessons_dir.exists() {
        let mut domains: Vec<_> = fs::read_dir(&lessons_dir)?.collect::<Result<Vec<_>, _>>()?;
        domains.sort_by_key(|entry| entry.file_name());

        for domain_entry in domains {
            if !domain_entry.file_type()?.is_dir() {
                continue;
            }

            let mut files: Vec<_> =
                fs::read_dir(domain_entry.path())?.collect::<Result<Vec<_>, _>>()?;
            files.sort_by_key(|entry| entry.file_name());

            for file_entry in files {
                let path = file_entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }

                let relative = relative_to(knowledge_home, &path);
                let line_count = count_lines(&path)?;
                if line_count > 30 {
                    report.push(
                        CheckId::LessonTooLong,
                        Severity::Error,
                        Some(relative.clone()),
                        format!("文件 {line_count} 行，超过 30 行上限"),
                    );
                }

                match load_lesson_for_lint(&path)? {
                    Ok(lesson) => {
                        let Some(domain) = path
                            .parent()
                            .and_then(Path::file_name)
                            .and_then(|name| name.to_str())
                            .map(str::to_string)
                        else {
                            continue;
                        };
                        let Some(keyword) = path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .map(str::to_string)
                        else {
                            continue;
                        };

                        if lesson.is_expired(today) {
                            let valid_until = lesson
                                .frontmatter
                                .valid_until
                                .map(|date| date.to_string())
                                .unwrap_or_else(|| "<unknown>".to_string());
                            report.push(
                                CheckId::ExpiredLesson,
                                Severity::Warn,
                                Some(relative.clone()),
                                format!("valid_until {valid_until} 已过期"),
                            );
                        }

                        let latest_occurrence =
                            lesson.frontmatter.occurrences.iter().max().copied();
                        if !lesson.frontmatter.promoted
                            && latest_occurrence
                                .is_some_and(|date| date < today - Duration::days(90))
                        {
                            let last_seen = latest_occurrence
                                .map(|date| date.to_string())
                                .unwrap_or_else(|| "<unknown>".to_string());
                            report.push(
                                CheckId::StaleLesson,
                                Severity::Warn,
                                Some(relative.clone()),
                                format!("90 天无新 write，最后一次 occurrence 为 {last_seen}"),
                            );
                        }

                        if lesson.frontmatter.promoted
                            && !rules_snapshot.has_section(&domain, &keyword)
                        {
                            report.push(
                                CheckId::MissingPromotedRuleSection,
                                Severity::Warn,
                                Some(relative.clone()),
                                format!("promoted=true，但 rules/{domain}.md 中无对应 ## {keyword} section"),
                            );
                        }

                        if let Some(conflicts) = &lesson.frontmatter.conflicts_with {
                            for conflict in conflicts {
                                let (conflict_domain, conflict_keyword) =
                                    split_conflict_target(&domain, conflict);
                                let lesson_exists = find_lesson(
                                    knowledge_home,
                                    &conflict_domain,
                                    &conflict_keyword,
                                )
                                .is_some();
                                let rule_exists =
                                    rules_snapshot.has_section(&conflict_domain, &conflict_keyword);

                                if !lesson_exists && !rule_exists {
                                    report.push(
                                        CheckId::MissingConflictTarget,
                                        Severity::Warn,
                                        Some(relative.clone()),
                                        format!(
                                            "conflicts_with 引用不存在：{conflict_domain}/{conflict_keyword}"
                                        ),
                                    );
                                }
                            }
                        }

                        lessons_by_domain
                            .entry(domain.clone())
                            .or_default()
                            .push(LoadedLesson {
                                lesson,
                                domain,
                                keyword,
                            });
                    }
                    Err(problem) => match problem {
                        LessonLintLoadError::MissingFields(fields) => {
                            let message = if fields.is_empty() {
                                "frontmatter 缺少必填字段或格式非法".to_string()
                            } else {
                                format!("frontmatter 缺少必填字段：{}", fields.join("/"))
                            };
                            report.push(
                                CheckId::MissingFrontmatterFields,
                                Severity::Error,
                                Some(relative),
                                message,
                            );
                        }
                        LessonLintLoadError::InvalidOccurrences => report.push(
                            CheckId::InvalidOccurrences,
                            Severity::Error,
                            Some(relative),
                            "occurrences 格式非法，应为 ISO-8601 日期数组",
                        ),
                    },
                }
            }
        }
    }

    for (domain, rule_data) in &rules_snapshot.by_domain {
        let relative = format!("rules/{domain}.md");

        if rule_data.line_count > 100 {
            report.push(
                CheckId::RulesTooLong,
                Severity::Warn,
                Some(relative.clone()),
                format!("文件 {} 行，超过 100 行上限", rule_data.line_count),
            );
        }

        for source in &rule_data.sources {
            let source_path = knowledge_home.join(source);
            if !source_path.exists() {
                report.push(
                    CheckId::MissingLessonReference,
                    Severity::Warn,
                    Some(relative.clone()),
                    format!("引用的 lesson 路径不存在：{source}"),
                );
            }
        }
    }

    if index_domains_mismatch(knowledge_home, &lessons_dir)? {
        report.push(
            CheckId::IndexDomainMismatch,
            Severity::Warn,
            Some("INDEX.md".to_string()),
            "domain 列与 lessons/ 子目录不一致",
        );
    }

    if let Some(rule_lines) = count_agents_rule_lines(&config.agents_home.join("AGENTS.md"))? {
        if rule_lines > 100 {
            report.push(
                CheckId::AgentsRuleOverflow,
                Severity::Warn,
                Some("AGENTS.md".to_string()),
                format!("实质规则 {rule_lines} 行，已超过 100 行上限"),
            );
        }
    }

    let log_path = knowledge_home.join("log.md");
    let log_lines = log::count_lines(&log_path);
    if log_lines > 1000 {
        report.push(
            CheckId::LogTooLong,
            Severity::Info,
            Some("log.md".to_string()),
            format!("文件 {log_lines} 行，超过 1000 行上限"),
        );
    }

    for lessons in lessons_by_domain.into_values() {
        for (left_index, left) in lessons.iter().enumerate() {
            for right in lessons.iter().skip(left_index + 1) {
                let left_tags: HashSet<&str> = left
                    .lesson
                    .frontmatter
                    .tags
                    .iter()
                    .map(String::as_str)
                    .collect();
                let right_tags: HashSet<&str> = right
                    .lesson
                    .frontmatter
                    .tags
                    .iter()
                    .map(String::as_str)
                    .collect();

                let union = left_tags.union(&right_tags).count();
                if union == 0 {
                    continue;
                }

                let inter = left_tags.intersection(&right_tags).count();
                let jaccard = inter as f64 / union as f64;
                if jaccard >= 0.7 {
                    report.push(
                        CheckId::DuplicateLessonCandidate,
                        Severity::Info,
                        None,
                        format!(
                            "蒸馏候选: {}/{} + {}/{} (tags Jaccard: {:.2})",
                            left.domain, left.keyword, right.domain, right.keyword, jaccard
                        ),
                    );
                }
            }
        }
    }

    Ok(report)
}

fn apply_fixes(
    report: &mut LintReport,
    config: &Config,
    yes: bool,
    today: NaiveDate,
) -> Result<(), SamsaraError> {
    let knowledge_home = &config.knowledge_home;
    let archive_candidates = collect_archive_candidates(&report.issues);
    let mut archived_count = 0usize;

    for location in archive_candidates {
        let should_archive = if yes {
            true
        } else {
            confirm(format!("归档 {location}？[y/N] "))?
        };

        if should_archive {
            archive_lesson(knowledge_home, &location, config.dry_run)?;
            report.mark_by_location_and_checks(
                &location,
                &[CheckId::ExpiredLesson, CheckId::StaleLesson],
                IssueStatus::Fixed,
            );
            archived_count += 1;
        } else {
            report.mark_by_location_and_checks(
                &location,
                &[CheckId::ExpiredLesson, CheckId::StaleLesson],
                IssueStatus::Skipped,
            );
        }
    }

    let should_rebuild_index = archived_count > 0
        || report.issues.iter().any(|issue| {
            issue.check == CheckId::IndexDomainMismatch && issue.status == IssueStatus::Open
        });
    let mut rebuilt_index = false;
    if should_rebuild_index {
        index::rebuild(knowledge_home, config.dry_run)?;
        report.mark_by_check(CheckId::IndexDomainMismatch, IssueStatus::Fixed);
        rebuilt_index = true;
    }

    if config.auto_commit && !config.dry_run && (archived_count > 0 || rebuilt_index) {
        git::auto_commit(
            knowledge_home,
            &format!("samsara: lint --fix (archive {archived_count} lessons, rebuild INDEX)"),
        )?;
    }

    if report
        .issues
        .iter()
        .any(|issue| issue.check == CheckId::LogTooLong && issue.status == IssueStatus::Open)
    {
        rotate_log(knowledge_home, today, config.dry_run)?;
        report.mark_by_check(CheckId::LogTooLong, IssueStatus::Fixed);

        if config.auto_commit && !config.dry_run {
            git::auto_commit(knowledge_home, "samsara: log rotate")?;
        }
    }

    Ok(())
}

fn collect_archive_candidates(issues: &[Issue]) -> Vec<String> {
    let mut locations = BTreeSet::new();
    for issue in issues {
        if matches!(issue.check, CheckId::ExpiredLesson | CheckId::StaleLesson)
            && issue.status == IssueStatus::Open
        {
            if let Some(location) = &issue.location {
                locations.insert(location.clone());
            }
        }
    }
    locations.into_iter().collect()
}

fn archive_lesson(
    knowledge_home: &Path,
    relative_location: &str,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    let source_path = knowledge_home.join(relative_location);
    let Some(path_after_prefix) = relative_location.strip_prefix("lessons/") else {
        return Ok(());
    };
    let destination_path = knowledge_home.join("archive").join(path_after_prefix);

    if dry_run {
        return Ok(());
    }

    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(source_path, destination_path)?;
    Ok(())
}

fn rotate_log(knowledge_home: &Path, today: NaiveDate, dry_run: bool) -> Result<(), SamsaraError> {
    let log_path = knowledge_home.join("log.md");
    if !log_path.exists() {
        return Ok(());
    }

    let entry_count = log::count_lines(&log_path);
    if entry_count == 0 {
        return Ok(());
    }

    let cutoff = today - Duration::days(90);
    let entries = log::read_last_n(&log_path, entry_count)?;
    let mut old_by_year: BTreeMap<i32, Vec<LogEntry>> = BTreeMap::new();
    let mut recent = Vec::new();

    for entry in entries {
        if entry.date < cutoff {
            old_by_year
                .entry(entry.date.year())
                .or_default()
                .push(entry);
        } else {
            recent.push(entry);
        }
    }

    if old_by_year.is_empty() {
        return Ok(());
    }

    if dry_run {
        return Ok(());
    }

    for (year, entries) in old_by_year {
        let archive_path = knowledge_home.join(format!("log.archive-{year}.md"));
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(archive_path)?;
        for entry in entries {
            writeln!(file, "{}", format_log_entry(&entry))?;
        }
    }

    let recent_content = if recent.is_empty() {
        String::new()
    } else {
        let mut content = recent
            .iter()
            .map(format_log_entry)
            .collect::<Vec<_>>()
            .join("\n");
        content.push('\n');
        content
    };
    fs::write(log_path, recent_content)?;

    Ok(())
}

fn print_report(report: &LintReport) {
    for issue in report
        .issues
        .iter()
        .filter(|issue| issue.status != IssueStatus::Fixed)
    {
        let suffix = if issue.status == IssueStatus::Skipped {
            " [skipped]"
        } else {
            ""
        };

        match &issue.location {
            Some(location) => println!(
                "{}{}: {}{}",
                severity_prefix(issue.severity),
                location,
                issue.message,
                suffix
            ),
            None => println!(
                "{}{}{}",
                severity_prefix(issue.severity),
                issue.message,
                suffix
            ),
        }
    }

    let (errors, warns, infos) = report.active_counts();
    println!(
        "共 {} 个问题（ERROR: {}，WARN: {}，INFO: {}）",
        errors + warns + infos,
        errors,
        warns,
        infos
    );
}

fn severity_prefix(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "[ERROR] ",
        Severity::Warn => "[WARN]  ",
        Severity::Info => "[INFO]  ",
    }
}

enum LessonLintLoadError {
    MissingFields(Vec<String>),
    InvalidOccurrences,
}

fn load_lesson_for_lint(path: &Path) -> Result<Result<Lesson, LessonLintLoadError>, SamsaraError> {
    match Lesson::load(path) {
        Ok(lesson) => Ok(Ok(lesson)),
        Err(_) => {
            let content = fs::read_to_string(path)?;
            let Some(frontmatter) = extract_frontmatter_block(&content) else {
                return Ok(Err(LessonLintLoadError::MissingFields(Vec::new())));
            };

            let yaml = match serde_yaml::from_str::<Value>(frontmatter) {
                Ok(value) => value,
                Err(_) => return Ok(Err(LessonLintLoadError::MissingFields(Vec::new()))),
            };

            let Some(mapping) = yaml.as_mapping() else {
                return Ok(Err(LessonLintLoadError::MissingFields(Vec::new())));
            };

            let required = ["date", "domain", "tags", "occurrences", "promoted"];
            let missing_fields = required
                .iter()
                .filter(|field| !mapping.contains_key(Value::String((*field).to_string())))
                .map(|field| (*field).to_string())
                .collect::<Vec<_>>();

            if !missing_fields.is_empty() {
                return Ok(Err(LessonLintLoadError::MissingFields(missing_fields)));
            }

            let Some(occurrences) = mapping.get(Value::String("occurrences".to_string())) else {
                return Ok(Err(LessonLintLoadError::MissingFields(vec![
                    "occurrences".to_string()
                ])));
            };

            let Some(values) = occurrences.as_sequence() else {
                return Ok(Err(LessonLintLoadError::InvalidOccurrences));
            };

            let invalid_occurrences = values.iter().any(|value| {
                value
                    .as_str()
                    .is_none_or(|raw| NaiveDate::parse_from_str(raw, "%Y-%m-%d").is_err())
            });

            if invalid_occurrences {
                return Ok(Err(LessonLintLoadError::InvalidOccurrences));
            }

            Ok(Err(LessonLintLoadError::MissingFields(Vec::new())))
        }
    }
}

fn extract_frontmatter_block(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some(&rest[..end])
}

struct RulesSnapshot {
    by_domain: BTreeMap<String, RuleFileData>,
}

impl RulesSnapshot {
    fn has_section(&self, domain: &str, keyword: &str) -> bool {
        self.by_domain
            .get(domain)
            .is_some_and(|data| data.sections.contains(keyword))
    }
}

struct RuleFileData {
    line_count: usize,
    sections: HashSet<String>,
    sources: Vec<String>,
}

fn scan_rules(knowledge_home: &Path, rules_dir: &Path) -> Result<RulesSnapshot, SamsaraError> {
    let mut by_domain = BTreeMap::new();

    if !rules_dir.exists() {
        return Ok(RulesSnapshot { by_domain });
    }

    let mut entries: Vec<_> = fs::read_dir(rules_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(domain) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_string)
        else {
            continue;
        };

        let content = fs::read_to_string(&path)?;
        let mut sections = HashSet::new();
        let mut sources = Vec::new();

        for line in content.lines().map(str::trim) {
            if let Some(keyword) = line.strip_prefix("## ") {
                sections.insert(keyword.trim().to_string());
            }
            if let Some(source) = extract_source_path(line) {
                let full_path = knowledge_home.join(&source);
                let relative = relative_to(knowledge_home, &full_path);
                let normalized = if source.starts_with("lessons/") {
                    source
                } else {
                    relative
                };
                sources.push(normalized);
            }
        }

        by_domain.insert(
            domain,
            RuleFileData {
                line_count: content.lines().count(),
                sections,
                sources,
            },
        );
    }

    Ok(RulesSnapshot { by_domain })
}

fn extract_source_path(line: &str) -> Option<String> {
    let raw = line.strip_prefix("来源：")?.trim();
    let end = raw
        .find('（')
        .or_else(|| raw.find('('))
        .unwrap_or(raw.len());
    Some(raw[..end].trim().trim_end_matches('.').to_string())
}

fn index_domains_mismatch(knowledge_home: &Path, lessons_dir: &Path) -> Result<bool, SamsaraError> {
    let actual = collect_lesson_domains(lessons_dir)?;
    let indexed = parse_index_domains(&knowledge_home.join("INDEX.md"))?;
    Ok(actual != indexed)
}

fn collect_lesson_domains(lessons_dir: &Path) -> Result<BTreeSet<String>, SamsaraError> {
    if !lessons_dir.exists() {
        return Ok(BTreeSet::new());
    }

    let mut domains = BTreeSet::new();
    for entry in fs::read_dir(lessons_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            domains.insert(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(domains)
}

fn parse_index_domains(index_path: &Path) -> Result<BTreeSet<String>, SamsaraError> {
    if !index_path.exists() {
        return Ok(BTreeSet::new());
    }

    let content = fs::read_to_string(index_path)?;
    let mut in_domain_table = false;
    let mut domains = BTreeSet::new();

    for line in content.lines().map(str::trim) {
        if line == "## Domain 列表" {
            in_domain_table = true;
            continue;
        }

        if !in_domain_table {
            continue;
        }

        if line.starts_with("## ") {
            break;
        }

        if !line.starts_with('|') || line.contains("|--------") || line.contains("| Domain |") {
            continue;
        }

        let columns = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if let Some(domain) = columns.first().filter(|domain| !domain.is_empty()) {
            domains.insert((*domain).to_string());
        }
    }

    Ok(domains)
}

fn count_agents_rule_lines(path: &Path) -> Result<Option<usize>, SamsaraError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let mut in_aaak = false;
    let mut count = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## AAAK") {
            in_aaak = true;
            continue;
        }

        if in_aaak && trimmed.starts_with("## ") {
            in_aaak = false;
        }

        if in_aaak || trimmed.is_empty() || trimmed.starts_with("<!--") {
            continue;
        }

        count += 1;
    }

    Ok(Some(count))
}

fn split_conflict_target(current_domain: &str, conflict: &str) -> (String, String) {
    if let Some((domain, keyword)) = conflict.split_once('/') {
        (domain.to_string(), keyword.to_string())
    } else {
        (current_domain.to_string(), conflict.to_string())
    }
}

fn count_lines(path: &Path) -> Result<usize, SamsaraError> {
    Ok(fs::read_to_string(path)?.lines().count())
}

fn relative_to(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn confirm(prompt: String) -> Result<bool, SamsaraError> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn format_log_entry(entry: &LogEntry) -> String {
    let action = match entry.action {
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
    };

    match &entry.note {
        Some(note) => format!("{} {:<12} {} ({})", entry.date, action, entry.target, note),
        None => format!("{} {:<12} {}", entry.date, action, entry.target),
    }
}
