use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::SearchArgs,
    config::Config,
    error::SamsaraError,
    knowledge::lesson::{Lesson, LessonType},
};

struct SearchHit {
    display_path: String,
    score: usize,
    tags: Vec<String>,
    occurrences: usize,
    previews: Vec<String>,
}

pub fn run(args: SearchArgs, config: &Config) -> Result<(), SamsaraError> {
    let query = args.query.as_str();
    let query_lower = query.to_lowercase();

    if query_lower.is_empty() {
        println!("未找到匹配结果：{query}");
        return Ok(());
    }

    let candidates = collect_candidates(&args, &config.knowledge_home)?;
    let mut hits = Vec::new();

    for path in candidates.lessons {
        if let Some(hit) = evaluate_lesson(
            &path,
            &config.knowledge_home,
            query,
            &query_lower,
            args.r#type.as_deref(),
        ) {
            hits.push(hit);
        }
    }

    for path in candidates.rules {
        if let Some(hit) = evaluate_rule(&path, &config.knowledge_home, query, &query_lower)? {
            hits.push(hit);
        }
    }

    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.display_path.cmp(&right.display_path))
    });

    if hits.is_empty() {
        println!("未找到匹配结果：{query}");
        return Ok(());
    }

    for hit in hits.into_iter().take(args.limit) {
        let tags = if hit.tags.is_empty() {
            "-".to_string()
        } else {
            hit.tags.join(", ")
        };

        println!(
            "[{}]  tags: {}  occurrences: {}",
            hit.display_path, tags, hit.occurrences
        );

        for preview in hit.previews {
            println!("  {preview}");
        }
    }

    Ok(())
}

struct CandidatePaths {
    lessons: Vec<PathBuf>,
    rules: Vec<PathBuf>,
}

fn collect_candidates(
    args: &SearchArgs,
    knowledge_home: &Path,
) -> Result<CandidatePaths, SamsaraError> {
    let search_lessons = !args.rules_only;
    let search_rules = !args.lessons_only;
    let mut lessons = Vec::new();
    let mut rules = Vec::new();

    if search_lessons {
        let lessons_root = knowledge_home.join("lessons");
        if let Some(domain) = args.domain.as_deref() {
            collect_markdown_files(&lessons_root.join(domain), false, &mut lessons)?;
        } else {
            collect_markdown_files(&lessons_root, true, &mut lessons)?;
        }
    }

    if search_rules {
        let rules_root = knowledge_home.join("rules");
        if let Some(domain) = args.domain.as_deref() {
            let path = rules_root.join(format!("{domain}.md"));
            if path.is_file() {
                rules.push(path);
            }
        } else {
            collect_markdown_files(&rules_root, false, &mut rules)?;
        }
    }

    Ok(CandidatePaths { lessons, rules })
}

fn collect_markdown_files(
    dir: &Path,
    recursive: bool,
    output: &mut Vec<PathBuf>,
) -> Result<(), SamsaraError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if recursive {
                collect_markdown_files(&path, true, output)?;
            }
            continue;
        }

        if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            output.push(path);
        }
    }

    Ok(())
}

fn evaluate_lesson(
    path: &Path,
    knowledge_home: &Path,
    query: &str,
    query_lower: &str,
    type_filter: Option<&str>,
) -> Option<SearchHit> {
    let lesson = match Lesson::load(path) {
        Ok(lesson) => lesson,
        Err(error) => {
            eprintln!(
                "warning: 跳过 lesson {}: {error}",
                relative_path(knowledge_home, path)
            );
            return None;
        }
    };

    let mut score = 0;

    if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.to_lowercase() == query_lower)
    {
        score += 100;
    }

    if lesson
        .frontmatter
        .tags
        .iter()
        .any(|tag| tag.to_lowercase() == query_lower)
    {
        score += 50;
    }

    if lesson_domain_name(path).is_some_and(|domain| domain.to_lowercase() == query_lower) {
        score += 40;
    }

    let body_matches = count_occurrences_case_insensitive(&lesson.body, query_lower);
    score += body_matches.min(5) * 10;

    if score == 0 {
        return None;
    }

    if let Some(requested_type) = type_filter {
        if !lesson_type_matches(lesson.frontmatter.lesson_type.as_ref(), requested_type) {
            return None;
        }
    }

    Some(SearchHit {
        display_path: relative_path(knowledge_home, path),
        score,
        tags: lesson.frontmatter.tags,
        occurrences: lesson.frontmatter.occurrences.len(),
        previews: preview_matching_lines(&lesson.body, query, query_lower),
    })
}

fn evaluate_rule(
    path: &Path,
    knowledge_home: &Path,
    query: &str,
    query_lower: &str,
) -> Result<Option<SearchHit>, SamsaraError> {
    let content = fs::read_to_string(path)?;
    let mut score = 0;

    if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.to_lowercase() == query_lower)
    {
        score += 100;
    }

    let body_matches = count_occurrences_case_insensitive(&content, query_lower);
    score += body_matches.min(5) * 10;

    if score == 0 {
        return Ok(None);
    }

    Ok(Some(SearchHit {
        display_path: relative_path(knowledge_home, path),
        score,
        tags: Vec::new(),
        occurrences: 0,
        previews: preview_matching_lines(&content, query, query_lower),
    }))
}

fn lesson_domain_name(path: &Path) -> Option<&str> {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
}

fn lesson_type_matches(lesson_type: Option<&LessonType>, requested_type: &str) -> bool {
    lesson_type
        .map(lesson_type_name)
        .is_some_and(|value| value.eq_ignore_ascii_case(requested_type))
}

fn lesson_type_name(lesson_type: &LessonType) -> &'static str {
    match lesson_type {
        LessonType::Error => "error",
        LessonType::Skill => "skill",
        LessonType::Pattern => "pattern",
        LessonType::Insight => "insight",
    }
}

fn count_occurrences_case_insensitive(content: &str, query_lower: &str) -> usize {
    content.to_lowercase().match_indices(query_lower).count()
}

fn preview_matching_lines(content: &str, query: &str, query_lower: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.to_lowercase().contains(query_lower))
        .take(2)
        .map(|line| highlight_query(line.trim(), query))
        .collect()
}

fn highlight_query(line: &str, query: &str) -> String {
    if line.is_empty() || query.is_empty() {
        return line.to_string();
    }

    let query_lower = query.to_lowercase();
    let boundaries: Vec<usize> = line
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(line.len()))
        .collect();
    let query_chars = query.chars().count();

    if query_chars == 0 {
        return line.to_string();
    }

    let mut result = String::new();
    let mut last_byte = 0;
    let mut boundary_index = 0;

    while boundary_index + query_chars < boundaries.len() {
        let start = boundaries[boundary_index];
        let end = boundaries[boundary_index + query_chars];
        let candidate = &line[start..end];

        if candidate.to_lowercase() == query_lower {
            result.push_str(&line[last_byte..start]);
            result.push_str("**");
            result.push_str(candidate);
            result.push_str("**");
            last_byte = end;
            boundary_index += query_chars;
        } else {
            boundary_index += 1;
        }
    }

    result.push_str(&line[last_byte..]);
    result
}

fn relative_path(knowledge_home: &Path, path: &Path) -> String {
    path.strip_prefix(knowledge_home)
        .ok()
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|| path.display().to_string())
}
