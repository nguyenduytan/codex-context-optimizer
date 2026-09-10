# Security Policy

Report vulnerabilities in ctxc, installers, process adapter, project-map handling, config parsing or local state through a private GitHub advisory: `https://github.com/nguyenduytan/codex-context-optimizer/security/advisories/new`. Never include credentials, auth files or private source in reports.

Security properties: prompts use stdin and explicit argv; passthrough does not remove process safety; maps do not read source bodies or secret-like files; local history excludes prompt/response bodies; state rejects symlinks/reparse points where supported; unsupported runtime controls fail closed; CI uses fake Codex only.
