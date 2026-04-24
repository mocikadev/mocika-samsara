use crate::{config::Config, error::SamsaraError, knowledge};
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn run(config: &Config) -> Result<(), SamsaraError> {
    if config.dry_run {
        println!(
            "DRY-RUN: git -C {} fetch origin",
            config.knowledge_home.display()
        );

        match git_output(&config.knowledge_home, &["fetch", "origin"]) {
            Ok(output) if output.status.success() => {
                let preview = git_output(
                    &config.knowledge_home,
                    &["log", "--oneline", "HEAD..origin/main"],
                );

                match preview {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.trim().is_empty() {
                            println!("DRY-RUN: origin/main 暂无可合并提交");
                        } else {
                            println!("DRY-RUN: 将合并以下提交：");
                            print!("{stdout}");
                            if !stdout.ends_with('\n') {
                                println!();
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("⚠️  无法生成 merge 预览：{error}");
                    }
                }
            }
            Ok(output) => {
                eprintln!("⚠️  fetch 预览失败：{}", command_error(&output));
            }
            Err(error) => {
                eprintln!("⚠️  fetch 预览失败：{error}");
            }
        }

        return Ok(());
    }

    let fetch_output = git_output(&config.knowledge_home, &["fetch", "origin"])?;
    if !fetch_output.status.success() {
        return Err(SamsaraError::UpdateError(format!(
            "git fetch origin 失败：{}",
            command_error(&fetch_output)
        )));
    }

    let merge_output = git_output(&config.knowledge_home, &["merge", "--no-ff", "origin/main"])?;
    if !merge_output.status.success() {
        let conflicts = conflict_files(&config.knowledge_home)?;
        eprintln!("❌ 检测到 merge 冲突，请手动解决后再运行 `samsara pull`");
        if conflicts.is_empty() {
            eprintln!("未能识别具体冲突文件：{}", command_error(&merge_output));
        } else {
            for file in &conflicts {
                eprintln!("- {file}");
            }
        }
        return Err(SamsaraError::PullConflict(conflicts));
    }

    knowledge::index::rebuild(&config.knowledge_home, false)?;
    println!("✅ 拉取并重建 INDEX 成功");
    Ok(())
}

fn conflict_files(knowledge_home: &Path) -> Result<Vec<String>, SamsaraError> {
    let output = git_output(knowledge_home, &["diff", "--name-only", "--diff-filter=U"])?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn git_output(knowledge_home: &Path, args: &[&str]) -> Result<Output, SamsaraError> {
    Command::new("git")
        .arg("-C")
        .arg(knowledge_home)
        .args(args)
        .output()
        .map_err(|error| SamsaraError::GitNotFound(error.to_string()))
}

fn command_error(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("git 退出码 {:?}", output.status.code())
}
