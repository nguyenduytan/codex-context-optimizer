# Installation and Releases

Native archives include the binary, `LICENSE`, `README.md` and a SHA256 entry in `SHA256SUMS`. Installers require an explicit semver tag, verify TLS/checksum, reject unexpected archive contents and do not modify PATH or global Codex config.

Before publication, run `scripts/set-repository.ps1 -Owner nguyenduytan`, review the diff, push `main`, tag `v0.1.0`, and inspect one release download on each OS. For a local build: `cargo build --release -p ctxc --locked` and `cargo test --workspace --locked`.
