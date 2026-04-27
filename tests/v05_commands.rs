use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

fn samsara(home: &Path, fake_home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("samsara").unwrap();
    cmd.env("SAMSARA_HOME", home);
    cmd.env("HOME", fake_home);
    cmd.env("GIT_AUTHOR_NAME", "test");
    cmd.env("GIT_AUTHOR_EMAIL", "test@test.com");
    cmd.env("GIT_COMMITTER_NAME", "test");
    cmd.env("GIT_COMMITTER_EMAIL", "test@test.com");
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

#[test]
fn init_injects_opencode_mcp_config() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    let kh = fake_home.join("knowledge");

    let opencode_dir = fake_home.join(".config").join("opencode");
    std::fs::create_dir_all(&opencode_dir).unwrap();
    std::fs::write(
        opencode_dir.join("opencode.json"),
        r#"{"$schema":"https://opencode.ai/config.json"}"#,
    )
    .unwrap();

    samsara(&kh, &fake_home)
        .arg("init")
        .arg("--yes")
        .assert()
        .success();

    let content = std::fs::read_to_string(opencode_dir.join("opencode.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        v["mcp"]["samsara"]["type"], "local",
        "opencode.json should contain samsara MCP entry"
    );
    assert_eq!(
        v["mcp"]["samsara"]["command"],
        serde_json::json!(["samsara", "mcp", "serve"])
    );
}

#[test]
fn init_mcp_injection_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    let kh = fake_home.join("knowledge");

    let opencode_dir = fake_home.join(".config").join("opencode");
    std::fs::create_dir_all(&opencode_dir).unwrap();
    std::fs::write(
        opencode_dir.join("opencode.json"),
        r#"{"$schema":"https://opencode.ai/config.json"}"#,
    )
    .unwrap();

    samsara(&kh, &fake_home)
        .arg("init")
        .arg("--yes")
        .assert()
        .success();
    samsara(&kh, &fake_home)
        .arg("init")
        .arg("--yes")
        .assert()
        .success();

    let content = std::fs::read_to_string(opencode_dir.join("opencode.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    let mcp_obj = v["mcp"].as_object().unwrap();
    assert_eq!(
        mcp_obj.keys().filter(|k| *k == "samsara").count(),
        1,
        "samsara key should appear exactly once"
    );
}

#[test]
fn init_injects_claude_mcp_config() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    let kh = fake_home.join("knowledge");

    let claude_dir = fake_home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    samsara(&kh, &fake_home)
        .arg("init")
        .arg("--yes")
        .assert()
        .success();

    let config_path = claude_dir.join("claude_desktop_config.json");
    assert!(
        config_path.exists(),
        "claude_desktop_config.json should be created"
    );

    let content = std::fs::read_to_string(&config_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(v["mcpServers"]["samsara"]["command"], "samsara");
    assert_eq!(
        v["mcpServers"]["samsara"]["args"],
        serde_json::json!(["mcp", "serve"])
    );
}

#[test]
fn init_mcp_skips_when_tool_dir_absent() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    let kh = fake_home.join("knowledge");
    std::fs::create_dir_all(&fake_home).unwrap();

    samsara(&kh, &fake_home)
        .arg("init")
        .arg("--yes")
        .assert()
        .success();

    assert!(
        !fake_home.join(".config/opencode/opencode.json").exists(),
        "should not create opencode.json when opencode dir is absent"
    );
    assert!(
        !fake_home
            .join(".claude/claude_desktop_config.json")
            .exists(),
        "should not create claude config when .claude dir is absent"
    );
}
