use anyhow::{bail, Context, Result};
use ctxc_codex_adapter::RunResult;
use ctxc_protocol::{Mode, ReasoningLevel, SizeEstimate, TraceEvent};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Refuse symlinks/junctions at all components below the user's chosen root.
/// Private state must never follow a repository-supplied link out of scope.
pub fn ensure_safe(root: &Path, target: &Path) -> Result<()> {
    let relative = target
        .strip_prefix(root)
        .context("write target outside project")?;
    let mut current = root.to_path_buf();
    for part in relative.components() {
        if !matches!(part, std::path::Component::Normal(_)) {
            bail!("non-normal state path");
        }
        current.push(part);
        if let Ok(m) = fs::symlink_metadata(&current) {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if m.file_attributes() & 0x400 != 0 {
                    bail!("refusing reparse-point state path: {}", current.display());
                }
            }
            if m.file_type().is_symlink() {
                bail!("refusing symlink state path: {}", current.display());
            }
        }
    }
    Ok(())
}
pub fn atomic_write(root: &Path, target: &Path, bytes: &[u8]) -> Result<()> {
    ensure_safe(root, target)?;
    let parent = target.parent().context("missing parent")?;
    fs::create_dir_all(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if parent.starts_with(root.join(".ctxc")) {
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(target).map_err(|e| e.error)?;
    Ok(())
}
pub fn redact(text: &str) -> String {
    let token = Regex::new(r"(?i)\b(?:sk-[a-z0-9_-]{8,}|gh[pousr]_[a-z0-9_]{8,}|github_pat_[a-z0-9_]+|AKIA[A-Z0-9]{16})\b").expect("constant regex");
    let assignment = Regex::new(r#"(?i)((?:api[_-]?key|access[_-]?token|password|secret|authorization)\s*[:=]\s*)(?:"[^"]*"|'[^']*'|\S+)"#).expect("constant regex");
    let bearer = Regex::new(r"(?i)Bearer\s+\S+").expect("constant regex");
    let out = token.replace_all(text, "[REDACTED]");
    let out = assignment.replace_all(&out, "${1}[REDACTED]");
    bearer.replace_all(&out, "Bearer [REDACTED]").into_owned()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    pub schema_version: u32,
    pub timestamp_unix_ms: u128,
    pub runtime: String,
    pub runtime_version: Option<String>,
    pub mode: Mode,
    pub requested_reasoning: ReasoningLevel,
    pub applied_reasoning: Option<ReasoningLevel>,
    pub estimate: SizeEstimate,
    pub project_map_bytes: usize,
    pub trace: Vec<TraceEvent>,
    pub result: Option<RunResult>,
}
pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
pub fn save(root: &Path, metadata: &RunMetadata, retain: usize) -> Result<()> {
    if retain == 0 {
        return Ok(());
    }
    let state = root.join(".ctxc/state");
    ensure_safe(root, &state)?;
    fs::create_dir_all(&state)?;
    // Separate run files avoid read/modify/write history loss under concurrency.
    let path = state.join(format!(
        "run-{}-{}-{}.json",
        metadata.timestamp_unix_ms,
        std::process::id(),
        now()
    ));
    atomic_write(root, &path, &serde_json::to_vec_pretty(metadata)?)?;
    let mut files = run_files(root)?;
    files.sort();
    let count = files.len().saturating_sub(retain);
    for path in files.into_iter().take(count) {
        ensure_safe(root, &path)?;
        fs::remove_file(path)?;
    }
    Ok(())
}
fn run_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let path = root.join(".ctxc/state");
    ensure_safe(root, &path)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut files = vec![];
    let pattern = Regex::new(r"^run-\d+-\d+-\d+\.json$").expect("constant regex");
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if pattern.is_match(&entry.file_name().to_string_lossy()) && entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    Ok(files)
}
pub fn history(root: &Path) -> Result<Vec<RunMetadata>> {
    let mut files = run_files(root)?;
    files.sort();
    let mut history = vec![];
    for file in files.into_iter().rev().take(1000) {
        ensure_safe(root, &file)?;
        if fs::metadata(&file)?.len() > 1024 * 1024 {
            continue;
        }
        if let Ok(record) = serde_json::from_slice(&fs::read(file)?) {
            history.push(record);
        }
    }
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secrets_redacted() {
        let s = redact("api_key=abc password: 'hello world' Bearer example sk-1234567890");
        for secret in ["abc", "hello world", "example", "sk-1234567890"] {
            assert!(!s.contains(secret));
        }
    }
    #[test]
    fn outside_path_refused() {
        let d = tempfile::tempdir().unwrap();
        assert!(atomic_write(d.path(), &d.path().join("../oops"), b"x").is_err());
    }
    #[test]
    fn write_roundtrip() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".ctxc/state/file.json");
        atomic_write(d.path(), &p, b"one").unwrap();
        atomic_write(d.path(), &p, b"two").unwrap();
        assert_eq!(fs::read(p).unwrap(), b"two");
    }
}
