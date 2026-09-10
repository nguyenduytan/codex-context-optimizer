# Compatibility

## Verified Interfaces

The local reference executable was Codex CLI 0.153.4 on Windows. Version/help probing was checked without a model call. Full model execution is covered by a fake runtime, not claimed as a live end-to-end result.

The 0.153.x config family maps reasoning effort, multi-agent enablement, tool-output budget and explicit web policy. Minimal intent maps to low because the adapter does not assume that the selected model accepts minimal. Extreme maps to xhigh, only by explicit user choice. Model/provider support still varies; an incompatible model produces its own error, and ctxc does not retry.

Unknown versions are not rejected solely for being newer. Detected JSONL, ephemeral and sandbox flags remain usable, while unverified native config keys are omitted. Requested effort/agents/web overrides fail before a model run if they cannot be mapped. See warnings in dry-run and doctor.

## Hard Controls Versus Guidance

- Native when verified: reasoning override, agents feature, tool-output history budget, explicit web option.
- Guidance only: targeted reads/tests, no repeated investigation, stop at acceptance, output verbosity.
- Not controlled: Codex's complete hidden context, upstream instructions, individual internal tool results, provider-side accounting or Codex's own internal transport retries.
- No project instruction-byte limit override: truncating unknown AGENTS.md constraints would be unsafe.
- No automatic compaction threshold override: selected model context limits are not inferred.

The standalone core reducer cannot intercept Codex-internal output; the adapter uses Codex's own tool-output limit. That upstream truncation policy is owned by Codex.

## Platforms

CI is configured for Windows, Linux and macOS; actual remote results are available only after pushing. Releases target GNU Linux x86_64/ARM64, macOS Intel/Apple Silicon, and Windows MSVC x86_64/ARM64. Windows GNU is also usable for local development. Source builds require Rust 1.98 or later.

Windows .cmd/.bat shims are deliberately not executed. Use --codex-bin with the native codex.exe shipped by the official installation. The CLI keeps Codex's existing sandbox/approval policy unless the user explicitly selects read-only or workspace-write.

## Known Limits

Classification is heuristic, with English signals and selected Vietnamese patterns; arbitrary tasks can be overridden with --mode. The original request is never semantically rewritten in v0.1. Preserving a long request can increase context; estimates include wrapper overhead. Persistent task history/resume and adaptive model capability discovery remain roadmap work.

TaskLedger is a serializable single-run model, not an automatic cross-run memory system. Stop conditions are instructions, not proof of task completion. Ctrl+C kills and waits for the direct Codex child; third-party descendants that detach from it are outside ctxc's process ownership.
