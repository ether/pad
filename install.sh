#!/bin/sh
# Install `pad` — the nano-class terminal editor with Etherpad collab.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ether/pad/main/install.sh | sh
#
# POSIX sh — works under dash (Debian/Ubuntu default for /bin/sh), bash,
# zsh, busybox sh. We deliberately do NOT use `pipefail` since dash
# doesn't support it.
#
# Picks the fastest path that works on the system:
#   1. If `cargo` is on PATH, install via
#      `cargo install --locked --git https://github.com/ether/pad pad`.
#   2. Else, ask the user to install Rust (we don't want to silently
#      rustup-init their shell).

set -eu

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
warn() { printf '\033[33m%s\033[0m\n' "$*"; }
err()  { printf '\033[31m%s\033[0m\n' "$*" >&2; }

REPO="https://github.com/ether/pad"

if ! command -v cargo >/dev/null 2>&1; then
  err "cargo not found."
  cat >&2 <<EOF

pad is distributed via Cargo. Install Rust first, then re-run this
script:

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Or with your system package manager (apt install rustup, brew install
rustup-init, etc.) and then run \`rustup default stable\`.

EOF
  exit 1
fi

bold "Installing pad from $REPO …"
echo "  (this calls 'cargo install --locked --git' — first run takes a"
echo "  few minutes while Cargo pulls and compiles dependencies)"
echo

cargo install --locked --git "$REPO" pad

CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
if [ -x "$CARGO_BIN/pad" ]; then
  echo
  bold "Installed: $CARGO_BIN/pad"
  if [ -n "${PATH:-}" ] && ! echo ":$PATH:" | grep -q ":$CARGO_BIN:"; then
    warn "Note: $CARGO_BIN isn't on your PATH — add it to your shell rc."
  else
    echo "Try it:"
    echo "  pad ~/notes.md"
    echo "  pad https://pad-dev.etherpad.org/p/hello"
  fi
else
  warn "Build finished but pad isn't in $CARGO_BIN — check 'cargo install --root' output above."
fi
