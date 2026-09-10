#!/bin/sh
# Explicit download only. No sudo, daemon, telemetry or PATH edits.
set -eu
repo=${CTXC_REPOSITORY:-nguyenduytan/codex-context-optimizer}
version=${CTXC_VERSION:-v0.1.0}
install_dir=${CTXC_INSTALL_DIR:-"$HOME/.local/bin"}

printf '%s\n' "$repo" | LC_ALL=C grep -Eq '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$' || exit 1
printf '%s\n' "$version" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$' || exit 1
case "$install_dir" in /*) ;; *) printf '%s\n' 'Install directory must be absolute.' >&2; exit 1;; esac
for tool in curl tar mktemp; do command -v "$tool" >/dev/null 2>&1 || { printf 'Missing command: %s\n' "$tool" >&2; exit 1; }; done
case "$(uname -s)/$(uname -m)" in
  Linux/x86_64) target=x86_64-unknown-linux-gnu;;
  Linux/aarch64|Linux/arm64) target=aarch64-unknown-linux-gnu;;
  Darwin/x86_64) target=x86_64-apple-darwin;;
  Darwin/arm64) target=aarch64-apple-darwin;;
  *) printf '%s\n' 'Unsupported platform; use a matching release or source build.' >&2; exit 1;;
esac
[ ! -L "$install_dir" ] && [ ! -L "$install_dir/ctxc" ] || { printf '%s\n' 'Refusing linked install target.' >&2; exit 1; }
if [ -e "$install_dir/ctxc" ] && [ "${CTXC_FORCE:-0}" != 1 ]; then
    printf '%s\n' 'ctxc exists. Set CTXC_FORCE=1 to explicitly replace it.' >&2; exit 1
fi
asset="ctxc-$version-$target.tar.gz"
base="https://github.com/$repo/releases/download/$version"
temp_dir=$(mktemp -d)
stage=''
cleanup() {
    rm -f "$temp_dir/archive.tar.gz" "$temp_dir/SHA256SUMS" "$temp_dir/ctxc"
    if [ -n "$stage" ]; then rm -f "$stage"; fi
    rmdir "$temp_dir" 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM
curl --fail --location --proto '=https' --tlsv1.2 "$base/$asset" --output "$temp_dir/archive.tar.gz"
curl --fail --location --proto '=https' --tlsv1.2 "$base/SHA256SUMS" --output "$temp_dir/SHA256SUMS"
expected=$(awk -v name="$asset" '$2 == name { print $1 }' "$temp_dir/SHA256SUMS")
[ "${#expected}" = 64 ] || { printf '%s\n' 'Missing or ambiguous checksum.' >&2; exit 1; }
printf '%s\n' "$expected" | LC_ALL=C grep -Eq '^[A-Fa-f0-9]{64}$' || exit 1
if command -v sha256sum >/dev/null 2>&1; then actual=$(sha256sum "$temp_dir/archive.tar.gz" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then actual=$(shasum -a 256 "$temp_dir/archive.tar.gz" | awk '{print $1}')
else printf '%s\n' 'Install sha256sum or shasum.' >&2; exit 1; fi
[ "$actual" = "$expected" ] || { printf '%s\n' 'Checksum mismatch; nothing installed.' >&2; exit 1; }
[ "$(tar -tzf "$temp_dir/archive.tar.gz" | grep -cx 'ctxc')" = 1 ] || exit 1
# Extract just one member to a fresh file, never archive-provided paths.
tar -xOzf "$temp_dir/archive.tar.gz" ctxc > "$temp_dir/ctxc"
[ -s "$temp_dir/ctxc" ] || exit 1
mkdir -p "$install_dir"
stage=$(mktemp "$install_dir/.ctxc-install.XXXXXX")
cp "$temp_dir/ctxc" "$stage"
chmod 755 "$stage"
mv -f "$stage" "$install_dir/ctxc"
printf 'Installed %s to %s/ctxc\nAdd %s to PATH if needed.\n' "$version" "$install_dir" "$install_dir"
