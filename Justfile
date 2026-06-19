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
# gitignored generated assets (sdk-*.json, the FFI cdylib, the wasm pkg) must
# exist before the cargo + bun builds that consume them, or a cold clone
# fails at compile/bundle time. On dispatched worktrees these are copied from
# the launch repo by dispatch.sh; this recipe generates them.
# (lucide-icons.json is committed source — regenerate via
# `cargo run -p xtask -- gen-icons` after a lucide-static version bump.)
[group('daily')]
bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v bun  >/dev/null || { echo "bun not on PATH — install via 'curl -fsSL https://bun.sh/install | bash'" >&2; exit 1; }
    bun install
    cargo run -p xtask --quiet -- gen-types
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
    # Deck is the only bundler: bake the framework singletons + sui-components
    # umbrella + islands + marketing/arcade in-process (rolldown). No per-pack
    # pre-compile step — components compile from source on demand.
    cargo run -p deck-cli --quiet -- build-assets
    # Hub + space scaffolds: the daemon-served `/` and `/space/<id>` HTML apps.
    # The bundler reads `apps.jsonc` for the per-app list; the per-app
    # `bundle-framework` flag materializes the framework singletons so
    # each app's `.deck/` is self-contained.
    cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework
    cargo run -q -p deck-cli -- build --project apps/spaces
    just dev-runtime-install
    echo "bootstrap complete — 'just dev' to start the daemon"

# Build + stash + start the dev daemon for this worktree (headless / browser dev).
[group('daily')]
dev:
    @scripts/dev/start.sh

# The whole desktop loop in ONE command: rebuild the hub scaffold, ensure the
# dev daemon, build + launch the native shell window. Always rebuilds the
# scaffold so a stale `.deck/hub-scaffold` can't silently blank the shell.
# Pass `--app=<id>` to open a space window. Daemon-only loop: `just dev`.
[group('daily')]
shell *ARGS:
    @scripts/dev/shell.sh {{ARGS}}

# Force a one-shot rebuild of every asset bundle served by the daemon
# (sui-runtime, sui-dom, sui-components, islands, views/*).
# Regenerate the typed dynamic-route table from the view file tree (SSOT).
[group('daily')]
gen-routes:
    @cargo run -q -p deck-cli -- routes

# Bakes every bundle via the in-process Rolldown bundler (#163) — the same
# engine shipd runs, and the ONLY bundler. Framework singletons + the
# sui-components umbrella + islands + views + marketing/arcade, all traced from
# source. No per-pack build step: components compile from `.sui` on demand.
# Also rebuilds the hub + space scaffolds (the daemon-served `/` and
# `/space/<id>`) so a fresh tree or a stale `.deck/space-scaffold` doesn't
# blank a Tauri `--app=<id>` window.
[group('daily')]
dev-build-all: _sui-compiler-lib gen-routes
    @cargo run -p deck-cli --quiet -- build-assets
    @cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework
    @cargo run -q -p deck-cli -- build --project apps/spaces

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
    # ship-host (apps/host) is Ship's single native host — Tauri webview wrapping
    # a daemon-served scaffold. Needs webkit2gtk/libsoup; excluded from the
    # framework test sweep. Build it via the Tauri toolchain (release.yml /
    # `cargo build -p ship-host` on a desktop box).
    # --no-fail-fast: run EVERY test binary. cargo's default stops at the first
    # failing bin and masks the rest — the gate then lies green while later
    # suites are red. Never drop this flag.
    cargo test --workspace --exclude ship-host --features unstable --no-fail-fast

# Visual baselines — Playwright screenshots vs committed PNGs.
# Daemon must be running (`just dev`). Pass --update-snapshots to refresh
# baselines after a deliberate visual change.
[group('daily')]
visual *ARGS:
    bun run test:e2e:visual {{ARGS}}

# Library tests only — skips integration suites for fast inner-loop feedback.
[group('daily')]
test-fast:
    cargo test --workspace --exclude ship-host --lib --features unstable --no-fail-fast

# Run tests for a single crate. Usage: just test-pkg runtime
[group('daily')]
test-pkg pkg:
    cargo test -p {{ pkg }} --features unstable

# cargo check across the workspace — fastest sanity gate.
[group('daily')]
check:
    cargo check --workspace --exclude ship-host --all-targets --features unstable

# THE GATE. Every dispatched job runs this before commit; the operator skims the
# diff for taste only — the objective bar is proven here, not claimed. Composes
# the existing check/lint/drift/test surface. If this is green, the mechanical
# bar is met. See .artifacts/services/_docs-strategy.md (gate note).
[group('daily')]
gate:
    @just check
    @just lint
    @just gen-contract-check
    @just gen-events-check
    @just gen-indexes-check
    cargo test --workspace --exclude ship-host --features unstable
    @echo "✓ gate green — objective bar met (taste review still required)"

# Rust clippy + home-layout grep guards. Hard-fail on raw .ship/ joins
# (devx-06 Tier 6).
[group('daily')]
lint:
    cargo clippy --workspace --exclude ship-host --features unstable -- -D warnings
    @just check-raw

# Visual QA against the daemon serving cwd. Discovers port automatically.
[group('daily')]
qa:
    bun tests/loops/run.mjs

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

# License audit gate — walks the resolved dependency tree (node_modules/.bun)
# and fails on copyleft (GPL/AGPL/LGPL/EPL/MPL/CDDL…). Replaces hand-copied
# vendor LICENSE files now that third-party engines are declared deps inlined
# at build (see docs/ship-adrs/.../ship-component-architecture.md). Run after
# `bun install`. Pre-approved build-time-only exceptions live in the script.
[group('occasional')]
license-check:
    node scripts/check-licenses.mjs

