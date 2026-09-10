<h1 align="center">Codex Context Optimizer</h1>

<p align="center"><strong>Ít ngữ cảnh thừa. Nhiều thông tin hữu ích.</strong></p>

<p align="center">
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml"><img src="https://github.com/nguyenduytan/codex-context-optimizer/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/releases"><img src="https://img.shields.io/github/v/release/nguyenduytan/codex-context-optimizer" alt="Latest release"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Rust-1.98%2B-00875A" alt="Rust 1.98+"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="MIT License"></a>
  <a href="https://github.com/nguyenduytan/codex-context-optimizer/stargazers"><img src="https://img.shields.io/github/stars/nguyenduytan/codex-context-optimizer?style=social" alt="GitHub stars"></a>
</p>

<p align="center"><a href="README.md">English</a> · <strong>Tiếng Việt</strong> · <a href="README.zh-CN.md">简体中文</a></p>

<p align="center">Trình biên dịch ngữ cảnh và điều phối ngân sách thực thi, ưu tiên xử lý cục bộ cho Codex và quy trình lập trình với AI.</p>

`ctxc` tạo một bản yêu cầu thực thi có thể kiểm tra, bảo vệ ràng buộc rõ ràng, chọn ngữ cảnh liên quan và áp dụng các giới hạn mà runtime hỗ trợ. Compiler chạy trên máy, không cần gọi thêm một model.

> **Phạm vi:** v0.1 tập trung vào Codex CLI. Công cụ không viết lại ngữ cảnh ẩn của ChatGPT hay điều khiển chuỗi suy luận nội bộ. Số token chỉ là ước tính; không cam kết phần trăm tiết kiệm quota.

### 🧭 Mục lục

