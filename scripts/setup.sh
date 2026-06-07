#!/usr/bin/env bash
# Ship dev setup — the single host bootstrap. Run once on a fresh machine.
# Idempotent: safe to re-run.
#
#   bash scripts/setup.sh
#
# Installs, in order:
#   1. System build deps      (scripts/system-deps.txt — shared with Containerfile)
#   2. Rust                   (pinned by rust-toolchain.toml; + rustfmt/clippy/wasm32)
#   3. bun                    (the ONE JS runtime for all of Ship)
#   4. wasm-pack              (for `bun run wasm` in bootstrap)
#   5. node                   (OPTIONAL — Playwright e2e/visual only; SHIP_WITH_E2E=1)
#   6. just bootstrap         (the canonical post-toolchain asset build)
#   7. ship CLI on PATH       (convenience)
#
# Works on Linux (apt) and macOS (brew, best-effort). protoc is NOT needed —
# it is vendored via protoc-bin-vendored.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
say() { printf '\n==> %s\n' "$*"; }

# ── 1. System build deps ──────────────────────────────────────────────────────
PKGS="$(grep -vE '^[[:space:]]*#|^[[:space:]]*$' scripts/system-deps.txt | tr '\n' ' ')"
if command -v apt-get >/dev/null 2>&1; then
  say "Installing system deps (apt): ${PKGS}"
  sudo apt-get update -y
  sudo apt-get install -y ${PKGS}
elif command -v brew >/dev/null 2>&1; then
  say "Installing system deps (brew, best-effort)"
  xcode-select -p >/dev/null 2>&1 || xcode-select --install || true
  brew install cmake pkg-config openssl@3 git curl || true
else
  echo "!! No apt-get or brew. Install manually: ${PKGS}" >&2
fi

# ── 2. Rust (pinned by rust-toolchain.toml) ───────────────────────────────────
if ! command -v rustup >/dev/null 2>&1; then
  say "Installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain none
  # shellcheck disable=SC1091
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"
CHANNEL="$(sed -nE 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' rust-toolchain.toml)"
say "Installing Rust ${CHANNEL} (+ rustfmt, clippy, wasm32-unknown-unknown)"
rustup toolchain install "${CHANNEL}" --profile minimal -c rustfmt -c clippy -t wasm32-unknown-unknown
echo "    rust: $(rustc --version 2>/dev/null || echo 'resolves on first cargo run in-repo')"

# ── 3. bun (the one JS runtime) ───────────────────────────────────────────────
if ! command -v bun >/dev/null 2>&1; then
  say "Installing bun"
  curl -fsSL https://bun.sh/install | bash
fi
export PATH="$HOME/.bun/bin:$PATH"
echo "    bun: $(bun --version 2>/dev/null || echo 'restart shell to pick up PATH')"

# ── 4. wasm-pack ──────────────────────────────────────────────────────────────
if ! command -v wasm-pack >/dev/null 2>&1; then
  say "Installing wasm-pack"
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -f
fi

# ── 5. Node — OPTIONAL, Playwright e2e/visual only ────────────────────────────
# Bun runs everything Ship owns. @playwright/test forks node worker processes
# internally, so the e2e/visual suites (`just visual`, `just smoke-deploy`) are
# the SOLE node holdout. Opt in with SHIP_WITH_E2E=1; otherwise we stay bun-only.
if [ "${SHIP_WITH_E2E:-0}" = "1" ]; then
  if ! command -v node >/dev/null 2>&1; then
    say "Installing node (Playwright e2e workers)"
    command -v apt-get >/dev/null 2>&1 && sudo apt-get install -y nodejs npm \
      || command -v brew >/dev/null 2>&1 && brew install node \
      || echo "!! install node manually for Playwright e2e" >&2
  fi
else
  echo "    node: skipped (bun-only). SHIP_WITH_E2E=1 to install it for Playwright suites."
fi

# ── 6. Project assets (canonical post-toolchain step) ─────────────────────────
say "Building project assets (just bootstrap)"
just bootstrap

# ── 7. ship CLI on PATH ───────────────────────────────────────────────────────
say "Installing the ship CLI"
cargo install --path apps/ship-cli --locked --force
echo "    ship: $(ship --version 2>/dev/null || echo 'ensure ~/.cargo/bin is on PATH')"

cat <<'EOF'

✓ Setup complete.

  PATH (add to your shell rc if missing):
    export PATH="$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.local/bin:$PATH"

  Next:
    just dev          # build + start the dev daemon for this worktree
    just init-worktree <branch>   # spin up an isolated worktree

  Backup reminder: verify `ssh -T git@github.com` works, then push your work —
  a fresh clone is your off-machine safety net.
EOF
