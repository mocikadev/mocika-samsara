use assert_cmd::Command;
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

fn copy_fixture(src_name: &str, dest: &Path) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(src_name);
    copy_dir_all(&fixtures, dest).unwrap();
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[test]
fn archive_moves_lesson_to_archive_dir() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-fmt",
            "--summary",
            "fmt before commit",
            "--yes",
        ])
        .assert()
        .success();

    samsara(&kh)
        .args(["archive", "rust", "cargo-fmt"])
        .assert()
        .success();

    assert!(
        !kh.join("lessons/rust/cargo-fmt.md").exists(),
        "original lesson should be removed"
    );
    assert!(
        kh.join("archive/rust/cargo-fmt.md").exists(),
        "lesson should be in archive/"
    );
}

#[test]
fn archive_logs_archive_action() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-fmt",
            "--summary",
            "fmt before commit",
            "--yes",
        ])
        .assert()
        .success();

    samsara(&kh)
        .args(["archive", "rust", "cargo-fmt"])
        .assert()
        .success();

    let log = std::fs::read_to_string(kh.join("log.md")).unwrap();
    assert!(log.contains("ARCHIVE"), "log should contain ARCHIVE action");
}

#[test]
fn archive_dry_run_does_not_move_file() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-fmt",
            "--summary",
            "fmt before commit",
            "--yes",
        ])
        .assert()
        .success();

    samsara(&kh)
        .args(["--dry-run", "archive", "rust", "cargo-fmt"])
        .assert()
        .success();

    assert!(
        kh.join("lessons/rust/cargo-fmt.md").exists(),
        "lesson should still exist after dry-run"
    );
}

#[test]
fn archive_fails_for_nonexistent_lesson() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["archive", "rust", "nonexistent"])
        .assert()
        .failure();
}

#[test]
fn demote_removes_rule_from_agents_md() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    let agents_md = tmp.path().join("AGENTS.md");
    std::fs::write(&agents_md, "# Rules\n- rust/cargo-fmt: fmt before commit\n").unwrap();

    samsara(&kh)
        .args(["demote", "cargo-fmt", "--yes"])
        .assert()
        .success();

    let content = std::fs::read_to_string(&agents_md).unwrap();
    assert!(
        !content.contains("cargo-fmt"),
        "AGENTS.md should not contain demoted rule"
    );
}

#[test]
fn demote_dry_run_does_not_modify_agents_md() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    let agents_md = tmp.path().join("AGENTS.md");
    std::fs::write(&agents_md, "# Rules\n- rust/cargo-fmt: fmt before commit\n").unwrap();

    samsara(&kh)
        .args(["--dry-run", "demote", "cargo-fmt", "--yes"])
        .assert()
        .success();

    let content = std::fs::read_to_string(&agents_md).unwrap();
    assert!(
        content.contains("cargo-fmt"),
        "AGENTS.md should be unchanged after dry-run"
    );
}

#[test]
fn demote_no_match_exits_ok() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    let agents_md = tmp.path().join("AGENTS.md");
    std::fs::write(&agents_md, "# Rules\n").unwrap();

    let output = samsara(&kh)
        .args(["demote", "nonexistent_pattern_xyz", "--yes"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("未找到"), "should warn about no match");
}

#[test]
fn prime_outputs_rules_with_enough_occurrences() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    samsara(&kh)
        .args(["promote", "rust", "cargo-fmt"])
        .assert()
        .success();

    let output = samsara(&kh).args(["prime"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo-fmt") || stdout.contains("rust"),
        "prime should list promoted rules"
    );
}

#[test]
fn prime_empty_knowledge_shows_no_candidates() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    let output = samsara(&kh).args(["prime"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("暂无"),
        "prime on empty knowledge should say no candidates"
    );
}

#[test]
fn log_rotate_no_op_on_fresh_log() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-fmt",
            "--summary",
            "fmt before commit",
            "--yes",
        ])
        .assert()
        .success();

    let output = samsara(&kh)
        .args(["log", "--rotate", "--keep", "90"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("无需轮转") || stdout.contains("已归档"),
        "log rotate should report status"
    );
}
