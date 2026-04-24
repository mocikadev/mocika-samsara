use crate::error::SamsaraError;
use std::path::Path;

pub fn append_to_rules(
    knowledge_home: &Path,
    domain: &str,
    keyword: &str,
    content: &str,
    occurrences: usize,
    dry_run: bool,
) -> Result<(), SamsaraError> {
    let rules_dir = knowledge_home.join("rules");
    let rules_path = rules_dir.join(format!("{domain}.md"));
    let mut rendered = String::new();

    if rules_path.exists() {
        rendered = std::fs::read_to_string(&rules_path)?;
    } else {
        rendered.push_str(&format!("# {domain} rules\n\n"));
    }

    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }

    rendered.push_str(&format!(
        "## {keyword}\n来源：lessons/{domain}/{keyword}.md（occurrences: {occurrences}）\n{content}\n---\n"
    ));

    if dry_run {
        println!(
            "将在 {} 追加：\n## {keyword}\n来源：lessons/{domain}/{keyword}.md（occurrences: {occurrences}）\n{content}\n---",
            rules_path.display()
        );
        return Ok(());
    }

    std::fs::create_dir_all(&rules_dir)?;
    std::fs::write(&rules_path, rendered)?;
    Ok(())
}
