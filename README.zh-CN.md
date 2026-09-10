<h1 align="center">Codex Context Optimizer</h1>

<p align="center"><strong>减少冗余上下文，保留关键信息。</strong></p>

<p align="center">
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml"><img src="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/releases"><img src="https://img.shields.io/github/v/release/nguyenduytan/codex-context-optimizer" alt="Latest release"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Rust-1.98%2B-00875A" alt="Rust 1.98+"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="MIT License"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/stargazers"><img src="https://img.shields.io/github/stars/nguyenduytan/codex-context-optimizer?style=social" alt="GitHub stars"></a>
</p>

<p align="center"><a href="README.md">English</a> · <a href="README.vi.md">Tiếng Việt</a> · <strong>简体中文</strong></p>

<p align="center">面向 Codex 和 AI 编程工作流的本地优先上下文编译器与执行预算管理工具。</p>

`ctxc` 为任务生成可检查的执行约定，保护明确约束，选择相关上下文，并应用运行时支持的预算控制。编译器在本地运行，无需额外调用模型。

> **适用范围：** v0.1 专注于 Codex CLI。它无法改写 ChatGPT 的隐藏上下文，也无法控制模型内部思维链。Token 数量仅为估算，不承诺固定比例的配额节省。

### 🧭 目录

- [📦 安装](#installation)
- [🚀 快速开始](#quick-start)
- [🧰 命令与选项](#commands)
- [🎯 优化原理](#optimization)
- [⚙️ 配置](#configuration)
- [🧱 架构](#architecture)
- [🔒 隐私与安全](#privacy)
- [🧪 开发与验证](#development)
- [🗺️ 路线图](#roadmap)
- [🤝 参与贡献](#contributing)
- [⭐ Star 历史](#star-history)
- [📄 许可证](#license)

<a id="installation"></a>

## 📦 安装

### 环境要求

- `ctxc plan` 在本地运行，无需 Codex、登录或 API key。
- `ctxc run` 需要单独安装 Codex CLI 并完成登录。
- 预编译二进制文件无需 Rust。从源码构建需要 **Rust 1.98+**，与本仓库的 `Cargo.toml` 一致。
- 常规 Codex 执行应在 Git 仓库中运行。

### 从源码构建

尚无适用发布包时，请使用此方式：

```bash
git clone https://github.com/nguyenduytan/codex-context-optimizer.git
cd codex-context-optimizer
cargo install --path crates/cli --locked
ctxc --version
```

### 预编译二进制文件

在 [GitHub Releases](https://github.com/nguyenduytan/codex-context-optimizer/releases) 查看已发布的构建产物。安装脚本默认使用 `v0.1.0`，相应标签和文件必须已发布。Release 徽章不代表所有平台都有可用下载。

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.sh -o install-ctxc.sh
# 执行前请先检查下载的脚本。
sh install-ctxc.sh
```

**Windows PowerShell**

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.ps1 -OutFile install-ctxc.ps1
# 执行前请先检查下载的脚本。
.\install-ctxc.ps1
```

安装脚本验证 `SHA256SUMS`。默认安装位置为 `~/.local/bin` 和 `%LOCALAPPDATA%\ctxc\bin`；如有需要，请自行加入 PATH。脚本不会修改 PATH 或全局 Codex 配置。

Shell 使用 `CTXC_VERSION` / `CTXC_INSTALL_DIR` 选择版本和目录；PowerShell 使用 `-Version` / `-InstallDir`。覆盖现有二进制文件需要显式设置 `CTXC_FORCE=1` 或 `-Force`。

发布工作流面向 Linux x86_64/ARM64、macOS Intel/Apple Silicon、Windows x86_64/ARM64。实际可用性取决于发布构建结果。参阅[安装说明](docs/installation.md)。

### Cargo Registry

仅在本项目的 `ctxc` 包已发布、且 crates.io 上的所有者身份已确认后使用：

```bash
cargo install ctxc --locked
```

<a id="quick-start"></a>

## 🚀 快速开始

先在本地检查任务，再主动运行 Codex：

```bash
cd your-repository
ctxc init
ctxc plan "fix the failing auth test without changing the public API"
ctxc run --dry-run "fix the failing auth test without changing the public API"
ctxc "fix the failing auth test without changing the public API"
ctxc explain
ctxc usage
```

`ctxc "task"` 是 `ctxc run "task"` 的简写。`plan` 不启动 Codex；`doctor` 和 `run --dry-run` 可能查询版本或帮助信息，但不会启动模型执行。只有实际运行才记录使用量历史。

**原样传递任务，跳过优化：**

```bash
ctxc run --passthrough "task"
```

**显式选择更高推理强度：**

```bash
ctxc run --effort high "investigate this concurrency issue"
```

<a id="commands"></a>

## 🧰 命令与选项

| 命令 | 行为 |
| --- | --- |
| `ctxc init` | 创建项目配置、有界项目映射和私有状态忽略规则；保留现有配置。 |
| `ctxc doctor` | 检查 Codex、Git 和配置，不消耗模型使用量。 |
| `ctxc plan <task>` | 本地编译，无需安装 Codex。 |
| `ctxc run <task>` | 编译并启动一次 Codex。 |
| `ctxc run --dry-run <task>` | 显示执行约定、估算和运行时调用，不执行模型。 |
| `ctxc explain` | 通过元数据和决策记录解释最近一次已记录的运行。 |
| `ctxc usage` | 显示观测到的使用量；缺失字段保持未知。 |
| `ctxc config` | 显示生效配置。 |
| `ctxc config set context.budget 8000` | 验证并修改一个项目配置项，保留注释。 |
| `ctxc setup codex` | 输出集成说明，不修改全局配置。 |
| `ctxc setup agents` | 输出可选 AGENTS.md 建议，供用户手动审阅。 |
| `ctxc update / ctxc uninstall` | 输出更新或卸载说明，不自动更新或删除文件。 |

### 常用选项

| Option | 行为 |
| --- | --- |
| `--mode` | `auto`、`chat`、`micro`、`code`、`complex` |
| `--effort` | `auto`、`minimal`、`low`、`medium`、`high`、`extreme` |
| `--agents / --web` | `auto`、`off`、`on` |
| `--verbosity` | `low`、`medium`、`high`（任务指引） |
| `--project-map-limit / --tool-output-limit` | 项目映射字节数 / 工具输出 token 数。 |
| `--budget` | 包含已配置预留部分的上下文 token 预算。 |
| `--input FILE` | 从 JSON `CompileInput` 读取输入，而非位置参数任务。 |
| `--codex-bin PATH` | 指定原生 Codex 可执行文件。 |
| `--sandbox` | 显式选择 `read-only` 或 `workspace-write`；否则继承 Codex 设置。 |
| `--json / --trace / -C DIR` | 结构化输出 / 决策记录 / 项目目录。 |

使用 `ctxc --help` 或 `ctxc run --help` 查看完整命令说明。

<a id="optimization"></a>

## 🎯 优化原理

| 任务类型 | 默认策略 |
| --- | --- |
| `CHAT` | 最低推理意图；ctxc 不扫描仓库或生成映射。 |
| `MICRO` | 低推理强度、窄范围验证指引；不自动生成项目映射。 |
| `CODE` | 低推理强度、有界映射、针对性调查。 |
| `COMPLEX` | 遇到架构、迁移、安全或并发信号时使用 medium。 |

自动推理强度不会超过 medium。默认关闭多代理，ctxc 不会自动重试模型运行。适配器将策略映射到运行时支持的控制：在已验证的 Codex 版本系列中，minimal 映射为 low。不同模型或提供商的支持情况可能不同。

受保护内容包括否定约束、路径、字面量、标识符、数值、命令和验收条件。v0.1 保留原始请求，不尝试任意语义改写。

**请求**

```text
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.
```

**执行约定节选**

```text
[TASK]
Objective / original request (preserved):
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.

Execution: Use the smallest sufficient investigation and targeted tests.
```

编译器为输入上下文分配预算，移除完全重复内容，并清理符合条件的非活动块。受保护内容可能超出预算，但不会因此丢失。短请求可能因附加执行指引而变长。

项目映射默认上限为 **8 KiB**，使用路径与元数据，遵循本地忽略规则，排除依赖、构建目录和疑似敏感文件名，不读取源码正文。

核心库中的独立日志缩减器保留失败证据。Codex 适配器**不会**拦截内部工具输出，而是在支持时应用 Codex 原生输出历史限制。停止条件和定向测试偏好属于指引，不是强制执行机制。

参阅[兼容性与限制](docs/compatibility.md)和[基准测试](docs/benchmarking.md)。

<a id="configuration"></a>

## ⚙️ 配置

`ctxc init` 创建 `.ctxc/config.toml`。普通任务命令的优先级为 **CLI 参数 → 项目配置 → 用户配置 → 默认值**。`--input` 提供协议内的独立策略，CLI 参数可以覆盖它。

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

用户配置来自 `CTXC_USER_CONFIG`，或适用路径：`$XDG_CONFIG_HOME/ctxc/config.toml`、Windows 的 `%APPDATA%/ctxc/config.toml`、Unix 的 `~/.config/ctxc/config.toml`。未知键会产生警告；不支持的安全设置会被拒绝。这些是 **ctxc 策略键**，不是可直接粘贴到全局 Codex 配置的内容。

<a id="architecture"></a>

## 🧱 架构

```text
request + context
  -> normalize + protect -> classify -> Context IR
  -> relevance + lifecycle + dedup + budget
  -> contract + fidelity guard
  -> Codex adapter -> JSONL events -> local metadata
```

- [`crates/protocol`](crates/protocol): 带版本的可序列化 IR 和结果类型。
- [`crates/core`](crates/core): 纯编译器、策略引擎、估算器与日志缩减器。
- [`crates/codex-adapter`](crates/codex-adapter): 原生进程执行、兼容性映射与 JSONL 解析。
- [`crates/cli`](crates/cli): 命令、项目配置、仓库映射与元数据存储。

技术文档：[架构](docs/architecture.md)、[CIR](docs/context-ir.md)、[策略](docs/policies.md)、[Codex 适配器](docs/codex.md)。

<a id="privacy"></a>

## 🔒 隐私与安全

- 不额外调用模型，不需要 ctxc 服务、独立 API key 或遥测。
- 核心处理在本地进行。Codex 仍可能通过正常的提供商连接发送数据。
- 运行历史保存有界元数据，不保存提示词或响应正文。
- 提示词通过 stdin 传递，不进行 shell 插值。
- 诊断输出会遮盖常见敏感模式。显式 plan 输出包含执行约定，应视为私有数据。
- `ctxc init` 不覆盖 AGENTS.md 或全局 Codex 配置。

参阅[隐私](docs/privacy.md)、[威胁模型](docs/threat-model.md)、[安全报告](SECURITY.md)。

<a id="development"></a>

## 🧪 开发与验证

使用 Rust 1.98+ 运行仓库检查：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
```

CI 覆盖 Windows、Linux 和 macOS。测试使用模拟 Codex 可执行文件，不消耗真实 AI 配额。红色 CI 徽章表示需要检查失败步骤，具体原因应以失败任务日志为准。参阅[贡献指南](CONTRIBUTING.md)。

<a id="roadmap"></a>

## 🗺️ 路线图

- **v0.1:** Codex 基础版本：编译器、策略、原生 CLI 与适配器。
- **v0.2:** OpenAI Responses API 中间件和 TypeScript/Python 包。
- **v0.3:** 优先采用 skill 的 ChatGPT 集成，可选 Apps SDK/MCP 工具。
- **v0.4+:** 持久上下文生命周期、更多提供商，以及经过基准验证的可选语义优化。

<a id="contributing"></a>

## 🤝 参与贡献

参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。说明消除的浪费、语义保真风险、测试和检查/禁用方式。保持三种 README 语言同步；不要提交凭据、私有提示词或 `.ctxc` 私有状态。

<a id="star-history"></a>

## ⭐ Star 历史

<p align="center">
  <a href="https://www.star-history.com/#nguyenduytan/codex-context-optimizer&amp;Date">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date&amp;theme=dark">
      <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date">
      <img src="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date" alt="Codex Context Optimizer 的 GitHub Star 历史" width="800">
    </picture>
  </a>
</p>

实时图表由 Star History 提供。如果图片暂时无法加载，请打开下方交互式图表；外部服务和 GitHub 图片缓存的更新时间可能不同。

[打开交互式 Star History 图表](https://www.star-history.com/#nguyenduytan/codex-context-optimizer&Date)

<a id="license"></a>

## 📄 许可证

采用 **MIT License**。参阅 [LICENSE](LICENSE)。
