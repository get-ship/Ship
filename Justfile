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

# Path resolution for cross-repo dev layouts. The ship repo consumes
# crates from the sui repo (deck, sui-compiler) and the proto/ tree
# from the ship-contracts repo. Both are pinned to the right path via
# these helpers. Set SUI_PATH / SHIP_CONTRACTS_PATH env vars to
# override; otherwise the recipes assume the canonical sibling layout
# (sui/, ship-contracts/ as siblings of ship/).
#
# The repo no longer has a hardcoded /home/madvib/dev/Ship/sui default;
# these are env-var-only. The operator must set SUI_PATH for any
# recipe that needs the sui binary (deck-cli, sui-compiler, sui-lsp).
# Recipes that need sui will fail loudly with 'sui_path() returned
# empty' if SUI_PATH is unset.
sui_path := env_var_or_default('SUI_PATH', '')
ship_contracts_path := env_var_or_default('SHIP_CONTRACTS_PATH', '')

default:
    @just --list

# ── Daily ────────────────────────────────────────────────────────────────────


# Build + stash + start the dev daemon for this worktree (headless / browser dev).
[group('daily')]
dev:
    @scripts/dev/start.sh



# Bakes every bundle via the in-process Rolldown bundler (#163) — the same
# engine shipd runs, and the ONLY bundler. Framework singletons + the
# sui-components umbrella + islands + views + marketing/arcade, all traced from
# source. No per-pack build step: components compile from `.sui` on demand.
# Also rebuilds every app's scaffold (the daemon-served `/` and `/space/<id>`)
# so a fresh tree or a stale `.deck/` doesn't blank a Tauri `--app=<id>` window.
#
# Route tables self-generate inside the build — no separate `gen-routes` step
# and no hardcoded app list. `build-assets` writes the view tables
# (`views/{routes,meta}.gen.ts`) from the framework's view scan; each
# `deck build --project <app>` writes that app's `routes.gen.ts` (the bundle
# entry) as its first step. The app list comes from `apps/shipd/apps.jsonc` —
# the single registry (also read by shipd at startup), never duplicated here.
[group('daily')]
dev-build-all: _sui-compiler-lib _deck-cli
    @cargo run --manifest-path {{sui_path}}/Cargo.toml -p deck-cli --quiet -- build-assets
    @just _build-apps

# Build every app in the registry from its `root` via `deck build --project`.
# The app list lives ONLY in `apps/shipd/apps.jsonc` (the single registry, also
# read by shipd at startup) — no recipe duplicates it. The root-mounted app
# (`route == "/"`) self-contains the /_deck/* framework surface
# (`--bundle-framework`); every `deck build` self-generates that app's
# `routes.gen.ts` entry as its first step. jq can't read JSONC, so strip `//`
# line comments first (apps.jsonc holds no `//` inside any string). deck-cli
# lives in the external sui repo post-carve, hence the explicit
# --manifest-path. cwd stays in the ship worktree (deck-cli walks the
# app's own root for entry discovery). `DECK_FRAMEWORK_ASSETS_DIR`
# points the bundler at sui's `crates/deck/deck-core/assets/` (where
# the staged framework bundles live post-carve) so cross-repo invocations
# find the assets their `bundle_framework: true` step needs. The
# path is TEMP — gets relativized to `../sui/...` in the layout-reshuffle
# capstone.
_build-apps:
    #!/usr/bin/env bash
    set -euo pipefail
    sed 's://.*$::' apps/shipd/apps.jsonc \
        | jq -r '.apps[] | [.root, .route] | @tsv' \
        | while IFS=$'\t' read -r root route; do
            [ -z "$root" ] && continue
            abs_root="$(realpath -m "$root")"
            if [ "$route" = "/" ]; then
                DECK_FRAMEWORK_ASSETS_DIR={{sui_path}}/crates/deck/deck-core/assets \
                    cargo run -q --manifest-path {{sui_path}}/Cargo.toml -p deck-cli -- build --project "$abs_root" --bundle-framework
            else
                DECK_FRAMEWORK_ASSETS_DIR={{sui_path}}/crates/deck/deck-core/assets \
                    cargo run -q --manifest-path {{sui_path}}/Cargo.toml -p deck-cli -- build --project "$abs_root"
            fi
        done

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

