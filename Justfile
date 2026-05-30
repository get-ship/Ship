# ── Ship Development ─────────────────────────────────────────────────────────
#
# Daily contributor flow runs through the recipes below. Raw cargo / bun /
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

# One-shot fresh-clone setup. Order matters and is dependency-driven: the
# gitignored generated assets (lucide registry, sdk-*.json, the FFI cdylib,
# the wasm pkg) must exist before the cargo + bun builds that consume them,
# or a cold clone fails at compile/bundle time. On dispatched worktrees these
# are copied from the launch repo by dispatch.sh; this recipe generates them.
[group('daily')]
bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v bun  >/dev/null || { echo "bun not on PATH — install via 'curl -fsSL https://bun.sh/install | bash'" >&2; exit 1; }
    bun install
    cargo run -p xtask --quiet -- gen-types
    # lucide-icons.json — deck-server's build.rs panics without it, so it must
    # land before any cargo build that compiles deck-server (incl. gen-sdk).
    bun scripts/codegen/generate-lucide-registry.mjs
    # sdk-*.json bootstrap cycle: shipd include_str!s these, and gen-sdk runs
    # *through* shipd (via xtask). Seed empty placeholders so shipd compiles,
    # then gen-sdk overwrites them with real content.
    mkdir -p packages/deck/sdk/src/generated
    for f in sdk-urls sdk-view-contexts sdk-views sdk-collections sdk-triggers; do
        [ -s "packages/deck/sdk/src/generated/$f.json" ] || echo '{}' > "packages/deck/sdk/src/generated/$f.json"
    done
    cargo run -p xtask --quiet -- gen-sdk
    # sui-compiler cdylib — the .so the bun asset builds load over FFI.
    cargo build -p sui-compiler
    # sui-compiler-wasm pkg — the in-browser compiler the sui-code-editor
    # component bundles against.
    bun run wasm
    cargo build -p ship-cli
    bun run --filter @ship/primitives build
    bun run --filter @ship/charts build
    bun run --filter @ship/ui-components build
    bun run --filter @ship/sui-catalog build
    bun packages/deck/catalog/src/cli.ts
    just dev-runtime-install
    echo "bootstrap complete — 'just dev' to start the daemon"

# Build + stash + start the dev daemon for this worktree.
[group('daily')]
dev:
    @scripts/dev/start.sh

# Frontend dev server for shell (vite). Proxies /api, /_sui, /sui, /view-host
# to the daemon — `just dev` must already be running. vite.config.ts writes
# its pid to <daemon_dir>/vite.pid so `just dev-stop` can reap it.
[group('daily')]
shell-dev:
    @bun run --filter shell dev

# Force a one-shot rebuild of every asset bundle served by the daemon
# (sui-runtime, sui-dom, sui-components, islands, ship-elements, views/*).
# Regenerate the typed dynamic-route table from the view file tree (SSOT).
[group('daily')]
gen-routes:
    @node scripts/gen-routes.mjs

# Build the standalone SUI component packs (primitives/charts/pipelines/
# ui-components) → each package's `dist/`, served by the daemon at
# `/_sui/packs/<id>/`. These are gitignored build outputs that the deck
# bundler does NOT produce, so without this they go stale (and a stale pack
# that inlines its own @ship/sui-runtime causes split-brain signals → blank
# shell). Part of dev-build-all so the JS build is actually complete.
build-packs: _sui-compiler-lib
    @bun run --filter './packages/components/*' build

# Runs `packages/deck/bundler/index.ts --all` against the installed bun toolchain.
[group('daily')]
dev-build-all: _sui-compiler-lib gen-routes build-packs
    @bun packages/deck/bundler/index.ts --all

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
    # Activate the tracked git hooks (.artifacts boundary guard). Local config,
    # so a fresh clone needs this once — hence here.
    git config core.hooksPath .githooks
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
    bun run test:e2e:visual {{ARGS}}

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

# Enforce layer-direction rules: substrate must not import platform; etc.
[group('occasional')]
lint-layers:
    cargo run -p xtask --quiet -- lint-layers

