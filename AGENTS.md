Keep changes scoped to the active task. Prefer targeted reads and tests.
Keep dependencies minimal; the core is deterministic, local-first and provider-independent.
Preserve constraints and exact values before optimizing token usage.
Never add telemetry, model calls or network dependencies to core.
Never consume live AI credits in tests or CI.
Run formatting, linting and relevant tests. Stop when acceptance criteria pass.