# Run all tests (Rust). For a faster inner loop use test-fast.
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
    @just doc-check
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
    @just lint-anti-theater

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

# bench + bench-check — moved to the sui repo's Justfile. tests/perf/ +
# the keyed/sui bench app live there now (sui owns the framework; the
# perf harness exercises sui against comparison frameworks). Run from
# the sui repo: `just bench <label>` and `just bench-check`.

# Format Rust.
[group('occasional')]
fmt:
    cargo fmt

# Check Rust formatting.
[group('occasional')]
fmt-check:
    cargo fmt --check

# Rustdoc link enforcement — fail on broken intra-doc links across the workspace.
# Equivalent to CI's doc-check step. Run after `just bootstrap` so deck-core
# build assets are present.
[group('occasional')]
doc-check:
    RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --workspace --exclude ship-host --no-deps

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

# Ban source-grep-in-tests (R18): tests must prove behavior, not assert on .rs text.
[group('occasional')]
lint-anti-theater:
    cargo run -p xtask --quiet -- lint-anti-theater

# docs site recipes (docs, docs-dev, docs-regen, docs-check-drift) — moved
# to ship-apps Justfile. apps/docs lives there now (the Astro site is an
# app-tier concern). Run from the ship-apps repo: `just docs` etc.

# The sui-compiler cdylib lives in the external sui repo post-carve. The
# deck bundler dlopens it for FFI compilation; the build needs the
# debug .so at target/debug/libsui_compiler.so (or platform equivalent).
# A real build-graph dependency, not a remembered step — so a
# compiler/lint source change always reaches every consumer (build,
# dev-build-all). cargo no-ops in ~2s when fresh, so it's effectively
# free.
_sui-compiler-lib:
    cargo build --manifest-path {{sui_path}}/Cargo.toml -p sui-compiler

# Same as _sui-compiler-lib for deck-cli (the build-assets + app-scaffold
# runner that ship's `just build` invokes). Lives in the external sui
# repo post-carve; ship runs it via the explicit manifest path.
_deck-cli:
    cargo build --manifest-path {{sui_path}}/Cargo.toml -p deck-cli

