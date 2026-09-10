//! Standalone tool-output reducer. The Codex adapter cannot intercept Codex's
//! internal tool outputs; it maps the runtime's native history limit instead.
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct ReducedOutput {
    pub content: String,
    pub truncated: bool,
    pub original_bytes: usize,
    pub budget_exceeded: bool,
}

pub fn reduce_output(
    command: &str,
    exit_code: i32,
    output: &str,
    max_bytes: usize,
) -> ReducedOutput {
    let header = format!("Command: {command}\nExit: {exit_code}\n");
    if header.len() + output.len() <= max_bytes {
        return ReducedOutput {
            content: header + output,
            truncated: false,
            original_bytes: output.len(),
            budget_exceeded: false,
        };
    }
    let lines: Vec<&str> = output.lines().collect();
    let mut keep = BTreeSet::new();
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if [
            "error",
            "fail",
            "panic",
            "assert",
            "exception",
            "test result:",
            "fatal",
        ]
        .iter()
        .any(|s| lower.contains(s))
        {
            for n in i.saturating_sub(1)..=(i + 2).min(lines.len().saturating_sub(1)) {
                keep.insert(n);
            }
        }
    }
    // Unknown failure shape: preserve raw evidence rather than silently lose it.
    if exit_code != 0 && keep.is_empty() {
        return ReducedOutput {
            content: header + output,
            truncated: false,
            original_bytes: output.len(),
            budget_exceeded: true,
        };
    }
    let mut result = header;
    for n in &keep {
        result.push_str(lines[*n]);
        result.push('\n');
    }
    for (n, line) in lines.iter().enumerate().skip(lines.len().saturating_sub(5)) {
        if !keep.contains(&n) && result.len() + line.len() + 40 <= max_bytes {
            result.push_str(line);
            result.push('\n');
        }
    }
    result.push_str("[output reduced; omitted lines]\n");
    let exceeded = result.len() > max_bytes;
    ReducedOutput {
        content: result,
        truncated: true,
        original_bytes: output.len(),
        budget_exceeded: exceeded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failure_survives() {
        let log = format!(
            "{}\nFAILED auth_refresh\nexpected 42, got 0\n{}",
            "passed\n".repeat(1000),
            "ok\n".repeat(1000)
        );
        let r = reduce_output("cargo test", 1, &log, 400);
        assert!(r.content.contains("FAILED auth_refresh"));
        assert!(r.content.contains("expected 42"));
        assert!(r.truncated);
    }
    #[test]
    fn unknown_failure_kept() {
        let r = reduce_output("x", 1, "unrecognized evidence", 5);
        assert!(r.content.contains("unrecognized evidence"));
        assert!(r.budget_exceeded);
    }
    #[test]
    fn unicode() {
        let r = reduce_output("test", 0, &"✅\n".repeat(100), 100);
        assert!(r.content.len() <= 100);
    }
}
