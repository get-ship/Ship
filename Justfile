# ── Ship Development ─────────────────────────────────────────────────────────
#
# Daily contributor flow runs through the recipes below. Raw cargo / pnpm /
# bash invocations stay possible as escape hatches but are not recommended:
# if you reach for one, ask whether a recipe should exist instead.
#
# Per devx-04, `just dev` and friends operate on a per-worktree daemon at
# ~/.ship/machine/daemons/<hash>/, with stashed binaries that survive
# subsequent rebuilds until you run `just dev-restart`.

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

# ── Daily ────────────────────────────────────────────────────────────────────

# Build + stash + start the dev daemon for this worktree.
[group('daily')]
dev:
    @scripts/dev/start.sh

# Stop, rebuild, re-stash, start. Use after Rust changes.
[group('daily')]
dev-restart:
    @scripts/dev/restart.sh

# Stop the daemon serving this worktree.
[group('daily')]
dev-stop:
    @scripts/dev/stop.sh

# List every daemon: workspace, port, pid, sha, alignment to current build.
[group('daily')]
status:
    @scripts/dev/status.sh

# `ship doctor`: build matrix, layout version, common-issue checklist.
[group('daily')]
doctor:
    ship doctor

# Run all tests (Rust + loops). For a faster inner loop use test-fast.
[group('daily')]
test:
    cargo test --workspace --features unstable

# Library tests only — skips integration suites for fast inner-loop feedback.
[group('daily')]
test-fast:
    cargo test --workspace --lib --features unstable

# Run tests for a single crate. Usage: just test-pkg runtime
[group('daily')]
test-pkg pkg:
    cargo test -p {{ pkg }} --features unstable

# cargo check across the workspace — fastest sanity gate.
[group('daily')]
check:
    cargo check --workspace --all-targets --features unstable

# Rust clippy + home-layout grep guards. Hard-fail on raw .ship/ joins
# (devx-06 Tier 6).
[group('daily')]
lint:
    cargo clippy --workspace --features unstable -- -D warnings
    @just check-raw

# Visual QA against the daemon serving cwd. Discovers port automatically.
[group('daily')]
qa:
    node tests/loops/run.mjs

# ── Occasional ───────────────────────────────────────────────────────────────

# Set up a worktree for a branch. Drops .ship/.boundary so views/pipelines
# stay isolated from the main checkout. First-run prompts for worktrees.dir.
[group('occasional')]
init-worktree branch:
    #!/usr/bin/env bash
    set -euo pipefail
    base=$(ship config get worktrees.dir 2>/dev/null \
           || echo "$HOME/dev/$(basename "$PWD")-worktrees")
    target="$base/$(echo "{{ branch }}" | tr / -)"
    git worktree add "$target" -b "{{ branch }}" 2>/dev/null \
        || git worktree add "$target" "{{ branch }}"
    mkdir -p "$target/.ship"
    [ -f "$target/.ship/.boundary" ] || echo '{}' > "$target/.ship/.boundary"
    echo "worktree: $target"

# Remove stash binaries for daemons that aren't currently running.
# Leaves library/ alone — that's portable user state.
[group('occasional')]
clean:
    @scripts/dev/clean.sh

# Cargo target dir + node_modules. Asks for confirmation.
[group('occasional')]
clean-deep:
    cargo clean
    rm -rf node_modules apps/*/node_modules packages/*/node_modules

# Format Rust.
[group('occasional')]
fmt:
    cargo fmt

# Check Rust formatting.
[group('occasional')]
fmt-check:
    cargo fmt --check

# Lint Ship views (no inline <script>, style budget, undefined signals).
[group('occasional')]
lint-views:
    cargo run --features unstable -p ship-cli --quiet -- lint views --format json

# Build docs site (Astro static).
[group('occasional')]
docs:
    cd apps/docs && npx tsx scripts/collect-skill-docs.ts && pnpm build

# Dev docs site.
[group('occasional')]
docs-dev:
    cd apps/docs && pnpm dev --port 4321 --host

# Regenerate the committed OpenAPI snapshot.
[group('occasional')]
openapi-regen:
    cd apps/docs && pnpm run openapi:generate

# Fail if the committed openapi.json disagrees with the live spec.
[group('occasional')]
openapi-check:
    cd apps/docs && pnpm run openapi:check-drift

# Regenerate the Astro reference tree from docs/<skill>/references/docs/.
[group('occasional')]
docs-regen:
    cd apps/docs && AUDIENCE=internal pnpm run collect

# Fail if the on-disk Astro reference tree disagrees with the collector.
[group('occasional')]
docs-check-drift:
    cd apps/docs && pnpm run docs:check-drift

# Frontend dev server (Vite). Daemon must be running separately.
[group('occasional')]
shell-dev:
    pnpm --filter shell dev