# One-shot production build: gen-types, every pack + ui-components rebuild,
# catalog regeneration, cargo --release, then verify outputs. Equivalent to
# `ship build`.
[group('daily')]
build: _sui-compiler-lib _deck-cli
    #!/usr/bin/env bash
    set -euo pipefail
    command -v bun >/dev/null || { echo "build: bun not on PATH" >&2; exit 1; }
    command -v jq >/dev/null || { echo "build: jq not on PATH" >&2; exit 1; }
    command -v cargo >/dev/null || { echo "build: cargo not on PATH" >&2; exit 1; }
    # Sui repo must be present for the deck-cli / sui-compiler invocations
    # to resolve. The SUI_PATH env var points at the sui repo root.
    [ -n "{{sui_path}}" ] && [ -f "{{sui_path}}/Cargo.toml" ] || {
        echo "build: SUI_PATH is unset or points to a non-sui directory — set SUI_PATH to the sui repo root" >&2
        exit 1
    }
    echo "[1/7] gen-types (no-op: removed)"
    # gen-types was a no-op (TS outputs moved to ship-contracts); the step
    # is removed entirely. The remaining build chain does not depend on it.
    echo "[2/7] sui-compiler cdylib (target/debug/libsui_compiler.so — FFI lib the bundler loads)"
    # Steps 3-7 drive the SUI compiler over Bun FFI against this .so. Nothing
    # out-of-band — the explicit --manifest-path guarantees the .so is from
    # the same sui checkout. Cheap no-op when fresh.
    # Post-carve: deck-cli is in sui's repo, AND it walks UP from cwd to
    # find its framework singletons (`packages/sui/runtime/...`,
    # `packages/deck/router/...`). Run from the sui repo so the walk
    # lands at sui's root, not ship's. cwd: the SUI_PATH directory.

    # App scaffolds (the daemon-served / and /space/<id>). Built from the
    # `apps/shipd/apps.jsonc` registry — the single app list, never duplicated
    # in a recipe; the root-mounted app self-contains the /_deck/* framework
    # surface (`--bundle-framework`).
    echo "[5/7] app scaffolds (from apps/shipd/apps.jsonc)"
    just _build-apps
    echo "[6/7] cargo build --release (ship-cli + shipd)"
    cargo build --release -p ship-cli -p shipd
    echo "[7/7] verify outputs"
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
    # Per-app `index.html` outputs come from the apps.jsonc registry (one app
    # list, never duplicated); framework bundles are checked by fixed path.
    app_indexes=$(sed 's://.*$::' apps/shipd/apps.jsonc | jq -r '.apps[].root + "/.deck/index.html"')
    for asset in .deck/framework/sui-runtime.js .deck/sui-marketing.js .deck/sui-arcade.js .deck/catalogs/workspace.json $app_indexes; do
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
    cargo build --features unstable -p ship-cli -p shipd && cargo build --manifest-path {{sui_path}}/Cargo.toml -p sui-lsp

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
    cp -f "{{sui_path}}/target/debug/sui-lsp" ~/.local/bin/sui-lsp
    echo "Installed: ship + shipd + sui-lsp → ~/.local/bin"

# Install stable binary to ~/.cargo/bin.
[group('occasional')]
install: build-release
    #!/usr/bin/env bash
    set -euo pipefail
    target=$(cargo metadata --format-version 1 --no-deps --offline | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    cargo install --path apps/ship-cli
    cp -f "$target/release/shipd" ~/.local/bin/shipd

# Rebuild the in-browser SUI template compiler. The sui-compiler crate
# lives in the external sui repo post-carve; wasm-pack writes into
# `<SUI_PATH>/packages/sui/compiler-wasm/pkg/` (gitignored in that
# repo) so the package's own package.json + .gitignore stay intact;
# the sui's bun workspace re-exports the generated entry via
# `main` / `exports`. Required before
# `cd $SUI_PATH && bun run build` — the docs playground imports
# `.sui` edits without a server round-trip. ~640KB wasm + ~9KB JS shim.
[group('occasional')]
sui-wasm:
    wasm-pack build {{sui_path}}/crates/sui/compiler --target web --out-dir packages/sui/compiler-wasm/pkg --out-name sui_compiler -- --features wasm

# Build the native Node-API addon the SUI test harness compiles through.
# The sui-compiler-napi crate moved to the external sui repo with
# sui-compiler. Build it there. Unlike `sui-wasm`, this is rarely run
# by hand: the harness loads whatever `cargo build` / `just build`
# already produced under `<SUI_PATH>/target/`, so a plain build
# rebuild (e.g. after editing the compiler crate while iterating on
# tests).
[group('occasional')]
sui-napi:
    cargo build --manifest-path {{sui_path}}/Cargo.toml -p sui-compiler-napi

# Regenerate Specta types (Rust → generated TS bindings).

# Regenerate the Ship SDK manifest.

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

# Verify generated event artifacts are up-to-date (CI gate — exits 1 on drift).

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
    @scripts/lint/check-dead-db-names.sh
    @scripts/lint/check-settings-single-owner.sh
    @scripts/lint/check-raw-env-reads.sh
    @scripts/lint/check-leaf-portability.sh
    @scripts/lint/check-runtime-access.sh
    @scripts/lint/check-module-dag.sh
    @scripts/lint/check-deck-package-names.sh
    @scripts/lint/check-module-docs.sh
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
