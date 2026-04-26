# ── Ship Development ───────────────────────────────────────────────────────────

default:
    @just --list

# ── Dev ────────────────────────────────────────────────────────────────────────

# Start the shell app dev server (Vite). Daemon must be running separately.
dev:
    pnpm --filter shell dev

# Watch and rebuild Rust on changes (unstable — includes all dev tools)
watch:
    cargo watch -x "build --features unstable -p ship-cli -p mcp"

# Point the running daemon at THIS worktree's binary + .ship dir.
# Common when developing views/themes from a worktree — the installed
# ~/.local/bin/shipd serves bundled bytes, not your edits.
dev-daemon:
    @scripts/dev-daemon.sh

# Tail the daemon log (Ctrl+C to stop).
daemon-logs:
    tail -f ~/.ship/shipd.log

# ── Build ──────────────────────────────────────────────────────────────────────

# Build CLI + MCP + shipd (unstable dev build — default for development)
build:
    cargo build --features unstable -p ship-cli -p mcp -p shipd

# Build release binary (stable — no unstable features, what users get)
build-release:
    cargo build --release -p ship-cli -p mcp -p shipd

# Install dev binaries to ~/.local/bin (ship + shipd).
# Resolves the cargo target dir via `cargo metadata` so the copy works when
# .cargo/config.toml redirects target-dir (e.g. for a shared build cache
# across worktrees).
install-dev: build
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cp -f "$target/debug/ship" ~/.local/bin/ship.new && mv ~/.local/bin/ship.new ~/.local/bin/ship
    cp -f "$target/debug/shipd" ~/.local/bin/shipd
    echo "Installed: ship + shipd → ~/.local/bin"

# Install stable binary to ~/.cargo/bin
install: build-release
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cargo install --path apps/ship-cli
    cp -f "$target/release/shipd" ~/.local/bin/shipd

# Restart shipd with the latest dev build (build → install → graceful restart).
# `ship daemon stop` terminates the supervisor cleanly and shipd's own
# shutdown path removes stale daemon.pid/daemon.port before the next start.
daemon-restart: install-dev
    -ship daemon stop 2>/dev/null || true
    @sleep 1
    ship daemon start

# Watch Rust sources and auto-restart the daemon on changes.
# Templates in .ship/views/ are hot-reloaded without restart.
# Install watchexec once: cargo install watchexec-cli
daemon-watch:
    watchexec --watch apps/shipd/src --watch crates --watch apps/shipd/assets \
        --exts rs,js --on-busy-update restart -- just daemon-restart

# Rebuild WASM compiler (requires build-essential)
wasm:
    wasm-pack build crates/core/compiler --target web --out-dir ../../../packages/compiler --out-name compiler -- --features wasm

# ── Test ───────────────────────────────────────────────────────────────────────

# Run all tests (Rust + loops)
test: test-rust test-loops

# End-to-end loop tests against a running daemon (headless only).
# Each loop is a single-purpose sanity gate exercising one substrate
# concern (channel delivery, hook correlation, write-path idempotency,
# id minting, etc.). Run UI-mode loops with `just test-loops-ui`.
test-loops:
    node tests/loops/run.mjs

# All loops including Playwright-driven UI checks. Slower, separate
# CI gate — needs a daemon AND a browser.
test-loops-ui:
    node tests/loops/run.mjs --ui

# Rust workspace tests (unstable)
test-rust:
    cargo test --workspace --features unstable

# Rust tests — stable surface only (CI release gate)
test-rust-stable:
    cargo test --workspace

# Runtime crate only (fastest Rust)
test-runtime:
    cargo test -p runtime

# Compiler crate only
test-compiler:
    cargo test -p compiler

# TypeScript type check (shell app)
typecheck:
    pnpm --filter shell exec tsc --noEmit

# ── Code generation ────────────────────────────────────────────────────────────

# Regenerate Specta types (Rust → packages/ship-set/src/generated.ts)
gen-types:
    cargo run -p xtask -- gen-types

# Regenerate the Ship SDK manifest (→ packages/ship-set/sdk/*.json)
gen-sdk:
    cargo run -p xtask -- gen-sdk

# ── Lint & Format ──────────────────────────────────────────────────────────────

# Rust clippy (unstable)
lint:
    cargo clippy --workspace --features unstable -- -D warnings

