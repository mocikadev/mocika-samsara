use std::{fs, io, path::Path};

use crate::{
    cli::{DomainAction, DomainArgs},
    config::Config,
    error::SamsaraError,
};

pub fn run(args: DomainArgs, config: &Config) -> Result<(), SamsaraError> {
    match args.action {
        DomainAction::List => list_domains(&config.knowledge_home),
        DomainAction::Add { name } => add_domain(&config.knowledge_home, &name),
    }
}

fn list_domains(knowledge_home: &Path) -> Result<(), SamsaraError> {
    let lessons_root = knowledge_home.join("lessons");
    let mut domains = Vec::new();

    if lessons_root.exists() {
        for entry in fs::read_dir(&lessons_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let domain = entry.file_name().to_string_lossy().into_owned();
            let lesson_count = count_lessons(&entry.path())?;
            let has_rules = knowledge_home
                .join("rules")
                .join(format!("{domain}.md"))
                .is_file();

            domains.push((domain, lesson_count, has_rules));
        }
    }

    domains.sort_by(|left, right| left.0.cmp(&right.0));

    if domains.is_empty() {
        println!("（暂无 domain，运行 samsara init 初始化）");
        return Ok(());
    }

    for (domain, lesson_count, has_rules) in domains {
        if has_rules {
            println!("{domain}  {lesson_count} lessons  [rules]");
        } else {
            println!("{domain}  {lesson_count} lessons");
        }
    }

    Ok(())
}

fn add_domain(knowledge_home: &Path, name: &str) -> Result<(), SamsaraError> {
    validate_domain_name(name)?;

    let domain_dir = knowledge_home.join("lessons").join(name);
    if domain_dir.exists() {
        println!("⚠️  domain '{name}' 已存在");
        return Ok(());
    }

    fs::create_dir_all(&domain_dir)?;
    println!("✅ domain '{name}' 已注册（lessons/{name}/）");
    Ok(())
}

fn count_lessons(domain_dir: &Path) -> Result<usize, SamsaraError> {
    let mut count = 0;

    for entry in fs::read_dir(domain_dir)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("md")
        {
            count += 1;
        }
    }

    Ok(count)
}

fn validate_domain_name(name: &str) -> Result<(), SamsaraError> {
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.chars().any(char::is_whitespace);

    if invalid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("非法 domain 名称: {name}"),
        )
        .into());
    }

    Ok(())
}
