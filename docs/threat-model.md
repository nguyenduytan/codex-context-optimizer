# Threat Model

| Threat | Control |
| --- | --- |
| Prompt becomes shell injection | stdin and explicit argv; no shell command string |
| Secret enters project map | no source-body scan; secret-like names excluded |
| Symlink redirects state | symlink/reparse checks and bounded targets |
| Stale context weakens constraints | exact/semantic protection and fidelity guard |
| New Codex rejects guessed option | capability detection and fail-closed controls |
| Huge logs exhaust context | failure-aware reducer and truncation metadata |
| Quota multiplies by retries | model retries default to zero |

ctxc is not a sandbox or secret manager. Review Codex permissions, auth and repository trust separately.
