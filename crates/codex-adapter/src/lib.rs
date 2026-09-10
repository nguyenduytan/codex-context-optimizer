//! Codex-specific compatibility and process I/O. No shell interpolation or retries.
use anyhow::{bail, Context, Result};
use ctxc_protocol::{CompiledContext, ReasoningLevel, UsageSnapshot};
use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsString,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub version: String,
    pub json: bool,
    pub config: bool,
    pub ephemeral: bool,
    pub sandbox: bool,
    pub known_config_schema: bool,
    pub reasoning: Vec<ReasoningLevel>,
    pub warnings: Vec<String>,
}

pub fn locate(binary: &Path) -> Result<PathBuf> {
    let explicit = binary.is_absolute() || binary.components().count() > 1;
    let candidates: Vec<PathBuf> = if explicit {
        vec![binary.to_path_buf()]
    } else {
        env::split_paths(&env::var_os("PATH").unwrap_or_default())
            .flat_map(|dir| {
                let path = dir.join(binary);
                if cfg!(windows) && path.extension().is_none() {
                    vec![path.with_extension("exe")]
                } else {
                    vec![path]
                }
            })
            .collect()
    };
    for path in candidates {
        if path.is_file() {
            let path = path.canonicalize()?;
            if cfg!(windows)
                && !path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
            {
                bail!("Use a native Codex .exe; .cmd/.bat wrappers are not executed for argument safety");
            }
            return Ok(path);
        }
    }
    bail!("Codex executable not found. Install Codex CLI and authenticate separately; or set --codex-bin to a native executable. See https://developers.openai.com/codex/cli")
}

fn probe(binary: &Path, args: &[&str]) -> Result<String> {
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("start read-only Codex capability probe")?;
    let stdout = child.stdout.take().context("probe stdout unavailable")?;
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        // Drain all output but retain at most 128 KiB.
        let mut stream = BufReader::new(stdout);
        let mut chunk = [0; 8192];
        loop {
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            let remaining = (128 * 1024usize).saturating_sub(bytes.len());
            bytes.extend_from_slice(&chunk[..n.min(remaining)]);
        }
        Ok::<_, std::io::Error>(bytes)
    });
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let bytes = reader
                .join()
                .map_err(|_| anyhow::anyhow!("probe reader failed"))??;
            if !status.success() {
                bail!("Codex capability probe failed; no model run attempted");
            }
            return Ok(String::from_utf8_lossy(&bytes).into_owned());
        }
        if started.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Codex capability probe timed out; no model run attempted");
        }
        thread::sleep(Duration::from_millis(15));
    }
}

pub fn detect(binary: &Path) -> Result<RuntimeCapabilities> {
    let version = probe(binary, &["--version"])?;
    let help = probe(binary, &["exec", "--help"])?;
    Ok(capabilities_from_help(version.trim(), &help))
}

