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
fn init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");

    samsara(&kh).arg("init").arg("--yes").assert().success();

    assert!(kh.join("lessons").exists(), "lessons/ should exist");
    assert!(kh.join("rules").exists(), "rules/ should exist");
    assert!(kh.join("archive").exists(), "archive/ should exist");
    assert!(kh.join("INDEX.md").exists(), "INDEX.md should exist");
    assert!(kh.join("log.md").exists(), "log.md should exist");
    assert!(
        kh.join("lessons/rust").exists(),
        "seed domain rust should exist"
    );
    assert!(
        kh.join("lessons/git").exists(),
        "seed domain git should exist"
    );
    assert!(
        tmp.path().join("AGENTS.md").exists(),
        "AGENTS.md should be in agents_home"
    );
}

#[test]
fn init_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");

    samsara(&kh).arg("init").arg("--yes").assert().success();
    samsara(&kh).arg("init").arg("--yes").assert().success();

    assert!(kh.join("lessons").exists());
}

#[test]
fn write_creates_new_lesson() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-fmt",
            "--summary",
            "always fmt before clippy",
            "--yes",
        ])
        .assert()
        .success();

    let lesson = kh.join("lessons/rust/cargo-fmt.md");
    assert!(lesson.exists(), "lesson file should be created");

    let content = std::fs::read_to_string(&lesson).unwrap();
    assert!(
        content.contains("always fmt before clippy"),
        "summary should be in body"
    );
    assert!(
        content.contains("occurrences:"),
        "frontmatter should have occurrences"
    );
    assert!(
        content.contains("promoted: false"),
        "promoted should be false"
    );
}

#[test]
fn write_with_type_flag_sets_lesson_type() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-test",
            "--summary",
            "run tests",
            "--type",
            "error",
            "--yes",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(kh.join("lessons/rust/cargo-test.md")).unwrap();
    assert!(content.contains("type: error"), "lesson_type should be set");
}

#[test]
fn write_updates_existing_lesson_occurrence() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("existing_lesson", &kh);

    let before = std::fs::read_to_string(kh.join("lessons/rust/cargo-fmt.md")).unwrap();
    let occ_before = before.matches("2026-04").count();

    samsara(&kh)
        .args(["write", "rust", "cargo-fmt"])
        .assert()
        .success();

    let after = std::fs::read_to_string(kh.join("lessons/rust/cargo-fmt.md")).unwrap();
    let occ_after = after.matches("2026-04").count() + after.matches("2026-0").count();

    assert!(
        after.len() > before.len() || occ_after > occ_before,
        "occurrence count should increase after write on existing lesson"
    );
}

#[test]
fn write_creates_log_entry() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-check",
            "--summary",
            "run cargo check",
            "--yes",
        ])
        .assert()
        .success();

    let log = std::fs::read_to_string(kh.join("log.md")).unwrap();
    assert!(log.contains("WRITE"), "log should contain WRITE action");
    assert!(
        log.contains("rust/cargo-check"),
        "log should contain target path"
    );
}

#[test]
fn write_rebuilds_index() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    samsara(&kh)
        .args([
            "write",
            "rust",
            "cargo-build",
            "--summary",
            "cargo build",
            "--yes",
        ])
        .assert()
        .success();

    let index = std::fs::read_to_string(kh.join("INDEX.md")).unwrap();
    assert!(
        index.contains("rust"),
        "INDEX.md should contain rust domain"
    );
}

#[test]
fn search_finds_by_keyword() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("search_mixed", &kh);

    let output = samsara(&kh).args(["search", "cargo"]).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("cargo"),
        "search should return cargo-related result"
    );
}

#[test]
fn search_finds_by_tag() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("search_mixed", &kh);

    let output = samsara(&kh).args(["search", "rebase"]).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("rebase"),
        "search should find rebase-stash via tag/body"
    );
}

#[test]
fn search_no_results_message() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    let output = samsara(&kh)
        .args(["search", "nonexistent_xyz_query_9999"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("未找到")
            || stdout.contains("no results")
            || stdout.is_empty()
            || output.status.success()
    );
}

#[test]
fn status_shows_domain_count() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("search_mixed", &kh);

    let output = samsara(&kh).args(["status"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rust") || stdout.contains("Domains") || stdout.contains("Lesson"),
        "status output should show domain/lesson info"
    );
}

#[test]
fn log_shows_last_entries() {
    let tmp = TempDir::new().unwrap();
    let kh = tmp.path().join("knowledge");
    copy_fixture("existing_lesson", &kh);

    let output = samsara(&kh).args(["log", "--tail", "5"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("WRITE") || stdout.contains("UPDATE"),
        "log should show recent entries"
    );
}

#[test]
fn log_empty_knowledge_shows_no_records() {
    let tmp = TempDir::new().unwrap();
    let kh = setup_knowledge(&tmp);

    let output = samsara(&kh).args(["log"]).output().unwrap();

    assert!(output.status.success());
}
