# Publishing Checklist

1. Use the public slug `codex-context-optimizer` and description `Local-first context compiler and execution-budget governor for Codex and AI workflows.`
2. Repository metadata already targets `nguyenduytan/codex-context-optimizer`. Only run `scripts/set-repository.ps1 -Owner OWNER` if ownership or slug changes, then review every URL.
3. Ensure no placeholder, secret, `.ctxc`, target artifact or private prompt is committed.
4. Run fmt, clippy, tests and `git diff --check`.
5. Initialize Git if needed, commit, add your GitHub remote and push the default branch.
6. Tag `v0.1.0`; CI creates native archives and SHA256 checksums.
7. Verify downloads before announcing. Do not claim savings until benchmark data exists.
