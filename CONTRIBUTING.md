# Contributing

Read `AGENTS.md`, relevant docs and the master plan. Keep changes focused. Add a unit/behavioral fixture for compiler behavior and a fake-runtime test for adapter behavior.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
git diff --check
```

Never run live model calls in CI or commit prompts, responses, credentials, `.ctxc` state, `target/` or unmeasured benchmark claims. Every optimization must explain removed waste, fidelity risk, tests and how users inspect or disable it.
