# Codex Integration

ctxc runs `codex exec` with the compiled contract on stdin. It requests JSONL when available and parses `thread.*`, `item.*`, `turn.completed`, `turn.failed` and `error`. Malformed events are forwarded raw and usage becomes unavailable rather than fabricated.

The adapter never edits `~/.codex/config.toml`, never shell-interpolates task input and never performs automatic quota-consuming retries. Capability detection supports graceful operation across Codex versions.