# Enforce layer-direction rules: substrate must not import platform; etc.
[group('occasional')]
lint-layers:
    cargo run -p xtask --quiet -- lint-layers

# Static coverage report: which ops/events have passing tests? Writes to .artifacts/coverage/report.md.
[group('occasional')]
coverage:
    cargo run -p xtask --quiet -- coverage

# Enforce layer-direction rules at the use-statement level (complements lint-layers).
[group('occasional')]
lint-imports:
    cargo run -p xtask --quiet -- lint-imports

# Build docs site (Astro static).
[group('occasional')]
docs:
    cd apps/docs && bun scripts/collect-skill-docs.ts && bun run build

# Dev docs site.
[group('occasional')]
docs-dev:
    cd apps/docs && bun run dev --port 4321 --host

# Regenerate the Astro reference tree from docs/<skill>/references/docs/.
[group('occasional')]
docs-regen:
    cd apps/docs && AUDIENCE=internal bun run collect

# Fail if the on-disk Astro reference tree disagrees with the collector.
[group('occasional')]
docs-check-drift:
    cd apps/docs && bun run docs:check-drift

# The sui-compiler cdylib that the deck bundler dlopens for FFI compilation.
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
    echo "[1/8] gen-types"
    cargo run -p xtask --quiet -- gen-types
    echo "[2/8] sui-compiler cdylib (target/debug/libsui_compiler.so — FFI lib the bundler loads)"
    # Steps 3-8 drive the SUI compiler over Bun FFI against this .so. Nothing
    # out-of-band), so without this step it silently goes stale and bundles
    # compile against an old token registry / lint set. Cheap no-op when fresh.
    cargo build -p sui-compiler
    echo "[3/8] framework + sui-* bundles + islands + views + catalog (cargo run -p deck-cli --quiet -- build-assets — in-process Rolldown; the only bundler AND the only catalog producer, → .deck/catalogs/workspace.json)"
    cargo run -p deck-cli --quiet -- build-assets
    echo "[4/8] catalog baked in [3/8] — the duplicate TS producer (@ship/sui-catalog) is retired; deck-bundler is the single source"
    echo "[5/8] hub scaffold (the daemon-served / — bundled with framework assets so the dir is self-contained for installed builds)"
    cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework
    echo "[6/8] space scaffold (the daemon-served /space/<id> — the thin per-space frontend a Tauri --app=<id> window loads; DECK_APP_ID is injected by the daemon at serve time)"
    cargo run -q -p deck-cli -- build --project apps/spaces
    echo "[7/8] cargo build --release (ship-cli + shipd)"
    cargo build --release -p ship-cli -p shipd
    echo "[8/8] verify outputs"
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
    for asset in .deck/framework/sui-runtime.js .deck/sui-marketing.js .deck/sui-arcade.js .deck/catalogs/workspace.json apps/hub/.deck/index.html apps/spaces/.deck/index.html; do
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

# Build the native Node-API addon the SUI test harness compiles through.
# Unlike `sui-wasm`, this is rarely run by hand: the harness loads whatever
# `cargo build` / `just build` already produced under `target/`, so a plain
# build keeps it fresh. This recipe exists for an explicit, isolated rebuild
# (e.g. after editing the compiler crate while iterating on tests) and is what
# the drift guard in packages/sui/dom/src/test-utils/compile-sui.ts names. The
# harness dlopen's the cdylib directly (it exports the N-API init symbol), so
# there is no `.node` rename step. One cargo-current build feeds both the
# daemon and the harness — the artifact-drift fix (#255).
[group('occasional')]
sui-napi:
    cargo build -p sui-compiler-napi

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

# Regenerate packages/deck/sdk/src/generated/collections.ts — the typed
# Database interface derived from .ship/collections/*.jsonc (typed Kysely input).
[group('occasional')]
gen-collection-types:
    cargo run -p xtask -- gen-collection-types

# Verify generated collection types are up-to-date (CI gate — exits 1 on drift).
[group('occasional')]
gen-collection-types-check:
    cargo run -p xtask -- gen-collection-types --check

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
    @scripts/lint/check-sui-runtime-no-ship-api.sh
    @scripts/lint/check-deck-package-names.sh
    @just lint-justfile
    @just lint-vocab
    @just check-migrations

# Syntax-check every bash-shebang recipe body with `bash -n`. `just` treats
# recipe bodies as opaque text, so a bash typo (e.g. a dropped `if … then`)
# parses fine at the just layer yet breaks `just build` at runtime. This catches
# that class statically. Recipes taking interpolated args ({{…}}) are skipped —
# their bodies aren't pure bash until just substitutes.
[group('occasional')]
lint-justfile:
    #!/usr/bin/env bash
    set -euo pipefail
    fail=0
    for r in $(just --summary); do
        body=$(just --show "$r" 2>/dev/null | sed -n 's/^    //p')
        printf '%s\n' "$body" | grep -q '^#!/usr/bin/env bash' || continue
        printf '%s\n' "$body" | grep -q '[{][{]' && continue
        if ! printf '%s\n' "$body" | bash -n 2>/dev/null; then
            echo "  ✗ recipe '$r' has a bash syntax error" >&2
            fail=1
        fi
    done
    [ "$fail" = "0" ] || { echo "lint-justfile: recipe shell syntax error(s)" >&2; exit 1; }
    echo "lint-justfile: all bash recipe bodies parse"

# Banned-vocabulary gate. Fails when a banned term (deleted concept) appears
# in hand-authored source/proto/tests. Ban-list: .ship/lint/banned-vocab.txt
# — a refactor step appends a term only once its last real use is deleted.
# Empty list ⟹ no-op.
[group('occasional')]
lint-vocab:
    @scripts/lint/check-banned-vocab.sh

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
