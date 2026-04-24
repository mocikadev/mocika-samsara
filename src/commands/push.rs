use crate::{config::Config, error::SamsaraError};
use chrono::Local;
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn run(config: &Config) -> Result<(), SamsaraError> {
    if config.dry_run {
        let date = Local::now().date_naive();
        println!("DRY-RUN: git -C {} add -A", config.knowledge_home.display());
        println!(
            "DRY-RUN: git -C {} commit -m \"samsara: sync {date}\" --allow-empty",
            config.knowledge_home.display()
        );
        println!(
            "DRY-RUN: git -C {} push origin main",
            config.knowledge_home.display()
        );
        return Ok(());
    }

    run_git_step(&config.knowledge_home, &["add", "-A"])?;

    let commit_message = format!("samsara: sync {}", Local::now().date_naive());
    run_git_step(
        &config.knowledge_home,
        &["commit", "-m", commit_message.as_str(), "--allow-empty"],
    )?;
    run_git_step(&config.knowledge_home, &["push", "origin", "main"])?;

    println!("✅ 推送成功");
    Ok(())
}

fn run_git_step(knowledge_home: &Path, args: &[&str]) -> Result<(), SamsaraError> {
    let output = git_output(knowledge_home, args).map_err(|error| {
        eprintln!("❌ 推送失败：{error}");
        SamsaraError::PushFailed(error.to_string())
    })?;

    if output.status.success() {
        return Ok(());
    }

    let message = command_error(&output);
    eprintln!("❌ 推送失败：{message}");
    Err(SamsaraError::PushFailed(message))
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