# Enforce layer-direction rules at the use-statement level (complements lint-layers).
[group('occasional')]
lint-imports:
    cargo run -p xtask --quiet -- lint-imports

# Regenerate the aggregated SUI Custom Elements Manifest from per-component
# CEMs under packages/components/ui-components/src/. Writes
# crates/deck/deck-core/assets/sui-custom-elements.json which apps/shipd embeds.
[group('occasional')]
cem:
    bun run --filter @ship/ui-components build

# Build docs site (Astro static).
[group('occasional')]
docs:
    cd apps/docs && bun scripts/collect-skill-docs.ts && bun run build

# Dev docs site.
[group('occasional')]
docs-dev:
    cd apps/docs && bun run dev --port 4321 --host

# Regenerate the committed OpenAPI snapshot.
[group('occasional')]
openapi-regen:
    cd apps/docs && bun run openapi:generate

# Fail if the committed openapi.json disagrees with the live spec.
[group('occasional')]
openapi-check:
    cd apps/docs && bun run openapi:check-drift

# Regenerate the Astro reference tree from docs/<skill>/references/docs/.
[group('occasional')]
docs-regen:
    cd apps/docs && AUDIENCE=internal bun run collect

# Fail if the on-disk Astro reference tree disagrees with the collector.
[group('occasional')]
docs-check-drift:
    cd apps/docs && bun run docs:check-drift

# The sui-compiler cdylib that sui-build/the bundler dlopen for FFI compilation.
# A real build-graph dependency, not a remembered step — so a compiler/lint
# source change always reaches every consumer (build, dev-build-all). cargo
# no-ops in ~2s when fresh, so it's effectively free.
_sui-compiler-lib:
    cargo build -p sui-compiler

# One-shot production build: gen-types, every pack + ui-components rebuild,
# catalog regeneration, cargo --release, then verify outputs. Equivalent to
# `ship build`.
[group('daily')]
build: _sui-compiler-lib
    #!/usr/bin/env bash
    set -euo pipefail
    command -v bun >/dev/null || { echo "build: bun not on PATH" >&2; exit 1; }
    echo "[1/9] gen-types"
    cargo run -p xtask --quiet -- gen-types
    echo "[2/9] sui-compiler cdylib (target/debug/libsui_compiler.so — FFI lib the bun builds load)"
    # Steps 3-6 drive the SUI compiler over Bun FFI against this .so. Nothing
    # in cargo's graph forces it to rebuild (the JS bundler consumes it
    # out-of-band), so without this step it silently goes stale and bundles
    # compile against an old token registry / lint set. Cheap no-op when fresh.
    cargo build -p sui-compiler
    echo "[3/9] primitives build (@ship/primitives → packages/components/primitives/dist/)"
    bun run --filter @ship/primitives build
    echo "[4/9] charts build (@ship/charts → packages/components/charts/dist/)"
    bun run --filter @ship/charts build
    echo "[5/9] ui-components build (sui-build → crates/deck/deck-core/assets/)"
    bun run --filter @ship/ui-components build
    echo "[6/9] view bundles + sui-* bundles (packages/deck/bundler/index.ts --all)"
    bun packages/deck/bundler/index.ts --all
    echo "[7/9] sui-catalog walk (→ .deck/catalogs/{workspace,published}.json)"
    bun run --filter @ship/sui-catalog build
    bun packages/deck/catalog/src/cli.ts
    echo "[8/9] cargo build --release (ship-cli + shipd)"
    cargo build --release -p ship-cli -p shipd
    echo "[9/9] verify outputs"
    target_dir=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    missing=0
    for bin in ship shipd; do
        if [ ! -x "$target_dir/release/$bin" ]; then
            echo "  ✗ $target_dir/release/$bin missing" >&2
            missing=1
        else
            echo "  ✓ $target_dir/release/$bin"
        fi
    done
    for asset in crates/deck/deck-core/assets/sui-runtime.js crates/deck/deck-core/assets/sui-components.js packages/components/primitives/dist/sui-primitives.js packages/components/charts/dist/sui-charts.js .deck/catalogs/workspace.json .deck/catalogs/published.json; do
        if [ ! -s "$asset" ]; then
            echo "  ✗ $asset missing or empty" >&2
            missing=1
        else
            echo "  ✓ $asset"
        fi
    done
    [ "$missing" = "0" ] || { echo "build: outputs incomplete" >&2; exit 1; }
    echo "build complete — release binaries at $target_dir/release/"