# Build CLI + MCP + shipd (unstable dev build).
[group('occasional')]
build:
    cargo build --features unstable -p ship-cli -p mcp -p shipd

# Build release binary (stable, no unstable features — what users get).
[group('occasional')]
build-release:
    cargo build --release -p ship-cli -p mcp -p shipd

# Install dev binaries to ~/.local/bin (ship + shipd).
[group('occasional')]
install-dev: build
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cp -f "$target/debug/ship" ~/.local/bin/ship.new && mv ~/.local/bin/ship.new ~/.local/bin/ship
    cp -f "$target/debug/shipd" ~/.local/bin/shipd
    echo "Installed: ship + shipd → ~/.local/bin"

# Install stable binary to ~/.cargo/bin.
[group('occasional')]
install: build-release
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cargo install --path apps/ship-cli
    cp -f "$target/release/shipd" ~/.local/bin/shipd

# Rebuild WASM compiler (requires build-essential).
[group('occasional')]
wasm:
    wasm-pack build crates/core/compiler --target web --out-dir ../../../packages/compiler --out-name compiler -- --features wasm

# Regenerate Specta types (Rust → packages/ship-set/src/generated.ts).
[group('occasional')]
gen-types:
    cargo run -p xtask -- gen-types

# Regenerate the Ship SDK manifest (→ packages/ship-set/sdk/*.json).
[group('occasional')]
gen-sdk:
    cargo run -p xtask -- gen-sdk

# Compile the active agent for the current project.
[group('occasional')]
compile:
    ship compile

# Activate an agent (default looks up `.ship/agents/default.jsonc`).
[group('occasional')]
use agent="default":
    ship use {{ agent }}

# Reconcile recorded migration checksums against current files.
[group('occasional')]
repair-migrations:
    ship daemon repair-migrations

# Mark a migration version as applied without executing it.
[group('occasional')]
repair-migrations-mark version:
    ship daemon repair-migrations --mark-applied {{ version }}

# Tail the daemon log. Discovers the right log file via daemon dir.
[group('occasional')]
logs:
    #!/usr/bin/env bash
    set -euo pipefail
    log="${SHIP_HOME:-$HOME/.ship}/machine/logs/shipd.log"
    if [ ! -f "$log" ]; then
        echo "no log at $log — has the daemon ever started?" >&2
        exit 1
    fi
    tail -f "$log"

# Verify both stable and unstable compile (CI gate).
[group('occasional')]
check-gates:
    cargo build -p ship-cli -p mcp
    cargo build --features unstable -p ship-cli -p mcp
    just check-no-set-var-global-dir
    @echo "Both gates compile ✓"

# Home-layout grep guards. Hard-fail per devx-06 Tier 6: any raw
# `.join(".ship")` outside crates/core/runtime/src/home/ blocks the build.
# Leaf-crate portability check runs as part of the same target — if a
# leaf crate accumulates a non-portable dep, modular-runtime-01's
# acceptance criterion fails here.
[group('occasional')]
check-raw:
    @scripts/lint/check-raw-ship-paths.sh
    @scripts/lint/check-raw-env-reads.sh
    @scripts/lint/check-leaf-portability.sh
    @scripts/lint/check-runtime-access.sh
    @scripts/lint/check-module-dag.sh
    @just check-migrations

# Enforce platform.db migration naming discipline.
[group('occasional')]
check-migrations:
    @scripts/lint/check-migration-naming.sh

# Fail if any test uses set_var("SHIP_GLOBAL_DIR") — causes env var races.
[group('occasional')]
check-no-set-var-global-dir:
    @if grep -rn 'set_var.*SHIP_GLOBAL_DIR' apps/ crates/ --include='*.rs'; then \
        echo "ERROR: set_var(\"SHIP_GLOBAL_DIR\") found in test code."; \
        echo "Use explicit ship_dir parameters instead."; \
        exit 1; \
    fi
    @echo "No set_var(SHIP_GLOBAL_DIR) found ✓"

# Seed the local registry with @unofficial packages.
[group('occasional')]
seed:
    @echo "Seeding local registry — requires dev server running on :3000"
    curl -s -X POST http://localhost:3000/api/registry/seed \
      -H "Content-Type: application/json" \
      -H "X-Seed-Secret: ${SEED_SECRET}" \
      -H "Cookie: ${AUTH_COOKIE}" | jq .

# Publish a package to the local registry.
[group('occasional')]
publish repo:
    curl -s -X POST http://localhost:3000/api/registry/publish \
      -H "Content-Type: application/json" \
      -H "Cookie: ${AUTH_COOKIE}" \
      -d '{"repo_url": "{{ repo }}"}' | jq .

# TypeScript type check (shell app).
[group('occasional')]
typecheck:
    pnpm --filter shell exec tsc --noEmit
