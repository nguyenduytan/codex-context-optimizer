# Architecture

ctxc is a deterministic local pipeline. `crates/core` has no process, filesystem, network, provider or model dependency. Provider behavior belongs behind `crates/codex-adapter`.

Pipeline: normalize input; extract protected spans; classify `CHAT`/`MICRO`/`CODE`/`COMPLEX`; build typed CIR; score relevance; apply lifecycle/dedup/budget; render a contract; validate fidelity; map policy to Codex; observe JSONL; store bounded metadata.

The adapter probes version/help, constructs explicit argv, passes the contract through stdin and applies per-run overrides only for a verified schema. Unknown versions warn; explicit controls that cannot be verified fail closed.