# Build CLI + shipd (unstable dev build).
[group('occasional')]
build-dev:
    cargo build --features unstable -p ship-cli -p shipd -p sui-lsp

# Cargo-only release build (no front-end). Use `just build` for the full
# production artifact set; this recipe is the cargo subset.
[group('occasional')]
build-release:
    cargo build --release -p ship-cli -p shipd

# Install dev binaries to ~/.local/bin (ship + shipd).
[group('occasional')]
install-dev: build-dev
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cp -f "$target/debug/ship" ~/.local/bin/ship.new && mv ~/.local/bin/ship.new ~/.local/bin/ship
    cp -f "$target/debug/shipd" ~/.local/bin/shipd
    cp -f "$target/debug/sui-lsp" ~/.local/bin/sui-lsp
    echo "Installed: ship + shipd + sui-lsp → ~/.local/bin"

# Install stable binary to ~/.cargo/bin.
[group('occasional')]
install: build-release
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cargo install --path apps/ship-cli
    cp -f "$target/release/shipd" ~/.local/bin/shipd

# Rebuild the in-browser SUI template compiler. wasm-pack writes into
# `packages/sui/compiler-wasm/pkg/` (gitignored) so the package's own
# package.json + .gitignore stay intact; the workspace re-exports the
# generated entry via `main` / `exports`. Required before
# `bun run --filter @ship/docs build` — the docs playground imports
# `@ship/sui-compiler-wasm/sui_compiler_bg.wasm?url` to compile `.sui`
# edits without a server round-trip. ~640KB wasm + ~9KB JS shim.
[group('occasional')]
sui-wasm:
    wasm-pack build crates/sui/compiler --target web --out-dir ../../../packages/sui/compiler-wasm/pkg --out-name sui_compiler -- --features wasm

# Regenerate Specta types (Rust → generated TS bindings).
[group('occasional')]
gen-types:
    cargo run -p xtask -- gen-types

# Regenerate the Ship SDK manifest.
[group('occasional')]
gen-sdk:
    cargo run -p xtask -- gen-sdk

# Generate per-domain consumer contract guides under docs/contract/.
[group('occasional')]
gen-contract:
    cargo run -p xtask -- gen-contract

# Verify generated contract docs are up-to-date (CI gate — exits 1 on drift).
[group('occasional')]
gen-contract-check:
    cargo run -p xtask -- gen-contract --check

# Regenerate audit coverage README from per-module audit docs.
[group('occasional')]
gen-audit-readme:
    cargo run -p xtask -- gen-audit-readme

# Regenerate tier-filtered event proto + TS from the event registry.
[group('occasional')]
gen-events:
    cargo run -p xtask -- gen-events

# Verify generated event artifacts are up-to-date (CI gate — exits 1 on drift).
[group('occasional')]
gen-events-check:
    cargo run -p xtask -- gen-events --check

# Regenerate emit-sites.md + substrate-fns.md citation indexes.
[group('occasional')]
gen-indexes:
    cargo run -p xtask -- gen-indexes

# Verify citation indexes are up-to-date (CI gate — exits 1 on drift).
[group('occasional')]
gen-indexes-check:
    cargo run -p xtask -- gen-indexes --check

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
    cargo build -p ship-cli -p shipd
    cargo build --features unstable -p ship-cli -p shipd
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

# Enforce platform.db migration naming + authoring discipline.
[group('occasional')]
check-migrations:
    @scripts/lint/check-migration-naming.sh
    @scripts/lint/check-migration-discipline.sh

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
    bun run --filter shell typecheck
