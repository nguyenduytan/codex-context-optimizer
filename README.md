# Codex Context Optimizer

[![CI](https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml) [![Release](https://img.shields.io/github/v/release/nguyenduytan/codex-context-optimizer)](https://github.com/nguyenduytan/codex-context-optimizer/releases) [![Crates.io](https://img.shields.io/crates/v/ctxc.svg)](https://crates.io/crates/ctxc) [![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Stars](https://img.shields.io/github/stars/nguyenduytan/codex-context-optimizer?style=social)](https://github.com/nguyenduytan/codex-context-optimizer/stargazers)

**A local-first context compiler and execution-budget governor for Codex and AI coding workflows.**

Less context. More signal. `ctxc` compiles task intent into an inspectable contract, protects high-risk constraints, bounds repository context/tool output, selects the lowest adequate reasoning effort and prevents wasteful retries.

> **v0.1 is Codex-first.** It does not intercept ChatGPT's private hidden context, control private chain-of-thought, or promise a quota-saving percentage. Claims require measured benchmarks.

![GitHub stars chart](https://starchart.cc/nguyenduytan/codex-context-optimizer.svg)

## Contents

- [Install](#install)
- [Quick start](#quick-start)
- [Commands](#commands)
- [Optimization](#optimization)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Privacy and security](#privacy-and-security)
- [Development](#development)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Install

Native releases target Linux x86_64/ARM64, macOS Intel/Apple Silicon and Windows x86_64/ARM64. Archives are checked with `SHA256SUMS`.

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.sh | CTXC_REPOSITORY=nguyenduytan/codex-context-optimizer sh
```

PowerShell:

```powershell
iwr https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.ps1 -UseBasicParsing | iex
```

The installers do not edit PATH, install a daemon, modify global Codex config or send telemetry. Source builds are also supported:

```bash
# After ctxc is published to crates.io:
cargo install ctxc
git clone https://github.com/nguyenduytan/codex-context-optimizer.git
cd codex-context-optimizer && cargo install --path crates/cli
```

## Quick start

```bash
cd your-repository
ctxc init
ctxc "fix the failing auth test without changing the public API"
ctxc plan "fix the failing auth test"
ctxc run --dry-run "fix the failing auth test"
ctxc explain
ctxc usage
```

`ctxc` is shorthand for `ctxc run`; `plan` never starts Codex; `doctor` is read-only. Use `ctxc run --passthrough "task"` for an explicit escape hatch.

## Commands

| Command | Behavior |
| --- | --- |
| `ctxc init` | Create versioned config, bounded project map and private-state ignore rules. |
| `ctxc doctor` | Check ctxc, Codex, Git and config without model usage. |
| `ctxc plan <task>` | Compile locally. |
| `ctxc run <task>` | Compile and run Codex once. |
| `ctxc run --dry-run <task>` | Show classification, contract, safe argv and estimates. |
| `ctxc explain` | Explain latest bounded metadata and trace. |
| `ctxc usage` | Show observed usage; unavailable values remain unavailable. |
| `ctxc config` | Print effective config. |
| `ctxc config set context.budget 8000` | Validate and update one project setting. |
| `ctxc setup codex` | Explain integration; never edit global config. |

Flags include `--mode auto|chat|micro|code|complex`, `--effort auto|minimal|low|medium|high|extreme`, `--agents off|on`, `--web auto|off|on`, `--project-map-limit`, `--tool-output-limit`, `--trace`, `--json`, `--input FILE`, `--passthrough` and `-C DIR`.

## Optimization

| Area | v0.1 behavior |
| --- | --- |
| Classification | Deterministic `CHAT`, `MICRO`, `CODE`, `COMPLEX` heuristics. |
| Reasoning | Minimal/low by default; automatic complex effort caps at medium. Explicit effort wins. |
| Context | Typed CIR blocks, lifecycle/status, deduplication and relevance budget. |
| Repository map | Path/metadata map, default 8 KiB; excludes dependencies, build trees, generated files and secret-like files. |
| Tool output | Keeps failure evidence and nearby lines; records truncation. |
| Codex | `codex exec`, stdin prompts, JSONL where available, capability detection and per-run overrides. |
| Usage | Bounded metadata only; raw prompts/responses are not stored by default. |

Explicit negations, paths, identifiers, literals, numbers, commands and acceptance-like lines are protected. If safety is uncertain, ctxc keeps content. It does not promise to rewrite hidden ChatGPT UI context.

## Configuration

`ctxc init` creates `.ctxc/config.toml`. CLI flags override project config, project config overrides user config, and safe built-ins are last. User config is read from `CTXC_USER_CONFIG`, `$XDG_CONFIG_HOME/ctxc/config.toml` or `%APPDATA%/ctxc/config.toml`.

```toml
[reasoning]
default = "low"
complex = "medium"
max_auto = "medium"

[context]
project_map_max_bytes = 8192
tool_output_token_limit = 4000
history_budget = 0

[agents]
enabled = false

[execution]
max_model_retries = 0
stop_when_acceptance_met = true

[privacy]
telemetry = false
store_prompts = false
store_responses = false
```

Unknown keys warn. Unsafe v0.1 settings fail closed.

## Architecture

```text
request/context -> normalize + protected spans -> classify -> Context IR
  -> relevance/lifecycle/dedup/budget -> contract + fidelity guard
  -> Codex adapter -> JSONL usage/events -> bounded local metadata
```

- `crates/protocol`: stable serializable IR and result types.
- `crates/core`: pure compiler, policy, estimator and reducer.
- `crates/codex-adapter`: safe process construction, capability mapping and JSONL parser.
- `crates/cli`: commands, config, project map and bounded storage.

See [docs/architecture.md](docs/architecture.md), [docs/context-ir.md](docs/context-ir.md), [docs/policies.md](docs/policies.md), [docs/codex.md](docs/codex.md), [docs/privacy.md](docs/privacy.md) and [docs/threat-model.md](docs/threat-model.md).

## Privacy and security

ctxc adds no network service, API key or telemetry. Codex may still use its normal provider traffic. ctxc passes prompts through stdin, never shell-interpolates task text, avoids source-body project scans and redacts common secrets in diagnostics. Local history stores bounded metadata rather than prompt/response bodies. See [SECURITY.md](SECURITY.md).

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

CI never authenticates to or invokes a live model. Fake Codex fixtures cover JSONL, malformed events, quota errors, interruption and argument safety.

## Roadmap

- **v0.1:** Codex Foundation.
- **v0.2:** OpenAI Responses API middleware and TypeScript/Python packages.
- **v0.3:** ChatGPT skill-first and optional Apps SDK/MCP tool integration.
- **v0.4+:** persistent lifecycle, other providers and opt-in semantic optimizers after benchmarks.

See [CONTEXT_OPTIMIZER_MASTER_PLAN.md](CONTEXT_OPTIMIZER_MASTER_PLAN.md) for the complete specification and [docs/publishing.md](docs/publishing.md) for the GitHub release checklist.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every optimization must explain waste removed, fidelity risk, tests and inspection/disable controls. Do not commit credentials, private prompts, `.ctxc` state or unmeasured savings claims.

## License

MIT. See [LICENSE](LICENSE).
