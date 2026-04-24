use crate::error::SamsaraError;
use std::path::Path;
use std::process::Command;

pub fn auto_commit(knowledge_home: &Path, message: &str) -> Result<(), SamsaraError> {
    let dir = knowledge_home.to_string_lossy();

    let is_repo = Command::new("git")
        .args(["-C", &dir, "rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_repo {
        eprintln!("⚠️  knowledge/ 不是 git 仓库，跳过自动提交。运行 `samsara init` 初始化。");
        return Ok(());
    }

    let add_ok = Command::new("git")
        .args(["-C", &dir, "add", "-A"])
        .status()
        .map_err(|e| SamsaraError::GitNotFound(e.to_string()))?
        .success();

    if !add_ok {
        return Err(SamsaraError::GitFailed);
    }

    // commit 可能因"nothing to commit"退出码非零 → 忽略退出码
    Command::new("git")
        .args(["-C", &dir, "commit", "-m", message])
        .status()
        .map_err(|e| SamsaraError::GitNotFound(e.to_string()))?;

    Ok(())
}

pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["-C", &path.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn uncommitted_files(knowledge_home: &Path) -> Vec<String> {
    Command::new("git")
        .args(["-C", &knowledge_home.to_string_lossy(), "status", "--short"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}