pub fn capabilities_from_help(version: &str, help: &str) -> RuntimeCapabilities {
    // Only this config family has a checked fixture. A newer executable still
    // runs with detected flags, without speculative dotted config keys.
    let known = version.split_whitespace().any(|v| v.starts_with("0.153."));
    let mut warnings = Vec::new();
    if !known {
        warnings.push("Unverified Codex config schema: native optimization overrides omitted; compact task guidance still applies.".into());
    }
    RuntimeCapabilities {
        version: version.to_owned(),
        json: help.contains("--json"),
        config: help.contains("--config"),
        ephemeral: help.contains("--ephemeral"),
        sandbox: help.contains("--sandbox"),
        known_config_schema: known,
        reasoning: if known {
            vec![
                ReasoningLevel::Low,
                ReasoningLevel::Medium,
                ReasoningLevel::High,
                ReasoningLevel::Extreme,
            ]
        } else {
            vec![]
        },
        warnings,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Invocation {
    pub executable: PathBuf,
    pub args: Vec<String>,
    #[serde(skip_serializing)]
    pub stdin: String,
    pub structured: bool,
    pub applied_reasoning: Option<ReasoningLevel>,
    pub warnings: Vec<String>,
}

pub fn map_reasoning(
    requested: ReasoningLevel,
    supported: &[ReasoningLevel],
) -> Option<ReasoningLevel> {
    supported.iter().copied().filter(|x| *x >= requested).min()
}

pub fn prepare(
    binary: &Path,
    caps: &RuntimeCapabilities,
    compiled: &CompiledContext,
    original: &str,
    passthrough: bool,
    model: Option<&str>,
    sandbox: Option<&str>,
) -> Result<Invocation> {
    let mut args = vec!["exec".into()];
    let mut warnings = caps.warnings.clone();
    if caps.json {
        args.push("--json".into());
    } else {
        warnings.push("JSONL unavailable; raw output forwarded and usage unavailable.".into());
    }
    if caps.ephemeral {
        args.push("--ephemeral".into());
    } else {
        warnings.push("Ephemeral mode unavailable; Codex may persist its own sessions.".into());
    }
    if let Some(model) = model {
        args.extend(["--model".into(), model.into()]);
    }
    if let Some(sandbox) = sandbox {
        if !["read-only", "workspace-write"].contains(&sandbox) {
            bail!("sandbox must be read-only or workspace-write");
        }
        if !caps.sandbox {
            bail!("requested sandbox flag unsupported by this Codex; no run attempted");
        }
        args.extend(["--sandbox".into(), sandbox.into()]);
    }
    let mut applied = None;
    if !passthrough && caps.config && caps.known_config_schema {
        applied = map_reasoning(compiled.reasoning, &caps.reasoning);
        if let Some(effort) = applied {
            let name = match effort {
                ReasoningLevel::Minimal => "minimal",
                ReasoningLevel::Low => "low",
                ReasoningLevel::Medium => "medium",
                ReasoningLevel::High => "high",
                ReasoningLevel::Extreme => "xhigh",
            };
            args.extend(["-c".into(), format!("model_reasoning_effort=\"{name}\"")]);
        }
        for value in [
            format!("features.multi_agent={}", compiled.policy.agents),
            format!(
                "tool_output_token_limit={}",
                compiled.policy.tool_output_token_limit
            ),
            "model_reasoning_summary=\"none\"".into(),
        ] {
            args.extend(["-c".into(), value]);
        }
        // Verbosity is model-specific: use task guidance by default rather than
        // risk rejecting a compatible but non-GPT-5 provider/model.
        warnings.push(
            "Verbosity is task guidance only; model-specific verbosity control is not assumed."
                .into(),
        );
        if compiled.policy.web != "auto" {
            let web = if compiled.policy.web == "off" {
                "disabled"
            } else {
                "live"
            };
            args.extend(["-c".into(), format!("web_search=\"{web}\"")]);
        }
    } else if !passthrough
        && (compiled.policy.effort.is_some()
            || compiled.policy.web != "auto"
            || compiled.policy.agents)
    {
        bail!("explicit runtime control cannot be mapped safely to this Codex version; use --passthrough or a supported version");
    }
    // Prompt never becomes argv, including leading hyphens, shell metacharacters
    // and Windows quoting. '-' is Codex's documented stdin mode.
    args.push("-".into());
    Ok(Invocation {
        executable: binary.into(),
        args,
        stdin: if passthrough {
            original.into()
        } else {
            compiled.contract.clone()
        },
        structured: caps.json,
        applied_reasoning: applied,
        warnings,
    })
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub usage: UsageSnapshot,
    pub exit_status: i32,
    pub duration_ms: u128,
    pub malformed_events: usize,
    pub failed_event: bool,
    pub interrupted: bool,
}

pub fn observe_line(line: &str, state: &mut RunResult) -> Option<String> {
    let value: serde_json::Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => {
            state.malformed_events += 1;
            return Some(line.into());
        }
    };
    let Some(kind) = value.get("type").and_then(|v| v.as_str()) else {
        state.malformed_events += 1;
        return Some(line.into());
    };
    match kind {
        "turn.completed" => {
            if let Some(u) = value.get("usage") {
                // Missing is null, never fabricated zero. Codex turn usage is a
                // per-turn snapshot; aggregate successive completed turns.
                fn add(to: &mut Option<u64>, n: Option<u64>) {
                    if let Some(n) = n {
                        *to = Some(to.unwrap_or(0).saturating_add(n));
                    }
                }
                add(
                    &mut state.usage.input_tokens,
                    u.get("input_tokens").and_then(|v| v.as_u64()),
                );
                add(
                    &mut state.usage.cached_input_tokens,
                    u.get("cached_input_tokens").and_then(|v| v.as_u64()),
                );
                add(
                    &mut state.usage.output_tokens,
                    u.get("output_tokens").and_then(|v| v.as_u64()),
                );
                add(
                    &mut state.usage.reasoning_tokens,
                    u.get("reasoning_output_tokens")
                        .or_else(|| u.get("reasoning_tokens"))
                        .and_then(|v| v.as_u64()),
                );
            }
            None
        }
        "error" | "turn.failed" => {
            state.failed_event = true;
            Some(line.into())
        }
        "item.completed" => {
            if value["item"]["type"] == "agent_message" {
                value["item"]["text"].as_str().map(str::to_owned)
            } else {
                None
            }
        }
        "thread.started" | "turn.started" | "item.started" | "item.updated" => None,
        _ => Some(line.into()),
    }
}

pub fn run<F>(
    invocation: &Invocation,
    cwd: &Path,
    cancelled: Arc<AtomicBool>,
    raw_json: bool,
    mut emit: F,
) -> Result<RunResult>
where
    F: FnMut(&str) -> Result<()>,
{
    let started = Instant::now();
    let mut child = Command::new(&invocation.executable)
        .args(invocation.args.iter().map(OsString::from))
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("start Codex")?;
    let mut stdin = child.stdin.take().context("stdin unavailable")?;
    let prompt = invocation.stdin.clone();
    let writer = thread::spawn(move || stdin.write_all(prompt.as_bytes()));
    let stdout = child.stdout.take().context("stdout unavailable")?;
    let (tx, rx) = mpsc::sync_channel(16);
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut bytes = Vec::new();
            // Bound every buffered chunk even for a malicious unterminated line.
            match reader
                .by_ref()
                .take(256 * 1024)
                .read_until(b'\n', &mut bytes)
            {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(Ok(bytes)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    break;
                }
            }
        }
    });
    let mut state = RunResult::default();
    let outcome: Result<()> = (|| {
        loop {
            if cancelled.load(Ordering::SeqCst) {
                state.interrupted = true;
                let _ = child.kill();
                break;
            }
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(bytes) => {
                    let bytes = bytes?;
                    let line = String::from_utf8_lossy(&bytes);
                    let display = if invocation.structured {
                        observe_line(&line, &mut state)
                    } else {
                        Some(line.to_string())
                    };
                    if raw_json {
                        emit(&line)?;
                    } else if let Some(text) = display {
                        emit(&text)?;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    })();
    if outcome.is_err() {
        let _ = child.kill();
    }
    drop(rx);
    let status = child.wait()?;
    let _ = reader.join();
    let write_result = writer
        .join()
        .map_err(|_| anyhow::anyhow!("prompt writer failed"))?;
    outcome?;
    if status.success() && !state.interrupted {
        write_result.context("send full task to Codex")?;
    }
    state.exit_status = if state.interrupted {
        130
    } else if state.failed_event && status.success() {
        1
    } else {
        status.code().unwrap_or(1)
    };
    state.duration_ms = started.elapsed().as_millis();
    if state.malformed_events > 0 {
        state.usage = UsageSnapshot::default();
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability_fallback() {
        let c = capabilities_from_help("codex-cli 99.0.0", "--json --config");
        assert!(c.json);
        assert!(!c.known_config_schema);
        assert!(!c.warnings.is_empty());
    }
    #[test]
    fn effort_mapping() {
        assert_eq!(
            map_reasoning(
                ReasoningLevel::Minimal,
                &[ReasoningLevel::Low, ReasoningLevel::Medium]
            ),
            Some(ReasoningLevel::Low)
        );
        assert_eq!(
            map_reasoning(ReasoningLevel::Extreme, &[ReasoningLevel::Low]),
            None
        );
    }
    #[test]
    fn event_usage() {
        let mut r = RunResult::default();
        observe_line(
            r#"{"type":"turn.completed","usage":{"input_tokens":42,"cached_input_tokens":12,"output_tokens":5}}"#,
            &mut r,
        );
        assert_eq!(r.usage.input_tokens, Some(42));
        assert_eq!(r.usage.reasoning_tokens, None);
        observe_line("bad json", &mut r);
        assert_eq!(r.malformed_events, 1);
    }
    #[test]
    fn missing_usage_and_error() {
        let mut r = RunResult::default();
        observe_line(r#"{"type":"turn.completed"}"#, &mut r);
        assert_eq!(r.usage.input_tokens, None);
        observe_line(r#"{"type":"error","message":"quota"}"#, &mut r);
        assert!(r.failed_event);
    }
}
