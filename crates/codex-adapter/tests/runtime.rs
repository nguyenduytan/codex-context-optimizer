#![cfg(feature = "test-runtime")]
use ctxc_codex_adapter::*;
use ctxc_protocol::*;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
fn compiled(prompt: &str) -> CompiledContext {
    CompiledContext {
        version: 1,
        classification: Mode::Code,
        reasoning: ReasoningLevel::Low,
        contract: prompt.into(),
        blocks: vec![],
        policy: ExecutionPolicy::default(),
        estimate: SizeEstimate {
            input_before: 1,
            input_after: 1,
            exact: false,
            method: "test".into(),
        },
        trace: vec![],
        budget_exceeded: false,
    }
}
#[test]
fn native_runtime_and_safe_argv() {
    let binary = Path::new(env!("CARGO_BIN_EXE_fake-codex"));
    let caps = detect(binary).unwrap();
    assert!(caps.json);
    assert!(caps.known_config_schema);
    let prompt="Do NOT edit C:\\My Project\\app.rs; $(echo stolen) | `whoami` & \"quotes\"\nUnicode: Việt Nam 🙂 --flag";
    let inv = prepare(binary, &caps, &compiled(prompt), prompt, false, None, None).unwrap();
    assert!(!inv.args.iter().any(|a| a.contains("stolen")));
    assert_eq!(inv.stdin, prompt);
    let dir = tempfile::tempdir().unwrap();
    let mut out = String::new();
    let result = run(
        &inv,
        dir.path(),
        Arc::new(AtomicBool::new(false)),
        false,
        |s| {
            out.push_str(s);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(result.exit_status, 0);
    assert_eq!(result.usage.input_tokens, Some(42));
    assert!(out.contains("fake complete"));
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
}
#[test]
fn passthrough_has_no_optimization_overrides() {
    let binary = Path::new(env!("CARGO_BIN_EXE_fake-codex"));
    let caps = detect(binary).unwrap();
    let inv = prepare(
        binary,
        &caps,
        &compiled("compiled"),
        "original",
        true,
        None,
        None,
    )
    .unwrap();
    assert_eq!(inv.stdin, "original");
    assert!(!inv.args.contains(&"-c".into()));
}
#[test]
fn cancellation_reaps_child() {
    let binary = Path::new(env!("CARGO_BIN_EXE_fake-codex"));
    let inv = prepare(
        binary,
        &detect(binary).unwrap(),
        &compiled("FAKE_WAIT"),
        "",
        false,
        None,
        None,
    )
    .unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    let trigger = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        signal.store(true, Ordering::SeqCst);
    });
    let dir = tempfile::tempdir().unwrap();
    let result = run(&inv, dir.path(), cancelled, false, |_| Ok(())).unwrap();
    trigger.join().unwrap();
    assert!(result.interrupted);
    assert_eq!(result.exit_status, 130);
}
#[test]
fn unknown_version_preserves_explicit_control_safety() {
    let caps = capabilities_from_help("99.0.0", "--config --json");
    let mut c = compiled("hello");
    c.policy.effort = Some(ReasoningLevel::High);
    assert!(prepare(Path::new("codex"), &caps, &c, "hello", false, None, None).is_err());
}
