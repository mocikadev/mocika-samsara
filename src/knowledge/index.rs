use crate::error::SamsaraError;
use crate::knowledge::lesson::Lesson;
use chrono::NaiveDate;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub struct IndexDomainEntry {
    pub domain: String,
    pub tags: Vec<String>,
    pub lesson_count: usize,
    pub has_rules: bool,
    pub related_skill: Option<String>,
    pub last_written: Option<NaiveDate>,
}

pub fn rebuild(knowledge_home: &Path, dry_run: bool) -> Result<(), SamsaraError> {
    let entries = scan(knowledge_home)?;
    if dry_run {
        return Ok(());
    }
    let content = render_index(&entries, knowledge_home);
    let index_path = knowledge_home.join("INDEX.md");
    std::fs::write(&index_path, content)?;
    Ok(())
}

pub fn scan(knowledge_home: &Path) -> Result<Vec<IndexDomainEntry>, SamsaraError> {
    let lessons_dir = knowledge_home.join("lessons");
    if !lessons_dir.exists() {
        return Ok(vec![]);
    }

    let mut domain_map: BTreeMap<String, DomainAccum> = BTreeMap::new();

    for entry in std::fs::read_dir(&lessons_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let domain = entry.file_name().to_string_lossy().to_string();
        let accum = domain_map.entry(domain.clone()).or_default();

        for file in std::fs::read_dir(entry.path())? {
            let file = file?;
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            accum.lesson_count += 1;
            if let Ok(lesson) = Lesson::load(&path) {
                let last = lesson.frontmatter.occurrences.iter().max().copied();
                if let Some(d) = last {
                    if accum.last_written.is_none_or(|prev| d > prev) {
                        accum.last_written = Some(d);
                    }
                }
                for tag in &lesson.frontmatter.tags {
                    accum.tags.insert(tag.clone());
                }
            }
        }

        let rules_file = knowledge_home.join("rules").join(format!("{domain}.md"));
        accum.has_rules = rules_file.exists();
    }

    let result = domain_map
        .into_iter()
        .map(|(domain, a)| {
            let mut tags: Vec<String> = a.tags.into_iter().collect();
            tags.sort();
            IndexDomainEntry {
                domain,
                tags,
                lesson_count: a.lesson_count,
                has_rules: a.has_rules,
                related_skill: None,
                last_written: a.last_written,
            }
        })
        .collect();

    Ok(result)
}

#[derive(Default)]
struct DomainAccum {
    lesson_count: usize,
    has_rules: bool,
    tags: HashSet<String>,
    last_written: Option<NaiveDate>,
}

fn render_index(entries: &[IndexDomainEntry], _knowledge_home: &Path) -> String {
    use std::fmt::Write;
    let today = chrono::Local::now().date_naive();
    let total_lessons: usize = entries.iter().map(|e| e.lesson_count).sum();
    let domains_with_rules = entries.iter().filter(|e| e.has_rules).count();

    let mut out = String::new();
    let _ = writeln!(out, "# Samsara · 知识库索引");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "> 此文件由 `samsara` 自动生成，请勿手动编辑。上次更新：{today}"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## 汇总");
    let _ = writeln!(out);
    let _ = writeln!(out, "| 统计项 | 数量 |");
    let _ = writeln!(out, "|--------|------|");
    let _ = writeln!(out, "| Domain | {} |", entries.len());
    let _ = writeln!(out, "| Lesson | {total_lessons} |");
    let _ = writeln!(out, "| 含 rules 的 Domain | {domains_with_rules} |");
    let _ = writeln!(out);
    let _ = writeln!(out, "## Domain 列表");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Domain | Lessons | Rules | 最近写入 | 标签 |");
    let _ = writeln!(out, "|--------|---------|-------|---------|------|");

    for e in entries {
        let rules = if e.has_rules { "✓" } else { "-" };
        let last = e
            .last_written
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let tags = if e.tags.is_empty() {
            "-".to_string()
        } else {
            e.tags.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            e.domain, e.lesson_count, rules, last, tags
        );
    }

    out
}
