use crate::{cli::SelfUpdateArgs, config::Config, error::SamsaraError};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/mocikadev/mocika-samsara/releases/latest";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SemVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

pub fn run(args: SelfUpdateArgs, _config: &Config) -> Result<(), SamsaraError> {
    if cfg!(target_os = "windows") {
        println!("⚠️  Windows 暂不支持自动升级，请手动下载");
        return Ok(());
    }

    let client = Client::builder()
        .user_agent(format!("samsara/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()?;
    let release = fetch_latest_release(&client)?;
    let current_version = SemVersion::parse(env!("CARGO_PKG_VERSION"))?;
    let latest_version = SemVersion::parse(&release.tag_name)?;

    if args.check {
        if latest_version > current_version {
            println!(
                "📦 发现新版本：{}（当前 {}）",
                release.tag_name,
                env!("CARGO_PKG_VERSION")
            );
        } else {
            println!("✅ 当前已是最新版本：{}", env!("CARGO_PKG_VERSION"));
        }
        return Ok(());
    }

    if latest_version <= current_version {
        println!("✅ 当前已是最新版本：{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let asset_name = asset_name_for_current_target()?;
    let archive_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| SamsaraError::UpdateError(format!("未找到发布资产：{asset_name}")))?;
    let checksum_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS.txt")
        .ok_or_else(|| SamsaraError::UpdateError("未找到 SHA256SUMS.txt".to_string()))?;

    let binary_bytes = download_bytes(&client, &archive_asset.browser_download_url)?;
    let checksum_text = download_text(&client, &checksum_asset.browser_download_url)?;
    verify_checksum(&binary_bytes, &checksum_text, asset_name)?;

    let exe_path = std::env::current_exe()?;
    let temp_dir = prepare_temp_dir(&exe_path)?;
    let temp_binary = temp_dir.join(asset_name);
    fs::write(&temp_binary, &binary_bytes)?;
    replace_current_exe(&temp_binary, &exe_path)?;
    let _ = fs::remove_dir_all(&temp_dir);

    println!("✅ samsara 已升级到 {}", release.tag_name);
    Ok(())
}

impl SemVersion {
    fn parse(raw: &str) -> Result<Self, SamsaraError> {
        let normalized = raw
            .trim()
            .trim_start_matches('v')
            .split_once('-')
            .map_or(raw.trim().trim_start_matches('v'), |(base, _)| base)
            .split_once('+')
            .map_or(raw.trim().trim_start_matches('v'), |(base, _)| base);

        let mut parts = normalized.split('.');
        let major = parse_version_part(parts.next(), raw, "major")?;
        let minor = parse_version_part(parts.next(), raw, "minor")?;
        let patch = parse_version_part(parts.next(), raw, "patch")?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

fn parse_version_part(part: Option<&str>, raw: &str, label: &str) -> Result<u64, SamsaraError> {
    let value = part.ok_or_else(|| SamsaraError::UpdateError(format!("无效版本号 {raw}")))?;
    value.parse::<u64>().map_err(|error| {
        SamsaraError::UpdateError(format!("解析版本号 {raw} 的 {label} 失败：{error}"))
    })
}

fn fetch_latest_release(client: &Client) -> Result<GitHubRelease, SamsaraError> {
    let response = client
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()?;
    let status = response.status();
    if !status.is_success() {
        return Err(SamsaraError::UpdateError(format!(
            "GitHub releases API 返回 {status}"
        )));
    }

    response.json().map_err(Into::into)
}

fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>, SamsaraError> {
    let response = client.get(url).send()?;
    let status = response.status();
    if !status.is_success() {
        return Err(SamsaraError::UpdateError(format!(
            "下载资产失败：{url} 返回 {status}"
        )));
    }

    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(Into::into)
}

fn download_text(client: &Client, url: &str) -> Result<String, SamsaraError> {
    let response = client.get(url).send()?;
    let status = response.status();
    if !status.is_success() {
        return Err(SamsaraError::UpdateError(format!(
            "下载校验文件失败：{url} 返回 {status}"
        )));
    }

    response.text().map_err(Into::into)
}

fn asset_name_for_current_target() -> Result<&'static str, SamsaraError> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok("samsara-linux-amd64"),
        ("aarch64", "linux") => Ok("samsara-linux-arm64"),
        ("x86_64", "macos") => Ok("samsara-macos-amd64"),
        ("aarch64", "macos") => Ok("samsara-macos-arm64"),
        (arch, os) => Err(SamsaraError::UpdateError(format!(
            "暂不支持自动升级的目标平台：{arch}-{os}"
        ))),
    }
}

fn verify_checksum(
    archive_bytes: &[u8],
    checksum_text: &str,
    asset_name: &str,
) -> Result<(), SamsaraError> {
    let expected = checksum_text
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file_name = parts.last()?;
            (file_name == asset_name).then_some(hash.to_string())
        })
        .ok_or_else(|| {
            SamsaraError::UpdateError(format!("SHA256SUMS.txt 缺少 {asset_name} 校验值"))
        })?;

    let actual = format!("{:x}", Sha256::digest(archive_bytes));
    if actual != expected {
        return Err(SamsaraError::UpdateError(format!(
            "SHA256 校验失败：expected {expected}, got {actual}"
        )));
    }

    Ok(())
}

fn prepare_temp_dir(exe_path: &Path) -> Result<PathBuf, SamsaraError> {
    let parent = exe_path.parent().ok_or_else(|| {
        SamsaraError::UpdateError(format!("无法确定可执行文件目录：{}", exe_path.display()))
    })?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temp_dir = parent.join(format!(".samsara-update-{stamp}"));
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

fn replace_current_exe(new_binary: &Path, exe_path: &Path) -> Result<(), SamsaraError> {
    let temp_target = exe_path.with_extension("tmp");
    fs::copy(new_binary, &temp_target)?;
    let permissions = fs::metadata(new_binary)?.permissions();
    fs::set_permissions(&temp_target, permissions)?;
    fs::rename(&temp_target, exe_path)?;
    Ok(())
}
