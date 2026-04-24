use crate::{
    cli::{RemoteAction, RemoteArgs},
    config::Config,
    error::SamsaraError,
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    process::{Command, Output},
};

#[derive(Debug, Default, Deserialize, Serialize)]
struct SamsaraToml {
    #[serde(default)]
    sync: SyncConfig,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct SyncConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_url: Option<String>,
}

pub fn run(args: RemoteArgs, config: &Config) -> Result<(), SamsaraError> {
    match args.action {
        RemoteAction::Add { url } => add_remote(&url, config),
        RemoteAction::Set { url } => set_remote(&url, config),
        RemoteAction::Show => show_remote(config),
    }
}

fn add_remote(url: &str, config: &Config) -> Result<(), SamsaraError> {
    if config.dry_run {
        println!(
            "DRY-RUN: git -C {} remote add origin {url}",
            config.knowledge_home.display()
        );
        println!(
            "DRY-RUN: 更新 {} [sync].remote_url = {url}",
            config.agents_home.join("samsara.toml").display()
        );
        return Ok(());
    }

    run_git_remote(&config.knowledge_home, &["remote", "add", "origin", url])?;
    write_remote_url(&config.agents_home.join("samsara.toml"), url)?;

    println!("✅ 已设置远端 origin: {url}");
    println!("提示：可用 `samsara push` 推送 / `samsara pull` 拉取");
    Ok(())
}

fn set_remote(url: &str, config: &Config) -> Result<(), SamsaraError> {
    if config.dry_run {
        println!(
            "DRY-RUN: git -C {} remote set-url origin {url}",
            config.knowledge_home.display()
        );
        println!(
            "DRY-RUN: 更新 {} [sync].remote_url = {url}",
            config.agents_home.join("samsara.toml").display()
        );
        return Ok(());
    }

    run_git_remote(
        &config.knowledge_home,
        &["remote", "set-url", "origin", url],
    )?;
    write_remote_url(&config.agents_home.join("samsara.toml"), url)?;

    println!("✅ 已更新远端 origin: {url}");
    Ok(())
}

fn show_remote(config: &Config) -> Result<(), SamsaraError> {
    match git_output(&config.knowledge_home, &["remote", "-v"]) {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                println!("当前未配置 git remote origin");
            } else {
                print!("{stdout}");
                if !stdout.ends_with('\n') {
                    println!();
                }
            }
        }
        Ok(_) | Err(_) => {
            println!("当前未配置 git remote origin");
        }
    }

    match read_remote_url(&config.agents_home.join("samsara.toml"))? {
        Some(url) => println!("samsara.toml [sync].remote_url = {url}"),
        None => println!("samsara.toml 未配置 remote_url"),
    }

    Ok(())
}

fn run_git_remote(knowledge_home: &Path, args: &[&str]) -> Result<(), SamsaraError> {
    let output = git_output(knowledge_home, args)
        .map_err(|error| SamsaraError::RemoteFailed(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    Err(SamsaraError::RemoteFailed(command_error(&output)))
}

fn git_output(knowledge_home: &Path, args: &[&str]) -> Result<Output, SamsaraError> {
    Command::new("git")
        .arg("-C")
        .arg(knowledge_home)
        .args(args)
        .output()
        .map_err(|error| SamsaraError::GitNotFound(error.to_string()))
}

fn read_remote_url(path: &Path) -> Result<Option<String>, SamsaraError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path).map_err(|error| {
        SamsaraError::RemoteFailed(format!("读取 {} 失败：{error}", path.display()))
    })?;
    let config: SamsaraToml = toml::from_str(&content).map_err(|error| {
        SamsaraError::RemoteFailed(format!("解析 {} 失败：{error}", path.display()))
    })?;
    Ok(config.sync.remote_url)
}

fn write_remote_url(path: &Path, url: &str) -> Result<(), SamsaraError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            SamsaraError::RemoteFailed(format!("创建 {} 失败：{error}", parent.display()))
        })?;
    }

    let mut config = if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|error| {
            SamsaraError::RemoteFailed(format!("读取 {} 失败：{error}", path.display()))
        })?;
        toml::from_str::<SamsaraToml>(&content).map_err(|error| {
            SamsaraError::RemoteFailed(format!("解析 {} 失败：{error}", path.display()))
        })?
    } else {
        SamsaraToml::default()
    };

    config.sync.remote_url = Some(url.to_string());
    let rendered = toml::to_string_pretty(&config).map_err(|error| {
        SamsaraError::RemoteFailed(format!("序列化 {} 失败：{error}", path.display()))
    })?;
    std::fs::write(path, rendered).map_err(|error| {
        SamsaraError::RemoteFailed(format!("写入 {} 失败：{error}", path.display()))
    })?;
    Ok(())
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
