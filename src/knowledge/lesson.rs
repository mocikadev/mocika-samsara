use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::SamsaraError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LessonType {
    Error,
    Skill,
    Pattern,
    Insight,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LessonFrontmatter {
    pub date: NaiveDate,
    pub domain: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub lesson_type: Option<LessonType>,
    pub tags: Vec<String>,
    pub occurrences: Vec<NaiveDate>,
    pub promoted: bool,
    #[serde(default)]
    pub verified: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts_with: Option<Vec<String>>,
}

pub struct Lesson {
    pub frontmatter: LessonFrontmatter,
    pub body: String,
    pub path: PathBuf,
}

impl Lesson {
    pub fn load(path: &Path) -> Result<Self, SamsaraError> {
        let content = std::fs::read_to_string(path)?;
        let (frontmatter, body) = parse_frontmatter(&content)?;
        Ok(Lesson {
            frontmatter,
            body,
            path: path.to_path_buf(),
        })
    }

    pub fn add_occurrence(&mut self, date: NaiveDate) {
        self.frontmatter.occurrences.push(date);
    }

    pub fn should_promote(&self) -> bool {
        self.frontmatter.occurrences.len() >= 3 && !self.frontmatter.promoted
    }

    pub fn is_expired(&self, today: NaiveDate) -> bool {
        self.frontmatter
            .valid_until
            .map(|d| d < today)
            .unwrap_or(false)
    }

    pub fn save(&self, dry_run: bool) -> Result<(), SamsaraError> {
        if dry_run {
            return Ok(());
        }
        let fm_yaml = serde_yaml::to_string(&self.frontmatter)?;
        let content = format!("---\n{}---\n{}", fm_yaml, self.body);
        atomic_write(&self.path, &content)?;
        Ok(())
    }
}

pub fn find_lesson(knowledge_home: &Path, domain: &str, keyword: &str) -> Option<PathBuf> {
    let path = knowledge_home
        .join("lessons")
        .join(domain)
        .join(format!("{keyword}.md"));
    path.exists().then_some(path)
}

fn parse_frontmatter(content: &str) -> Result<(LessonFrontmatter, String), SamsaraError> {
    let rest = content
        .strip_prefix("---\n")
        .ok_or_else(|| SamsaraError::FrontmatterParse("缺少开头的 ---".to_string()))?;
    let end = rest
        .find("\n---\n")
        .ok_or_else(|| SamsaraError::FrontmatterParse("缺少结尾的 ---".to_string()))?;
    let yaml = &rest[..end];
    let body = rest[end + 5..].to_string();
    let fm: LessonFrontmatter = serde_yaml::from_str(yaml)?;
    Ok((fm, body))
}

fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
