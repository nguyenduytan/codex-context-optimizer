<h1 align="center">Codex Context Optimizer</h1>

<p align="center"><strong>Less context. More signal.</strong></p>

<p align="center">
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml"><img src="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/releases"><img src="https://img.shields.io/github/v/release/nguyenduytan/codex-context-optimizer" alt="Latest release"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Rust-1.98%2B-00875A" alt="Rust 1.98+"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="MIT License"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/stargazers"><img src="https://img.shields.io/github/stars/nguyenduytan/codex-context-optimizer?style=social" alt="GitHub stars"></a>
</p>

<p align="center"><strong>English</strong> · <a href="README.vi.md">Tiếng Việt</a> · <a href="README.zh-CN.md">简体中文</a></p>

<p align="center">A local-first context compiler and execution-budget governor for Codex and AI coding workflows.</p>

`ctxc` builds an inspectable execution contract around your task, protects explicit constraints, selects relevant context and applies supported runtime budget controls. The compiler runs locally, without a second model call.

> **Scope:** v0.1 focuses on Codex CLI. It cannot rewrite ChatGPT's hidden context or control private chain-of-thought. Token estimates are approximate; no percentage of quota savings is promised.

### 🧭 Explore

- [📦 Installation](#installation)
- [🚀 Quick Start](#quick-start)
- [🧰 Commands and Options](#commands)
- [🎯 How Optimization Works](#optimization)
- [⚙️ Configuration](#configuration)
- [🧱 Architecture](#architecture)
- [🔒 Privacy and Security](#privacy)
- [🧪 Development and Validation](#development)
- [🗺️ Roadmap](#roadmap)
- [🤝 Contributing](#contributing)
- [⭐ Star History](#star-history)
- [📄 License](#license)

<a id="installation"></a>

## 📦 Installation

### Requirements

- `ctxc plan` runs locally without Codex, authentication or an API key.
- `ctxc run` requires Codex CLI installed and authenticated separately.
- Native binaries do not require Rust. Building from source requires **Rust 1.98+**, as declared in this repository's `Cargo.toml`.
- Run inside a Git repository for normal Codex execution.

### Build from Source

Use this route if a matching release is not yet available:

```bash
git clone https://github.com/nguyenduytan/codex-context-optimizer.git
cd codex-context-optimizer
cargo install --path crates/cli --locked
ctxc --version
```

### Native Binaries

Check [GitHub Releases](https://github.com/nguyenduytan/codex-context-optimizer/releases) for published artifacts. The installers below default to `v0.1.0`; they require that tag and its assets to exist. A release badge is not proof that all platform downloads are available.

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.sh -o install-ctxc.sh
# Review the downloaded script before running it.
sh install-ctxc.sh
```

**Windows PowerShell**

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.ps1 -OutFile install-ctxc.ps1
# Review the downloaded script before running it.
.\install-ctxc.ps1
```

Installers verify `SHA256SUMS`. Their default destinations are `~/.local/bin` and `%LOCALAPPDATA%\ctxc\bin`; add the destination to PATH yourself if needed. They do not edit PATH or global Codex configuration.

Select a version/directory with `CTXC_VERSION` / `CTXC_INSTALL_DIR` (shell), or `-Version` / `-InstallDir` (PowerShell). Replacing an existing binary requires `CTXC_FORCE=1` or `-Force`.

The release workflow targets Linux x86_64/ARM64, macOS Intel/Apple Silicon and Windows x86_64/ARM64. Availability depends on successful release builds. See [installation details](docs/installation.md).

### Cargo Registry

Only after this project's `ctxc` package is published and its ownership is verified on crates.io:

```bash
cargo install ctxc --locked
```

<a id="quick-start"></a>

## 🚀 Quick Start

First inspect a task locally, then run it deliberately:

```bash
cd your-repository
ctxc init
ctxc plan "fix the failing auth test without changing the public API"
ctxc run --dry-run "fix the failing auth test without changing the public API"
ctxc "fix the failing auth test without changing the public API"
ctxc explain
ctxc usage
```

`ctxc "task"` is shorthand for `ctxc run "task"`. `plan` does not launch Codex; `doctor` and `run --dry-run` may probe version/help but do not start a model run. Only actual runs create usage history.

**Pass through the original task without optimization:**

```bash
ctxc run --passthrough "task"
```

**Choose higher effort explicitly:**

```bash
ctxc run --effort high "investigate this concurrency issue"
```

<a id="commands"></a>

## 🧰 Commands and Options

| Command | Behavior |
| --- | --- |
| `ctxc init` | Create project config, bounded map and private-state ignore rules; preserve existing config. |
| `ctxc doctor` | Check Codex, Git and configuration without model usage. |
| `ctxc plan <task>` | Compile locally; no Codex installation required. |
| `ctxc run <task>` | Compile and launch Codex once. |
| `ctxc run --dry-run <task>` | Show the contract, estimates and runtime invocation without a model run. |
| `ctxc explain` | Explain the latest recorded run using metadata and trace reasons. |
| `ctxc usage` | Display observed usage; missing fields remain unavailable. |
| `ctxc config` | Show effective configuration. |
| `ctxc config set context.budget 8000` | Validate and change one project key, retaining comments. |
| `ctxc setup codex` | Print integration guidance; do not edit global Codex config. |
| `ctxc setup agents` | Print optional AGENTS.md guidance for manual review. |
| `ctxc update / ctxc uninstall` | Print update/removal instructions; do not automatically update or delete files. |

### Common options

| Option | Behavior |
| --- | --- |
| `--mode` | `auto`, `chat`, `micro`, `code`, `complex` |
| `--effort` | `auto`, `minimal`, `low`, `medium`, `high`, `extreme` |
| `--agents / --web` | `auto`, `off`, `on` |
| `--verbosity` | `low`, `medium`, `high` (task guidance) |
| `--project-map-limit / --tool-output-limit` | Map bytes / tool-output tokens. |
| `--budget` | Context token budget including the configured reserve. |
| `--input FILE` | Read an explicit JSON `CompileInput`, instead of a positional task. |
| `--codex-bin PATH` | Use a specific native Codex executable. |
| `--sandbox` | Explicit `read-only` or `workspace-write`; otherwise inherit Codex settings. |
| `--json / --trace / -C DIR` | Structured output / decision trace / project directory. |

Use `ctxc --help` or `ctxc run --help` for the complete CLI interface.

<a id="optimization"></a>

## 🎯 How Optimization Works

| Task | Default policy |
| --- | --- |
| `CHAT` | Minimal intent; no ctxc repository scan/map. |
| `MICRO` | Low effort and narrow validation guidance; no generated repository map. |
| `CODE` | Low effort, bounded map, targeted investigation. |
| `COMPLEX` | Medium effort for architecture, migration, security or concurrency signals. |

Automatic effort never exceeds medium. Agents are off by default, and ctxc does not automatically retry model runs. The adapter maps intent to available runtime controls: for the verified Codex family, minimal maps to low. Model/provider support can differ.

Protected content includes negations, paths, literals, identifiers, numbers, commands and acceptance constraints. v0.1 preserves the original request instead of attempting arbitrary semantic rewriting.

**Request**

```text
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.
```

**Contract excerpt**

```text
[TASK]
Objective / original request (preserved):
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.

Execution: Use the smallest sufficient investigation and targeted tests.
```

The compiler budgets supplied context, removes exact duplicates and prunes eligible inactive blocks. Protected content may exceed the budget rather than be lost. Short requests may become longer due to contract overhead.

The default project map is bounded to **8 KiB**. It uses paths/metadata, honors local ignore rules and excludes dependency/build trees and secret-like names without reading source bodies.

The standalone core log reducer preserves failure evidence. The Codex adapter does **not** intercept internal tool output; it applies Codex's native output-history limit when supported. Stop conditions and targeted-test preferences are guidance, not hard enforcement.

See [compatibility and limits](docs/compatibility.md) and [benchmarking](docs/benchmarking.md).

<a id="configuration"></a>

## ⚙️ Configuration

`ctxc init` creates `.ctxc/config.toml`. For ordinary prompt commands, precedence is **CLI flags → project config → user config → defaults**. `--input` supplies its own protocol policy; CLI flags can override it.

```toml
version = 1

[reasoning]
default = "low"
complex = "medium"
max_auto = "medium"

[context]
project_map_max_bytes = 8192
tool_output_token_limit = 4000
budget = 12000
reserve = 2000
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

User config is read from `CTXC_USER_CONFIG`, then the applicable location: `$XDG_CONFIG_HOME/ctxc/config.toml`, `%APPDATA%/ctxc/config.toml` on Windows, or `~/.config/ctxc/config.toml` on Unix. Unknown keys warn; unsupported safety settings are rejected. These are **ctxc policy keys**, not a file to paste into Codex global config.

<a id="architecture"></a>

## 🧱 Architecture

```text
request + context
  -> normalize + protect -> classify -> Context IR
  -> relevance + lifecycle + dedup + budget
  -> contract + fidelity guard
  -> Codex adapter -> JSONL events -> local metadata
```

- [`crates/protocol`](crates/protocol): Versioned serializable IR and result types.
- [`crates/core`](crates/core): Pure compiler, policy engine, estimator and log reducer.
- [`crates/codex-adapter`](crates/codex-adapter): Native process execution, compatibility mapping and JSONL parser.
- [`crates/cli`](crates/cli): Commands, project config, repository map and metadata storage.

Technical references: [architecture](docs/architecture.md), [CIR](docs/context-ir.md), [policies](docs/policies.md) and [Codex adapter](docs/codex.md).

<a id="privacy"></a>

## 🔒 Privacy and Security

- No extra model call, ctxc service, API key or telemetry.
- Core processing is local. Codex may still send data through its normal provider connection.
- Run history stores bounded metadata, not prompt/response bodies.
- Prompts travel over stdin, never through shell interpolation.
- Common secret patterns are redacted from diagnostics. Explicit plan output shows the contract, so treat it as private.
- `ctxc init` does not overwrite AGENTS.md or global Codex config.

See [privacy](docs/privacy.md), [threat model](docs/threat-model.md) and [security reporting](SECURITY.md).

<a id="development"></a>

## 🧪 Development and Validation

Run the repository checks with Rust 1.98+:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
```

CI covers Windows, Linux and macOS. Tests use a fake Codex executable, not live AI credits. A red CI badge means a check needs investigation; use the failed job's log for the actual cause. See [contributing](CONTRIBUTING.md).

<a id="roadmap"></a>

## 🗺️ Roadmap

- **v0.1:** Codex Foundation: compiler, policy, native CLI and adapter.
- **v0.2:** OpenAI Responses API middleware and TypeScript/Python packages.
- **v0.3:** ChatGPT skill-first integration and optional Apps SDK/MCP tools.
- **v0.4+:** Persistent lifecycle, additional providers and opt-in semantic optimizers after benchmarks.

<a id="contributing"></a>

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Describe waste removed, fidelity risk, tests and inspection/disable controls. Keep all three README translations in sync; never commit credentials, private prompts or private `.ctxc` state.

<a id="star-history"></a>

## ⭐ Star History

<p align="center">
  <a href="https://www.star-history.com/#nguyenduytan/codex-context-optimizer&amp;Date">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date&amp;theme=dark">
      <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date">
      <img src="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date" alt="Star history for Codex Context Optimizer" width="800">
    </picture>
  </a>
</p>

Live chart supplied by Star History. If the image is temporarily unavailable, open the interactive chart below; external services and GitHub's image cache may refresh at different times.

[Open interactive Star History](https://www.star-history.com/#nguyenduytan/codex-context-optimizer&Date)

<a id="license"></a>

## 📄 License

Released under the **MIT License**. See [LICENSE](LICENSE).
