use crate::{
    cli::StatusArgs,
    config::Config,
    error::SamsaraError,
    git,
    knowledge::{index, lesson::Lesson, log},
};

use std::path::Path;

pub fn run(_args: StatusArgs, config: &Config) -> Result<(), SamsaraError> {
    let knowledge_home = &config.knowledge_home;

    if !knowledge_home.exists() {
        println!("知识库未初始化，运行 samsara init");
        return Ok(());
    }

    let domains = index::scan(knowledge_home)?;
    let domain_count = domains.len();
    let total_lessons: usize = domains.iter().map(|entry| entry.lesson_count).sum();
    let promoted_lessons = count_promoted_lessons(&knowledge_home.join("lessons"))?;
    let unpromoted_lessons = total_lessons.saturating_sub(promoted_lessons);
    let rules_count = count_rules_files(&knowledge_home.join("rules"))?;
    let log_count = log::count_lines(&knowledge_home.join("log.md"));
    let uncommitted_files = git::uncommitted_files(knowledge_home);

    println!("📊 samsara status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("知识库路径：{}", knowledge_home.display());
    println!(
        "Domains    : {}（{}）",
        domain_count,
        format_domain_list(&domains)
    );
    println!(
        "Lessons    : {}（promoted: {}，未晋升: {}）",
        total_lessons, promoted_lessons, unpromoted_lessons
    );
    println!(
        "Rules 文件 : {}（{}）",
        rules_count,
        format_rule_list(knowledge_home)?
    );
    println!("Log 条目   : {}", log_count);
    println!("未提交变更：");
    if uncommitted_files.is_empty() {
        println!("  无");
    } else {
        for file in uncommitted_files {
            println!("  {}", file);
        }
    }

    Ok(())
}

fn count_promoted_lessons(lessons_root: &Path) -> Result<usize, SamsaraError> {
    if !lessons_root.exists() {
        return Ok(0);
    }

    let mut total = 0;
    for domain_entry in std::fs::read_dir(lessons_root)? {
        let domain_entry = domain_entry?;
        if !domain_entry.file_type()?.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(domain_entry.path())? {
            let file_entry = file_entry?;
            let path = file_entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            if let Ok(lesson) = Lesson::load(&path) {
                if lesson.frontmatter.promoted {
                    total += 1;
                }
            }
        }
    }

    Ok(total)
}

fn count_rules_files(rules_root: &Path) -> Result<usize, SamsaraError> {
    if !rules_root.exists() {
        return Ok(0);
    }

    let mut total = 0;
    for entry in std::fs::read_dir(rules_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            total += 1;
        }
    }

    Ok(total)
}

fn format_domain_list(domains: &[index::IndexDomainEntry]) -> String {
    let names: Vec<&str> = domains.iter().map(|entry| entry.domain.as_str()).collect();
    match names.len() {
        0 => String::from("-"),
        1..=5 => names.join(", "),
        _ => format!("{}, ...", names[..5].join(", ")),
    }
}

fn format_rule_list(knowledge_home: &Path) -> Result<String, SamsaraError> {
    let rules_root = knowledge_home.join("rules");
    if !rules_root.exists() {
        return Ok(String::from("-"));
    }

    let mut names = Vec::new();
    for entry in std::fs::read_dir(rules_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            names.push(name.to_string());
        }
    }

    names.sort();
    Ok(if names.is_empty() {
        String::from("-")
    } else {
        names.join(", ")
    })
}