# Format Rust
fmt:
    cargo fmt

# Check Rust formatting
fmt-check:
    cargo fmt --check

# Lint Ship views — no inline <script>, style budget, undefined signals.
# Exits non-zero on any error-severity finding so CI can gate on it.
lint-views:
    cargo run --features unstable -p ship-cli --quiet -- lint views --format json

# ── Docs ──────────────────────────────────────────────────────────────────────

# Build docs site
docs:
    cd apps/docs && npx tsx scripts/collect-skill-docs.ts && pnpm build

# Dev docs site
docs-dev:
    cd apps/docs && pnpm dev --port 4321 --host

# Regenerate the committed OpenAPI snapshot from the live daemon spec.
openapi-regen:
    cd apps/docs && pnpm run openapi:generate

# Fail if the committed apps/docs/openapi.json disagrees with the live spec.
# CI gate for "someone edited a utoipa annotation but forgot to regenerate".
check-openapi-drift:
    cd apps/docs && pnpm run openapi:check-drift

# Regenerate the Astro reference tree from docs/<skill>/references/docs/.
# The output (apps/docs/src/content/docs/reference/) is gitignored build
# output; predev/prebuild run this automatically.
docs-regen:
    cd apps/docs && AUDIENCE=internal pnpm run collect

# Fail if the on-disk Astro reference tree disagrees with what the collector
# would produce from source. CI gate against hand-edited generated docs.
check-docs-drift:
    cd apps/docs && pnpm run docs:check-drift

# ── Registry ───────────────────────────────────────────────────────────────────

# Seed the local registry with @unofficial packages
seed:
    @echo "Seeding local registry — requires dev server running on :3000"
    curl -s -X POST http://localhost:3000/api/registry/seed \
      -H "Content-Type: application/json" \
      -H "X-Seed-Secret: ${SEED_SECRET}" \
      -H "Cookie: ${AUTH_COOKIE}" | jq .

# Publish a package to the local registry
publish repo:
    curl -s -X POST http://localhost:3000/api/registry/publish \
      -H "Content-Type: application/json" \
      -H "Cookie: ${AUTH_COOKIE}" \
      -d '{"repo_url": "{{repo}}"}' | jq .

# ── Ship CLI ──────────────────────────────────────────────────────────────────

# Compile the active agent for the current project
compile:
    ship compile

# Activate an agent (default looks up `.ship/agents/default.jsonc`)
use agent="default":
    ship use {{agent}}

# Reconcile recorded migration checksums against current files. Non-
# destructive — touches only _sqlx_migrations, never any data table.
# Use after merging a branch that intentionally edited a baseline migration.
repair-migrations:
    ship daemon repair-migrations

# Mark a specific migration version as applied without executing it.
# For when a migration's effects already landed via a renamed file
# (running it would fail on duplicate columns / existing tables).
repair-migrations-mark version:
    ship daemon repair-migrations --mark-applied {{version}}

# ── Housekeeping ───────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Rebuild everything from scratch
rebuild: clean build wasm
    pnpm install

# Verify both stable and unstable compile (CI check)
check-gates:
    cargo build -p ship-cli -p mcp
    cargo build --features unstable -p ship-cli -p mcp
    just check-no-set-var-global-dir
    @echo "Both gates compile ✓"

# Run home-layout grep guards (warn-only in phase-1; phase-2 will flip to hard-fail)
check-raw:
    @scripts/check-raw-ship-paths.sh
    @scripts/check-raw-env-reads.sh
    @just check-migrations

# Enforce platform.db migration naming discipline (timestamp-prefixed; 0001–0006 frozen)
check-migrations:
    @scripts/check-migration-naming.sh

# Fail if any test uses set_var("SHIP_GLOBAL_DIR") — causes env var races
check-no-set-var-global-dir:
    @if grep -rn 'set_var.*SHIP_GLOBAL_DIR' apps/ crates/ --include='*.rs'; then \
        echo "ERROR: set_var(\"SHIP_GLOBAL_DIR\") found in test code."; \
        echo "Use explicit ship_dir parameters instead. See job-spec: test-db-isolation."; \
        exit 1; \
    fi
    @echo "No set_var(SHIP_GLOBAL_DIR) found ✓"