- [📦 Cài đặt](#installation)
- [🚀 Bắt đầu nhanh](#quick-start)
- [🧰 Lệnh và tùy chọn](#commands)
- [🎯 Cách tối ưu hoạt động](#optimization)
- [⚙️ Cấu hình](#configuration)
- [🧱 Kiến trúc](#architecture)
- [🔒 Riêng tư và bảo mật](#privacy)
- [🧪 Phát triển và kiểm thử](#development)
- [🗺️ Lộ trình](#roadmap)
- [🤝 Đóng góp](#contributing)
- [⭐ Lịch sử Star](#star-history)
- [📄 Giấy phép](#license)

<a id="installation"></a>

## 📦 Cài đặt

### Yêu cầu

- `ctxc plan` chạy cục bộ, không cần Codex, đăng nhập hay API key.
- `ctxc run` cần cài Codex CLI và đăng nhập riêng.
- Binary dựng sẵn không cần Rust. Build từ source cần **Rust 1.98+**, theo `Cargo.toml` của repository.
- Chạy trong Git repository để dùng luồng thực thi Codex thông thường.

### Build từ source

Dùng cách này nếu chưa có release phù hợp:

```bash
git clone https://github.com/nguyenduytan/codex-context-optimizer.git
cd codex-context-optimizer
cargo install --path crates/cli --locked
ctxc --version
```

### Binary dựng sẵn

Kiểm tra các artifact đã phát hành tại [GitHub Releases](https://github.com/nguyenduytan/codex-context-optimizer/releases). Installer mặc định dùng `v0.1.0`; tag và artifact tương ứng phải tồn tại. Badge release không có nghĩa mọi nền tảng đều đã có bản tải xuống.

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.sh -o install-ctxc.sh
# Đọc script đã tải trước khi chạy.
sh install-ctxc.sh
```

**Windows PowerShell**

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/nguyenduytan/codex-context-optimizer/main/scripts/install.ps1 -OutFile install-ctxc.ps1
# Đọc script đã tải trước khi chạy.
.\install-ctxc.ps1
```

Installer xác minh `SHA256SUMS`. Thư mục mặc định là `~/.local/bin` và `%LOCALAPPDATA%\ctxc\bin`; tự thêm thư mục vào PATH nếu cần. Installer không sửa PATH hay cấu hình Codex toàn cục.

Chọn phiên bản/thư mục bằng `CTXC_VERSION` / `CTXC_INSTALL_DIR` (shell), hoặc `-Version` / `-InstallDir` (PowerShell). Ghi đè binary đã có cần `CTXC_FORCE=1` hoặc `-Force`.

Workflow release nhắm tới Linux x86_64/ARM64, macOS Intel/Apple Silicon và Windows x86_64/ARM64. Bản tải thực tế phụ thuộc kết quả build release. Xem [chi tiết cài đặt](docs/installation.md).

### Cargo Registry

Chỉ dùng sau khi package `ctxc` của dự án được phát hành và xác minh đúng chủ sở hữu trên crates.io:

```bash
cargo install ctxc --locked
```

<a id="quick-start"></a>

## 🚀 Bắt đầu nhanh

Kiểm tra yêu cầu tại local trước, sau đó chủ động chạy Codex:

```bash
cd your-repository
ctxc init
ctxc plan "fix the failing auth test without changing the public API"
ctxc run --dry-run "fix the failing auth test without changing the public API"
ctxc "fix the failing auth test without changing the public API"
ctxc explain
ctxc usage
```

`ctxc "task"` là viết tắt của `ctxc run "task"`. `plan` không khởi chạy Codex; `doctor` và `run --dry-run` có thể đọc version/help nhưng không khởi chạy model. Chỉ lượt chạy thật mới tạo lịch sử usage.

**Chuyển nguyên yêu cầu, bỏ qua tối ưu:**

```bash
ctxc run --passthrough "task"
```

**Chủ động chọn reasoning cao hơn:**

```bash
ctxc run --effort high "investigate this concurrency issue"
```

<a id="commands"></a>

## 🧰 Lệnh và tùy chọn

| Lệnh | Hành vi |
| --- | --- |
| `ctxc init` | Tạo config, project map có giới hạn và quy tắc bỏ qua trạng thái riêng; giữ config đã có. |
| `ctxc doctor` | Kiểm tra Codex, Git và cấu hình, không dùng quota model. |
| `ctxc plan <task>` | Biên dịch cục bộ; không cần cài Codex. |
| `ctxc run <task>` | Biên dịch và khởi chạy Codex một lần. |
| `ctxc run --dry-run <task>` | Hiển thị contract, ước tính và lệnh runtime mà không chạy model. |
| `ctxc explain` | Giải thích lượt chạy đã ghi nhận gần nhất bằng metadata và trace. |
| `ctxc usage` | Hiển thị usage quan sát được; trường thiếu không bị gán thành số 0. |
| `ctxc config` | Hiển thị cấu hình có hiệu lực. |
| `ctxc config set context.budget 8000` | Kiểm tra và sửa một khóa config project, giữ comment. |
| `ctxc setup codex` | In hướng dẫn tích hợp; không sửa config toàn cục. |
| `ctxc setup agents` | In gợi ý AGENTS.md để bạn tự xem xét. |
| `ctxc update / ctxc uninstall` | In hướng dẫn cập nhật/gỡ bỏ; không tự cập nhật hay xóa file. |

### Các tùy chọn thường dùng

| Option | Hành vi |
| --- | --- |
| `--mode` | `auto`, `chat`, `micro`, `code`, `complex` |
| `--effort` | `auto`, `minimal`, `low`, `medium`, `high`, `extreme` |
| `--agents / --web` | `auto`, `off`, `on` |
| `--verbosity` | `low`, `medium`, `high` (chỉ dẫn trong task) |
| `--project-map-limit / --tool-output-limit` | Số byte của map / số token output công cụ. |
| `--budget` | Ngân sách token ngữ cảnh, bao gồm phần dự trữ đã cấu hình. |
| `--input FILE` | Đọc JSON `CompileInput` thay cho task nhập trực tiếp. |
| `--codex-bin PATH` | Chọn executable Codex native cụ thể. |
| `--sandbox` | Chọn `read-only` hoặc `workspace-write`; mặc định kế thừa Codex. |
| `--json / --trace / -C DIR` | Output có cấu trúc / trace quyết định / thư mục project. |

Dùng `ctxc --help` hoặc `ctxc run --help` để xem giao diện CLI đầy đủ.

<a id="optimization"></a>

## 🎯 Cách tối ưu hoạt động

| Loại task | Chính sách mặc định |
| --- | --- |
| `CHAT` | Ý định reasoning tối thiểu; ctxc không quét repo hay tạo map. |
| `MICRO` | Reasoning thấp, gợi ý kiểm thử hẹp; không tự tạo project map. |
| `CODE` | Reasoning thấp, map có giới hạn, điều tra đúng phạm vi. |
| `COMPLEX` | Reasoning medium khi có tín hiệu kiến trúc, migration, bảo mật hoặc concurrency. |

Reasoning tự động không vượt medium. Agents mặc định tắt và ctxc không tự retry lượt chạy model. Adapter ánh xạ mức mong muốn sang runtime hỗ trợ: với nhóm phiên bản Codex đã xác minh, minimal được ánh xạ thành low. Model/provider có thể hỗ trợ khác nhau.

Nội dung được bảo vệ gồm phủ định, đường dẫn, literal, identifier, số, command và điều kiện nghiệm thu. v0.1 giữ nguyên yêu cầu gốc thay vì cố viết lại ngữ nghĩa tùy ý.

**Yêu cầu**

```text
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.
```

**Trích đoạn contract**

```text
[TASK]
Objective / original request (preserved):
Fix refresh token expiration in src/auth.ts. Do NOT change the public API.

Execution: Use the smallest sufficient investigation and targeted tests.
```

Compiler phân bổ ngân sách cho context đầu vào, loại trùng chính xác và bỏ các block hết hiệu lực đủ điều kiện. Nội dung được bảo vệ có thể vượt budget để tránh bị mất. Yêu cầu ngắn có thể dài hơn do phần hướng dẫn bổ sung.

Project map mặc định giới hạn **8 KiB**, lấy đường dẫn/metadata, tuân thủ ignore cục bộ, bỏ qua dependency/build và tên file có dấu hiệu chứa secret, không đọc source body.

Bộ rút gọn log trong core giữ bằng chứng lỗi. Adapter **không** chặn output công cụ nội bộ của Codex; nó áp dụng giới hạn lịch sử output do Codex hỗ trợ. Điều kiện dừng và ưu tiên test hẹp là chỉ dẫn, không phải cơ chế cưỡng chế.

Xem [tương thích và giới hạn](docs/compatibility.md), [benchmark](docs/benchmarking.md).

<a id="configuration"></a>

## ⚙️ Cấu hình

`ctxc init` tạo `.ctxc/config.toml`. Với task nhập thông thường, ưu tiên là **CLI flags → config project → config user → mặc định**. `--input` cung cấp policy riêng theo protocol; CLI flags có thể ghi đè.

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

Config user được đọc từ `CTXC_USER_CONFIG`, hoặc vị trí phù hợp: `$XDG_CONFIG_HOME/ctxc/config.toml`, `%APPDATA%/ctxc/config.toml` trên Windows, `~/.config/ctxc/config.toml` trên Unix. Khóa lạ sinh cảnh báo; thiết lập an toàn không hỗ trợ bị từ chối. Đây là **policy của ctxc**, không phải file để dán vào config Codex toàn cục.

<a id="architecture"></a>

## 🧱 Kiến trúc

```text
request + context
  -> normalize + protect -> classify -> Context IR
  -> relevance + lifecycle + dedup + budget
  -> contract + fidelity guard
  -> Codex adapter -> JSONL events -> local metadata
```

- [`crates/protocol`](crates/protocol): IR và kiểu kết quả có version, hỗ trợ serialize.
- [`crates/core`](crates/core): Compiler thuần, policy engine, ước tính token và rút gọn log.
- [`crates/codex-adapter`](crates/codex-adapter): Chạy tiến trình native, ánh xạ tương thích và parser JSONL.
- [`crates/cli`](crates/cli): CLI, config project, project map và lưu metadata.

Tài liệu kỹ thuật: [kiến trúc](docs/architecture.md), [CIR](docs/context-ir.md), [policies](docs/policies.md), [Codex adapter](docs/codex.md).

<a id="privacy"></a>

## 🔒 Riêng tư và bảo mật

- Không gọi model phụ, không cần dịch vụ ctxc, API key riêng hay telemetry.
- Core xử lý tại local. Codex vẫn có thể gửi dữ liệu qua kết nối provider thông thường.
- Lịch sử lưu metadata có giới hạn, không lưu nội dung prompt/response.
- Prompt truyền qua stdin, không nội suy vào shell.
- Các mẫu secret phổ biến được che trong chẩn đoán. Lệnh plan hiển thị contract, nên cần coi output đó là dữ liệu riêng.
- `ctxc init` không ghi đè AGENTS.md hoặc config Codex toàn cục.

Xem [riêng tư](docs/privacy.md), [threat model](docs/threat-model.md), [báo cáo bảo mật](SECURITY.md).

<a id="development"></a>

## 🧪 Phát triển và kiểm thử

Chạy kiểm tra repository với Rust 1.98+:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
```

CI bao gồm Windows, Linux và macOS. Tests dùng fake Codex, không tiêu tốn AI credits thật. Badge CI đỏ nghĩa là có bước cần kiểm tra; log của job thất bại mới cho biết nguyên nhân. Xem [hướng dẫn đóng góp](CONTRIBUTING.md).

<a id="roadmap"></a>

## 🗺️ Lộ trình

- **v0.1:** Nền tảng Codex: compiler, policy, CLI native và adapter.
- **v0.2:** OpenAI Responses API middleware, package TypeScript/Python.
- **v0.3:** Tích hợp ChatGPT ưu tiên skill, tùy chọn Apps SDK/MCP.
- **v0.4+:** Lifecycle dài hạn, provider khác và tối ưu ngữ nghĩa opt-in sau benchmark.

<a id="contributing"></a>

## 🤝 Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) và [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Nêu rõ phần lãng phí được loại bỏ, rủi ro mất ý nghĩa, test và cách kiểm tra/tắt tối ưu. Giữ ba bản README đồng bộ; không commit credential, prompt riêng hay trạng thái riêng trong `.ctxc`.

<a id="star-history"></a>

## ⭐ Lịch sử Star

<p align="center">
  <a href="https://www.star-history.com/#nguyenduytan/codex-context-optimizer&amp;Date">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date&amp;theme=dark">
      <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date">
      <img src="https://api.star-history.com/svg?repos=nguyenduytan/codex-context-optimizer&amp;type=Date" alt="Lịch sử GitHub Star của Codex Context Optimizer" width="800">
    </picture>
  </a>
</p>

Biểu đồ trực tiếp do Star History cung cấp. Nếu ảnh tạm thời không tải được, mở biểu đồ tương tác bên dưới; dịch vụ ngoài và bộ nhớ đệm ảnh GitHub có thể cập nhật khác thời điểm.

[Mở biểu đồ Star History tương tác](https://www.star-history.com/#nguyenduytan/codex-context-optimizer&Date)

<a id="license"></a>

## 📄 Giấy phép

Phát hành theo **MIT License**. Xem [LICENSE](LICENSE).
