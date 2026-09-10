# Benchmarking

No percentage savings, live task-success measurements or statistically significant quality claims are published for v0.1.

## Offline Checks

Unit/behavioral fixtures verify classification, protected exact values, budgets, lifecycle, failure evidence and fake-runtime behavior. These prove specific invariants, not model task quality.

Token estimates use ceil(UTF-8 bytes / 3), marked approximate. This is not a tokenizer or a guaranteed upper bound. Short requests may grow due to execution guidance. Input-before/input-after compare only caller-supplied request and context, not Codex's complete prompt or hidden instruction stack.

## Manual Baseline

Run paired tasks on fresh copies of the same repository commit with the same Codex version, model, authentication tier, sandbox and tests. Run the plain Codex baseline first or alternate order to control caching. Then run ctxc, capturing provider JSONL usage separately. Do not put private task payloads in the benchmark repository.

Record: task identifier, commit, runtime/model, requested/applied effort, pass/fail of task acceptance, changed files, input/cached/output/reasoning tokens where exposed, tool calls where exposed, wall time, test results and retry count. Missing usage stays null.

Use repeated trials across micro edits, focused bugs, architecture/migrations, large failing logs, long tasks and adversarial constraints. Report quality retained alongside usage, distributions and uncertainty, not just best-case reductions. Live benchmarks must be explicitly requested and never run in normal CI.
