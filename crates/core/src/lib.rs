//! Pure deterministic compiler: no filesystem, processes, network or model calls.
pub mod reducer;
use ctxc_protocol::*;
use regex::Regex;
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("unsupported protocol version {0}; supported: 1")]
    Version(u32),
    #[error("request must not be empty")]
    Empty,
    #[error("invalid policy: {0}")]
    Policy(&'static str),
    #[error("protected content failed fidelity validation")]
    Fidelity,
}

pub trait TokenEstimator {
    fn estimate(&self, text: &str) -> TokenEstimate;
}
pub struct ApproximateEstimator;
impl TokenEstimator for ApproximateEstimator {
    fn estimate(&self, text: &str) -> TokenEstimate {
        // Deliberately cautious heuristic, not a tokenizer nor a guaranteed upper bound.
        TokenEstimate {
            tokens: text.len().div_ceil(3),
            exact: false,
            method: "utf8-bytes/3-ceil (approximate)".into(),
        }
    }
}

fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|w| text.contains(w))
}

pub fn classify(request: &str) -> (Mode, &'static str) {
    let lower = request.trim().to_lowercase();
    let greeting = lower.trim_end_matches(['!', '.', '?']);
    if [
        "hello",
        "hi",
        "hey",
        "thanks",
        "thank you",
        "xin chào",
        "chào",
        "cảm ơn",
        "what model are you using",
    ]
    .contains(&greeting)
    {
        return (Mode::Chat, "standalone-social-or-model-question");
    }
    if contains_any(
        &lower,
        &[
            "migration",
            "migrate",
            "concurren",
            "race condition",
            "deadlock",
            "security",
            "vulnerability",
            "cryptograph",
            "cross-package",
            "architecture",
            "schema change",
            "backward compatib",
            "nondeterministic",
            "bảo mật",
            "kiến trúc",
            "di trú",
        ],
    ) {
        return (
            Mode::Complex,
            "architecture-migration-security-or-concurrency-signal",
        );
    }
    if lower.len() < 220
        && contains_any(
            &lower,
            &[
                "typo",
                "spelling",
                "rename one variable",
                "change one config",
                "lỗi chính tả",
            ],
        )
        && !contains_any(
            &lower,
            &[
                " and ",
                " across ",
                "multiple",
                " và ",
                "then ",
                "implement",
                "refactor",
            ],
        )
    {
        return (Mode::Micro, "single-small-edit");
    }
    (Mode::Code, "conservative-normal-task-default")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedSpan {
    pub start: usize,
    pub end: usize,
    pub reason: &'static str,
}

/// Offsets reference normalized UTF-8 input. Constraints protect the whole line;
/// literals, code fences, paths, commands and identifiers are additionally marked.
pub fn protected_spans(text: &str) -> Vec<ProtectedSpan> {
    let mut spans = Vec::new();
    let constraint = Regex::new(r"(?i)\b(must|not|never|required|only|without|acceptance|preserve|keep|don't|cannot|shall|instead|correction|decided)\b|không|bắt buộc|giữ nguyên|chỉ được").expect("constant regex");
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if constraint.is_match(line) {
            spans.push(ProtectedSpan {
                start: offset,
                end: offset + line.trim_end_matches('\n').len(),
                reason: "constraint-or-correction",
            });
        }
        offset += line.len();
    }
    for (pattern, reason) in [
        (r"(?s)```.*?```|~~~.*?~~~", "code-fence"),
        (r#"`[^`\n]+`|"[^"\n]*"|'[^'\n]*'"#, "literal"),
        (
            r"https?://[^\s]+|(?:[A-Za-z]:[\\/]|\.{0,2}/)[^\s]+|[\w.-]+\.[A-Za-z][\w.-]*",
            "path-url-symbol",
        ),
        (
            r"\b\d[\w.%-]*|\b\w+_\w+\b|\b[a-z]+[A-Z]\w*\b",
            "number-or-identifier",
        ),
        (
            r"(?m)^\s*(?:\$ |PS> |cargo |npm |git |ctxc |python |node ).*$",
            "command",
        ),
    ] {
        let re = Regex::new(pattern).expect("constant regex");
        spans.extend(re.find_iter(text).map(|m| ProtectedSpan {
            start: m.start(),
            end: m.end(),
            reason,
        }));
    }
    spans.sort_by_key(|s| (s.start, s.end));
    spans.dedup_by_key(|s| (s.start, s.end));
    spans
}

pub fn reasoning(mode: Mode, policy: &ExecutionPolicy) -> ReasoningLevel {
    if let Some(level) = policy.effort {
        return level;
    }
    let requested = match mode {
        Mode::Chat => ReasoningLevel::Minimal,
        Mode::Micro => ReasoningLevel::Low,
        Mode::Code => policy.default_reasoning,
        Mode::Complex => policy.complex_reasoning,
    };
    requested
        .min(policy.max_auto_reasoning)
        .min(ReasoningLevel::Medium)
}

pub fn validate_policy(policy: &ExecutionPolicy) -> Result<(), CompileError> {
    if policy.context_budget == 0 || policy.reserve_tokens >= policy.context_budget {
        return Err(CompileError::Policy("context budget must exceed reserve"));
    }
    if policy.max_auto_reasoning > ReasoningLevel::Medium {
        return Err(CompileError::Policy(
            "automatic reasoning ceiling cannot exceed medium; use explicit effort",
        ));
    }
    if !["low", "medium", "high"].contains(&policy.verbosity.as_str()) {
        return Err(CompileError::Policy("verbosity must be low|medium|high"));
    }
    if !["auto", "off", "on"].contains(&policy.web.as_str()) {
        return Err(CompileError::Policy("web must be auto|off|on"));
    }
    if policy.tool_output_token_limit == 0 {
        return Err(CompileError::Policy("tool output limit must be positive"));
    }
    Ok(())
}

fn protected(block: &Block) -> bool {
    if block.kind == BlockKind::ProjectMap && block.preserve == Preserve::Droppable {
        return false;
    }
    if block.status != Status::Active
        && matches!(
            block.kind,
            BlockKind::Error | BlockKind::ToolResult | BlockKind::Code
        )
        && matches!(block.preserve, Preserve::Compressible | Preserve::Droppable)
    {
        return false;
    }
    matches!(block.preserve, Preserve::Exact | Preserve::Semantic)
        || matches!(
            block.kind,
            BlockKind::Constraint
                | BlockKind::Acceptance
                | BlockKind::Objective
                | BlockKind::Decision
                | BlockKind::Error
        )
        || !protected_spans(&block.content).is_empty()
}

fn score(block: &Block) -> f64 {
    fn safe(n: f64) -> f64 {
        if n.is_finite() {
            n.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
    safe(block.relevance) * 0.4
        + safe(block.priority) * 0.25
        + safe(block.dependency) * 0.15
        + safe(block.freshness) * 0.1
        + safe(block.user_weight) * 0.1
}

fn trace(action: &str, id: Option<&str>, reason: &str) -> TraceEvent {
    TraceEvent {
        action: action.into(),
        block_id: id.map(str::to_owned),
        reason: reason.into(),
    }
}

pub fn compile(input: CompileInput) -> Result<CompiledContext, CompileError> {
    compile_with_estimator(input, &ApproximateEstimator)
}

pub fn compile_with_estimator(
    input: CompileInput,
    estimator: &dyn TokenEstimator,
) -> Result<CompiledContext, CompileError> {
    if input.version != PROTOCOL_VERSION {
        return Err(CompileError::Version(input.version));
    }
    if input.request.trim().is_empty() {
        return Err(CompileError::Empty);
    }
    validate_policy(&input.policy)?;
    let request = input.request.replace("\r\n", "\n").replace('\r', "\n");
    let spans = protected_spans(&request);
    let (auto_mode, reason) = classify(&request);
    let mode = input.policy.mode.unwrap_or(auto_mode);
    let effort = reasoning(mode, &input.policy);
    let mut traces = vec![
        trace(
            "CLASSIFY",
            None,
            if input.policy.mode.is_some() {
                "explicit-user-mode"
            } else {
                reason
            },
        ),
        trace(
            "EFFORT",
            None,
            if input.policy.effort.is_some() {
                "explicit-user-effort"
            } else {
                "lowest-policy-level-with-medium-ceiling"
            },
        ),
    ];
    for span in &spans {
        traces.push(trace("KEEP", Some("request"), span.reason));
    }
    // Never rewrite the natural request in v0.1. Deterministic semantic rewriting
    // cannot reliably infer arbitrary multilingual intent. Optimize context around it.
    let mut contract = format!("[TASK]\nObjective / original request (preserved):\n{request}\n");
    if mode == Mode::Chat {
        contract.push_str(
            "\nExecution: Answer concisely. No repository investigation, tests or agents.\n",
        );
    } else {
        contract.push_str("\nExecution: Use the smallest sufficient investigation and targeted tests. Do not repeat resolved investigation or broaden scope. Stop when the requested acceptance criteria are satisfied. Keep explanations concise.\n");
        if !input.policy.agents {
            contract.push_str("Use a single agent.\n");
        }
        if input.policy.web == "auto" {
            contract.push_str("Avoid unrequested web browsing unless required for correctness.\n");
        }
    }
    if input.policy.verbosity != "low" {
        contract.push_str(if input.policy.verbosity == "high" {
            "User output preference: provide detailed explanations relevant to this task.\n"
        } else {
            "User output preference: provide a moderate amount of relevant explanation.\n"
        });
    }
    let before = estimator.estimate(&input.request).tokens
        + input
            .context
            .iter()
            .map(|b| estimator.estimate(&b.content).tokens)
            .sum::<usize>();
    let budget = input.policy.context_budget - input.policy.reserve_tokens;
    let mut blocks = input.context;
    for (i, b) in blocks.iter_mut().enumerate() {
        if b.id.is_empty() {
            b.id = format!("ctx_{i}");
        }
        b.token_estimate = estimator.estimate(&b.content).tokens;
    }
    // Protected blocks first, then transparent score, stable input order on ties.
    let mut ranked: Vec<_> = blocks
        .into_iter()
        .map(|b| (protected(&b), score(&b), b))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.total_cmp(&a.1)));
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for (is_protected, _, b) in ranked {
        let reason_drop = if mode == Mode::Chat && b.kind == BlockKind::ProjectMap {
            Some("chat-needs-no-project-map")
        } else if b.status != Status::Active && !is_protected {
            Some("inactive-lifecycle")
        } else if !seen.insert((
            b.content.clone(),
            format!("{:?}:{:?}:{:?}:{}", b.kind, b.preserve, b.status, b.source),
        )) {
            Some("exact-duplicate")
        } else {
            None
        };
        if let Some(reason) = reason_drop {
            traces.push(trace("DROP", Some(&b.id), reason));
            continue;
        }
        let rendered = format!(
            "\n[CONTEXT {:?}; source={}]\n{}\n",
            b.kind,
            safe_label(&b.source),
            b.content
        );
        if !is_protected && estimator.estimate(&(contract.clone() + &rendered)).tokens > budget {
            traces.push(trace("DROP", Some(&b.id), "context-budget"));
            continue;
        }
        traces.push(trace(
            "KEEP",
            Some(&b.id),
            if is_protected {
                "protected-content"
            } else {
                "relevance-within-budget"
            },
        ));
        contract.push_str(&rendered);
        selected.push(b);
    }
    if !contract.contains(&request) || selected.iter().any(|b| !contract.contains(&b.content)) {
        return Err(CompileError::Fidelity);
    }
    let after = estimator.estimate(&contract);
    let exceeded = after.tokens > budget;
    if exceeded {
        traces.push(trace(
            "WARN",
            None,
            "protected-content-exceeds-budget-kept-intact",
        ));
    }
    Ok(CompiledContext {
        version: PROTOCOL_VERSION,
        classification: mode,
        reasoning: effort,
        contract,
        blocks: selected,
        policy: input.policy,
        estimate: SizeEstimate {
            input_before: before,
            input_after: after.tokens,
            exact: after.exact,
            method: after.method,
        },
        trace: traces,
        budget_exceeded: exceeded,
    })
}

fn safe_label(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || "_-./".contains(*c))
        .take(80)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(s: &str) -> CompileInput {
        CompileInput {
            version: 1,
            request: s.into(),
            context: vec![],
            policy: ExecutionPolicy::default(),
        }
    }
    #[test]
    fn behavior() {
        for (s, mode, effort) in [
            ("hello", Mode::Chat, ReasoningLevel::Minimal),
            ("Fix typo in README.md.", Mode::Micro, ReasoningLevel::Low),
            (
                "Fix refresh token expiration in src/auth.ts without changing the public API.",
                Mode::Code,
                ReasoningLevel::Low,
            ),
            (
                "Cross-package schema migration with concurrency and backward compatibility",
                Mode::Complex,
                ReasoningLevel::Medium,
            ),
        ] {
            let c = compile(input(s)).unwrap();
            assert_eq!(c.classification, mode);
            assert_eq!(c.reasoning, effort);
            assert!(c.contract.contains(s));
        }
    }
    #[test]
    fn fidelity_adversarial() {
        for s in [
            "Do NOT modify package-lock.json.",
            "MUST preserve API v1.2.3 and limit 42%",
            "Không được đổi public API. Giữ nguyên `user_id`.",
            "Use C:\\My Project\\app.rs; echo $HOME | `whoami`",
            "```json\n{\"key\": \"a  b\"}\n```",
            "first decision: X\ncorrection: use Y instead",
            "$(touch stolen) --help",
        ] {
            let c = compile(input(s)).unwrap();
            assert!(c.contract.contains(s));
            for p in protected_spans(s) {
                assert!(c.contract.contains(&s[p.start..p.end]));
            }
        }
    }
    #[test]
    fn budget_keeps_constraints_and_drops_noise() {
        let mut i = input("fix bug");
        i.policy.context_budget = 100;
        i.policy.reserve_tokens = 1;
        i.context = vec![
            Block {
                content: "must not remove authentication".into(),
                kind: BlockKind::Constraint,
                ..Block::default()
            },
            Block {
                content: "noise ".repeat(200),
                ..Block::default()
            },
        ];
        let c = compile(i).unwrap();
        assert_eq!(c.blocks.len(), 1);
        assert!(c.budget_exceeded);
    }
    #[test]
    fn dedup_and_lifecycle() {
        let b = Block {
            content: "some ordinary context".into(),
            ..Block::default()
        };
        let mut i = input("fix bug");
        i.context = vec![
            b.clone(),
            b,
            Block {
                content: "old noise".into(),
                status: Status::Resolved,
                ..Block::default()
            },
        ];
        let c = compile(i).unwrap();
        assert_eq!(c.blocks.len(), 1);
        assert_eq!(c.trace.iter().filter(|t| t.action == "DROP").count(), 2);
    }
    #[test]
    fn explicit_override_and_auto_ceiling() {
        let mut i = input("migration");
        i.policy.effort = Some(ReasoningLevel::Extreme);
        assert_eq!(compile(i).unwrap().reasoning, ReasoningLevel::Extreme);
        let p = ExecutionPolicy {
            complex_reasoning: ReasoningLevel::Extreme,
            ..ExecutionPolicy::default()
        };
        assert_eq!(reasoning(Mode::Complex, &p), ReasoningLevel::Medium);
    }
    #[test]
    fn estimate_is_honest_and_unicode_safe() {
        for s in ["", "hi", "你好", "🙂"] {
            let e = ApproximateEstimator.estimate(s);
            assert!(!e.exact);
            assert_eq!(e.tokens, s.len().div_ceil(3));
        }
    }
    #[test]
    fn deterministic() {
        let a = compile(input("Fix api.rs")).unwrap();
        let b = compile(input("Fix api.rs")).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }
}
