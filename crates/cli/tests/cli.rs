use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};
fn fake() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_BIN_EXE_ctxc"));
    p.set_file_name(if cfg!(windows) {
        "fake-codex.exe"
    } else {
        "fake-codex"
    });
    p
}
fn command(dir: &tempfile::TempDir, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ctxc"));
    c.current_dir(dir.path()).args(args).env(
        "CTXC_USER_CONFIG",
        dir.path().join("absent-user-config.toml"),
    );
    c
}
fn good(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
#[test]
fn plan_no_codex_and_no_state() {
    let d = tempfile::tempdir().unwrap();
    let result = good(
        command(
            &d,
            &["plan", "hello", "--json", "--codex-bin", "nonexistent"],
        )
        .output()
        .unwrap(),
    );
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["classification"], "chat");
    assert_eq!(v["blocks"].as_array().unwrap().len(), 0);
    assert!(!d.path().join(".ctxc").exists());
}
#[test]
fn dry_run_and_shorthand() {
    let d = tempfile::tempdir().unwrap();
    let result = good(
        command(&d, &["hello", "--dry-run", "--json"])
            .arg("--codex-bin")
            .arg(fake())
            .output()
            .unwrap(),
    );
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["compiled"]["classification"], "chat");
    assert_eq!(v["invocation"]["applied_reasoning"], "low");
    assert!(!d.path().join(".ctxc").exists());
}
#[test]
fn run_history_no_prompt_body() {
    let d = tempfile::tempdir().unwrap();
    let out = good(
        command(
            &d,
            &["run", "Fix auth.rs; PRIVATE_MARKER_123 Do NOT change API"],
        )
        .arg("--codex-bin")
        .arg(fake())
        .output()
        .unwrap(),
    );
    assert!(out.contains("fake complete"));
    let usage = good(command(&d, &["usage", "--json"]).output().unwrap());
    assert!(!usage.contains("PRIVATE_MARKER_123"));
    assert!(!usage.contains("auth.rs"));
    let v: Value = serde_json::from_str(&usage).unwrap();
    assert_eq!(v[0]["result"]["usage"]["input_tokens"], 42);
    assert!(v[0]["result"]["usage"]["reasoning_tokens"].is_null());
}
#[test]
fn malformed_missing_quota_and_unsupported() {
    for (scenario, code) in [
        ("malformed", 0),
        ("missing", 0),
        ("quota", 9),
        ("unsupported", 2),
    ] {
        let d = tempfile::tempdir().unwrap();
        let output = command(&d, &["run", "hello"])
            .arg("--codex-bin")
            .arg(fake())
            .env("CTXC_FAKE_SCENARIO", scenario)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(code));
        let usage = good(command(&d, &["usage", "--json"]).output().unwrap());
        let v: Value = serde_json::from_str(&usage).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 1);
        assert!(v[0]["result"]["usage"]["input_tokens"].is_null());
        if scenario == "malformed" {
            assert!(String::from_utf8_lossy(&output.stdout).contains("raw fallback evidence"));
        }
    }
}
#[test]
fn argument_roundtrip_exact_and_no_injection() {
    let d = tempfile::tempdir().unwrap();
    let prompt="--flag ; echo stolen > marker | $(whoami) `whoami` & \"quoted\" C:\\with spaces\\a.rs\nTiếng Việt 🙂";
    let output = good(
        command(&d, &["run", "--passthrough", "--codex-bin"])
            .arg(fake())
            .arg("--")
            .arg(prompt)
            .env("CTXC_FAKE_SCENARIO", "echo")
            .output()
            .unwrap(),
    );
    assert_eq!(output.trim_end(), prompt);
    assert!(!d.path().join("marker").exists());
}
#[test]
fn init_config_does_not_overwrite_agents() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("AGENTS.md"), "user instructions").unwrap();
    good(command(&d, &["init"]).output().unwrap());
    good(
        command(&d, &["config", "set", "context.budget", "9000"])
            .output()
            .unwrap(),
    );
    good(command(&d, &["init"]).output().unwrap());
    let c = good(command(&d, &["config", "--json"]).output().unwrap());
    assert_eq!(
        serde_json::from_str::<Value>(&c).unwrap()["context"]["budget"],
        9000
    );
    assert_eq!(
        fs::read_to_string(d.path().join("AGENTS.md")).unwrap(),
        "user instructions"
    );
}
#[test]
fn json_input_budget_and_protected_constraints() {
    let d = tempfile::tempdir().unwrap();
    let input = serde_json::json!({"version":1,"request":"Fix bug","context":[{"kind":"constraint","content":"Do NOT modify lockfile","preserve":"exact"},{"kind":"conversation","content":"noise ".repeat(4000),"preserve":"droppable"}],"policy":{"context_budget":300,"reserve_tokens":20}});
    fs::write(d.path().join("input.json"), input.to_string()).unwrap();
    let out = good(
        command(&d, &["plan", "--input", "input.json", "--json"])
            .output()
            .unwrap(),
    );
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["contract"]
        .as_str()
        .unwrap()
        .contains("Do NOT modify lockfile"));
    assert!(!v["contract"].as_str().unwrap().contains("noise noise"));
}
#[cfg(unix)]
#[test]
fn symlink_state_refused() {
    let d = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join(".ctxc")).unwrap();
    assert!(!command(&d, &["init"]).output().unwrap().status.success());
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
}
