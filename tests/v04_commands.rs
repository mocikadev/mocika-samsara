use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn samsara(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("samsara").unwrap();
    cmd.env("SAMSARA_HOME", home);
    cmd.env("GIT_AUTHOR_NAME", "test");
    cmd.env("GIT_AUTHOR_EMAIL", "test@test.com");
    cmd.env("GIT_COMMITTER_NAME", "test");
    cmd.env("GIT_COMMITTER_EMAIL", "test@test.com");
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

fn setup_knowledge(tmp: &TempDir) -> std::path::PathBuf {
    let kh = tmp.path().join("knowledge");
    std::fs::create_dir_all(kh.join("lessons")).unwrap();
    std::fs::create_dir_all(kh.join("rules")).unwrap();
    std::fs::create_dir_all(kh.join("archive")).unwrap();
    kh
}

#[test]
fn remote_show_without_remote_prints_hint_and_exits_ok() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["remote", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("未配置").or(predicate::str::contains("当前未配置")));
}

#[test]
fn remote_add_writes_remote_url_to_samsara_toml() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    let remote_url = "https://example.com/mocika-samsara.git";

    samsara(&kh).args(["init", "--yes"]).assert().success();
    samsara(&kh)
        .args(["remote", "add", remote_url])
        .assert()
        .success();

    let toml = std::fs::read_to_string(tmp.path().join("samsara.toml")).unwrap();
    assert!(toml.contains("[sync]"));
    assert!(toml.contains(remote_url));
}

#[test]
fn remote_set_updates_remote_url_in_samsara_toml() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    let old_url = "https://example.com/old.git";
    let new_url = "https://example.com/new.git";

    samsara(&kh).args(["init", "--yes"]).assert().success();
    samsara(&kh)
        .args(["remote", "add", old_url])
        .assert()
        .success();
    samsara(&kh)
        .args(["remote", "set", new_url])
        .assert()
        .success();

    let toml = std::fs::read_to_string(tmp.path().join("samsara.toml")).unwrap();
    assert!(toml.contains(new_url));
    assert!(!toml.contains(old_url));
}

#[test]
fn push_dry_run_exits_ok_without_git_side_effects() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["--dry-run", "push"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY-RUN"));

    assert!(
        !kh.join(".git").exists(),
        "dry-run should not create git repo"
    );
}

#[test]
fn pull_dry_run_exits_ok_without_merge() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["--dry-run", "pull"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY-RUN"));
}

#[test]
fn self_update_check_does_not_panic() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    let output = samsara(&kh)
        .args(["self-update", "--check"])
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(101), "command should not panic");
}

#[test]
fn mcp_serve_help_is_available() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["mcp", "serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stdio").or(predicate::str::contains("port")));
}
