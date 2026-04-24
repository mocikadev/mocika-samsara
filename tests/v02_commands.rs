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
fn domain_list_shows_init_domains() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");

    samsara(&kh).args(["init", "--yes"]).assert().success();

    let output = samsara(&kh).args(["domain", "list"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rust") || stdout.contains("git"),
        "domain list should show seed domains"
    );
}

#[test]
fn domain_add_creates_directory() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["domain", "add", "myapp"])
        .assert()
        .success();

    assert!(
        kh.join("lessons/myapp").is_dir(),
        "lessons/myapp/ should be created"
    );
}

#[test]
fn domain_add_duplicate_shows_warning() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    std::fs::create_dir_all(kh.join("lessons/rust")).unwrap();

    let output = samsara(&kh)
        .args(["domain", "add", "rust"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("已存在"),
        "should warn that domain already exists"
    );
}

#[test]
fn skill_note_records_use_in_log() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args(["skill-note", "rust-skills"])
        .assert()
        .success();

    let log = std::fs::read_to_string(kh.join("log.md")).unwrap();
    assert!(
        log.contains("SKILL_USE"),
        "log should contain SKILL_USE action"
    );
    assert!(log.contains("rust-skills"), "log should contain skill name");
}

#[test]
fn skill_note_fail_records_in_log() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "skill-note",
            "broken-skill",
            "--fail",
            "--note",
            "tool crashed",
        ])
        .assert()
        .success();

    let log = std::fs::read_to_string(kh.join("log.md")).unwrap();
    assert!(
        log.contains("SKILL_FAIL"),
        "log should contain SKILL_FAIL action"
    );
}

#[test]
fn promote_fails_with_insufficient_occurrences() {
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
        .args(["promote", "rust", "cargo-fmt"])
        .assert()
        .failure();
}

#[test]
fn promote_succeeds_with_enough_occurrences() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    samsara(&kh)
        .args(["promote", "rust", "cargo-fmt"])
        .assert()
        .success();

    assert!(
        kh.join("rules/rust.md").is_file(),
        "rules/rust.md should be created after promote"
    );

    let rules = std::fs::read_to_string(kh.join("rules/rust.md")).unwrap();
    assert!(
        rules.contains("cargo-fmt"),
        "rules file should contain promoted keyword"
    );
}

#[test]
fn promote_sets_promoted_flag_in_lesson() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    samsara(&kh)
        .args(["promote", "rust", "cargo-fmt"])
        .assert()
        .success();

    let lesson = std::fs::read_to_string(kh.join("lessons/rust/cargo-fmt.md")).unwrap();
    assert!(
        lesson.contains("promoted: true"),
        "lesson frontmatter should have promoted: true after promote"
    );
}

#[test]
fn promote_layer0_writes_to_agents_md() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    let agents_md = tmp.path().join("AGENTS.md");
    std::fs::write(&agents_md, "# Rules\n").unwrap();

    samsara(&kh)
        .args(["promote", "rust", "cargo-fmt", "--layer0", "--yes"])
        .assert()
        .success();

    let content = std::fs::read_to_string(&agents_md).unwrap();
    assert!(
        content.contains("cargo-fmt"),
        "AGENTS.md should contain promoted rule"
    );
}

#[test]
fn lint_exits_ok_on_clean_knowledge() {
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

    samsara(&kh).args(["lint"]).assert().success();
}

#[test]
fn lint_dry_run_does_not_modify_files() {
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

    let before = std::fs::read_to_string(kh.join("lessons/rust/cargo-fmt.md")).unwrap();
    samsara(&kh).args(["lint", "--dry-run"]).assert().success();
    let after = std::fs::read_to_string(kh.join("lessons/rust/cargo-fmt.md")).unwrap();

    assert_eq!(
        before, after,
        "lint --dry-run should not modify lesson files"
    );
}

#[test]
fn reflect_runs_on_empty_knowledge() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh).args(["reflect"]).assert().success();
}

#[test]
fn reflect_runs_on_existing_knowledge() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("promote_ready", &kh);

    let agents_md = tmp.path().join("AGENTS.md");
    std::fs::write(&agents_md, "# Rules\n").unwrap();

    let output = samsara(&kh).args(["reflect"]).output().unwrap();

    assert!(output.status.success());
}
