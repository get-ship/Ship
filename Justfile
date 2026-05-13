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

# One-shot fresh-clone setup. Order matters: gen-types feeds the workspace,
# ship-set icons feed @ship/set build, both pnpm builds populate the asset
# trees that shipd's include_str! consumes, and dev-runtime-install copies
# bun + sui-compiler + bundler.ts into ~/.ship/runtime/.
[group('daily')]
bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v bun  >/dev/null || { echo "bun not on PATH — install via 'curl -fsSL https://bun.sh/install | bash'" >&2; exit 1; }
    command -v pnpm >/dev/null || { echo "pnpm not on PATH" >&2; exit 1; }
    cargo run -p xtask --quiet -- gen-types
    cargo build -p ship-cli
    pnpm install
    pnpm -F @ship/set icons
    pnpm -F @ship/set build
    pnpm -F @ship/ui-components build
    just dev-runtime-install
    echo "bootstrap complete — 'just dev' to start the daemon"

# Build + stash + start the dev daemon for this worktree.
[group('daily')]
dev:
    @scripts/dev/start.sh

# Frontend dev server for shell2 (vite). Proxies /api, /_sui, /sui, /view-host
# to the daemon — `just dev` must already be running.
[group('daily')]
shell2-dev:
    @scripts/dev/shell2-start.sh

# Force a one-shot rebuild of every asset bundle served by the daemon
# (sui-runtime, sui-dom, sui-components, islands, ship-elements, views/*).
# Runs `runtime-assets/bundler.ts --all` against the installed bun toolchain.
[group('daily')]
dev-build-all:
    @bun runtime-assets/bundler.ts --all

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

# Populate ~/.ship/runtime/ from local sources for dev: bun from $BUN_PATH
# (defaulting to a `bun` on PATH), sui-compiler from target/release/.
# Idempotent. Run once after a fresh clone.
[group('daily')]
dev-runtime-install:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p sui-compiler --release
    cargo build -p ship-cli
    BUN_BIN="${BUN_PATH:-$(command -v bun || true)}"
    if [ -z "$BUN_BIN" ] || [ ! -x "$BUN_BIN" ]; then
        echo "no bun on PATH; install via \`curl -fsSL https://bun.sh/install | bash\` or set BUN_PATH" >&2
        exit 1
    fi
    ./target/debug/ship runtime install --bun "$BUN_BIN" --sui-compiler target/release/sui-compiler

# Same as dev-runtime-install but wipes ~/.ship/runtime/ first.
[group('daily')]
dev-runtime-update:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p sui-compiler --release
    cargo build -p ship-cli
    BUN_BIN="${BUN_PATH:-$(command -v bun || true)}"
    if [ -z "$BUN_BIN" ] || [ ! -x "$BUN_BIN" ]; then
        echo "no bun on PATH; install via \`curl -fsSL https://bun.sh/install | bash\` or set BUN_PATH" >&2
        exit 1
    fi
    ./target/debug/ship runtime update --bun "$BUN_BIN" --sui-compiler target/release/sui-compiler

# Run all tests (Rust + loops). For a faster inner loop use test-fast.
[group('daily')]
test:
    cargo test --workspace --features unstable

# Visual baselines — Playwright screenshots vs committed PNGs.
# Daemon must be running (`just dev`). Pass --update-snapshots to refresh
# baselines after a deliberate visual change.
[group('daily')]
visual *ARGS:
    pnpm test:e2e:visual {{ARGS}}

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

# Full smoke: build everything, restart the daemon, hit every baselined
# view, run the Playwright visual suite. What CI will run — aim for green
# before opening a PR.
[group('daily')]
smoke-deploy:
    @scripts/dev/smoke-deploy.sh

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

# Capture a fresh perf baseline. Rebuilds the SUI bench bundle, runs the
# js-framework-benchmark suite against SUI + comparison frameworks, and
# writes tests/perf/baselines/<date>-<label>.json. Commit the result.
# `label` is required — pass e.g. `just bench baseline` or `just bench post-d5`
# (passes through to `--label`). No implicit overwrite of the date's anchor.
[group('occasional')]
bench label:
    @scripts/dev/bench-run.mjs --label {{label}}

# Compare current SUI perf vs the latest committed baseline. Exits non-zero
# if any op regresses by more than 15% (override with
# BENCH_REGRESSION_THRESHOLD). Manual gate before merging large runtime
# changes.
[group('occasional')]
bench-check:
    @scripts/dev/bench-check.mjs

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

# Regenerate the aggregated SUI Custom Elements Manifest from per-component
# CEMs under packages/ui-components/src/. Writes
# crates/ship-sui/assets/sui-custom-elements.json which apps/shipd embeds.
[group('occasional')]
cem:
    pnpm -F @ship/ui-components build

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

# Populate the local daemon with one agent + skill + collection + artifact
# so first-run dev doesn't look like a dead app. Targets the daemon serving
# the current worktree (discovers port + token from its daemon_dir).
# Requires `just dev` to be running.
[group('daily')]
seed:
    @scripts/dev/seed.sh

# Seed the local *registry* (not the daemon) with @unofficial packages.
# Separate from `just seed`; targets registry-site dev server on :3000.
[group('occasional')]
seed-registry:
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
