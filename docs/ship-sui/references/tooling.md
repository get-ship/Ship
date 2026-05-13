# SUI — Tooling

The chain from `.sui` source on disk to a live, hot-reloaded view in the browser passes through six tools. Each maps to one entry point.

## `sui-compiler` — the Rust ahead-of-time compiler

- **Library:** `crates/sui-compiler/src/lib.rs`. Pure transformation; no filesystem walk, no daemon, no network. The `compile(source, opts) -> CompileOutput` entry is documented in `references/4-layer-overview.md` § Layer 1.
- **Binary:** `crates/sui-compiler/src/bin/sui-compiler.rs` — the `sui-compiler` Cargo bin. One subcommand today: `build`.

```sh
# Compile a single file. Emits to the project mirror tree
# (<repo-root>/.ship/sui-out/<src-rel>/<stem>.compiled.js by default):
cargo run -q -p sui-compiler -- build views/home/page.sui

# Compile a directory recursively, into a single output dir (legacy <stem>.js):
cargo run -q -p sui-compiler -- build views --out target/sui-debug --clean

# Component mode — implied by --component-name:
cargo run -q -p sui-compiler -- build packages/ui-components/src/sui-button/sui-button.sui \
  --component-name sui-button \
  --styles packages/ui-components/src/sui-button/sui-button.css
```

Flags (from `crates/sui-compiler/src/bin/sui-compiler.rs`):

| Flag | Purpose |
|---|---|
| `--out <dir>` | Single output directory (legacy `<stem>.js` naming). When omitted, emits to `<repo-root>/.ship/sui-out/<src-rel>/<stem>.compiled.js`. |
| `--clean` | Remove each source file's mirror directory (or `--out` directory) before emitting. Idempotent rebuilds. |
| `--mode <view\|component\|auto>` | Override mode (otherwise auto-detected from `--component-name`). |
| `--no-dts` | Suppress `.sui.d.ts` emission. |
| `--no-ssr` | Suppress `.ssr.html` emission (e.g. island bundles). |
| `--styles <file>` | Prepend a CSS file as a `<style>` block. Component sources only. |
| `--component-name <tag>` | Set the custom element tag name; implies component mode. |

The binary is also exposed as a `cdylib` (`crate-type = ["lib", "cdylib"]` in `Cargo.toml`) so editor extensions and `@ship/sui-build` can call it without spawning a subprocess. The Rust release build is what the daemon ships at `~/.ship/runtime/sui-compiler` (see `docs/ship-architecture/references/docs/toolchain.md`).

## `@ship/sui-build` — the project bundler

The Node/Bun-driven build pipeline that wraps `sui-compiler` for the whole repo. Source: `packages/sui-build/`.

```sh
sui-build                          # one-shot production build
sui-build --watch                  # rebuild on source changes
sui-build --bundler bun            # force the Bun adapter
sui-build --bundler esbuild        # force the esbuild adapter
sui-build --config path/to/sui.config.ts
sui-build --mode development       # passes through to compile flags + bundler env
```

Entry: `packages/sui-build/bin/sui-build.mjs` → `dist/cli.js` via Bun. The runner (`src/runner.ts`) walks the configured source globs, invokes `sui-compiler` (preferring the in-repo `target/debug/sui-compiler` build, falling back to `cargo run --quiet -p sui-compiler`), and feeds the emitted JS / CSS / CEM through the chosen bundler adapter (`adapters/bun.ts`, `adapters/esbuild.ts`).

Configuration: `sui.config.ts` at the repo root. Watch mode wires `chokidar` over the source globs and re-runs the affected files.

## `@ship/sui-lsp` — the language server

Source: `packages/sui-lsp/`. Provides diagnostics, hover, and completions for `.sui` files.

```sh
pnpm -F @ship/sui-lsp build           # compile TS to dist/

# Native LSP stdio (point a generic editor at this):
node packages/sui-lsp/bin/sui-lsp.js --stdio

# One-shot CLI (what MCP shims wrap):
bun packages/sui-lsp/src/cli.ts diagnostics views/home/page.sui
bun packages/sui-lsp/src/cli.ts hover views/home/page.sui --line 12 --col 4
```

Analyzers live one-per-file under `packages/sui-lsp/src/`:

| File | Owns |
|---|---|
| `parse.ts` (+ `parse-attrs.ts`, `parse-blocks.ts`, `parse-braces.ts`, `parse-types.ts`) | `.sui` parser |
| `diagnostics.ts` | Compile-grammar issues, missing bindings, unstable-key warnings |
| `setup.ts` + `setup-controller.ts` + `setup-diagnostics.ts` | Paired-controller resolution + cross-file diagnostics |
| `hover.ts` | Type + binding info at a point |
| `completions.ts` | `<sui-*>`, prop names, control-flow blocks |
| `bindings.ts` | Reactive identifier resolution against scope |
| `server.ts` | LSP plumbing — `TextDocuments`, `onDidChangeContent`, `onHover`, `onCompletion` |
| `cli.ts` | JSON-emitting CLI; used by MCP shims |

VS Code: `apps/vscode/` auto-launches the server on `.sui` open (recommended via `.vscode/extensions.json`). Other editors map the `sui` language ID to `node packages/sui-lsp/bin/sui-lsp.js --stdio`.

## MCP shims — agents that don't speak LSP

`apps/mcp/src/tools/sui_lsp.rs` bridges the LSP analyzers to MCP. Two tools:

| MCP tool | What it returns |
|---|---|
| `mcp__ship__sui_diagnostics` | Compile-grammar issues + missing bindings + lint warnings for one file |
| `mcp__ship__sui_hover` | Same content the VS Code extension shows for a `(line, col)` position |

Both spawn `bun` against `packages/sui-lsp/src/cli.ts` and parse its JSON output. CLI path resolution: `SHIP_WORKSPACE` env var, else walks upward looking for `pnpm-workspace.yaml`. Registration: `apps/mcp/src/server/mod.rs:783` (`sui_diagnostics`) and `:796` (`sui_hover`).

```ts
// Before declaring a `.sui` edit done:
mcp__ship__sui_diagnostics({ file: "views/hello/page.sui" })
mcp__ship__sui_hover({ file: "views/hello/page.sui", line: 12, col: 4 })
```

## UI Studio — the in-browser catalog

`views/ship-ui-studio/`, mounted at `/ship-ui-studio` (rail icon: `columns`).

| File | Owns |
|---|---|
| `page.sui` + `controller.ts` | Catalog index — search, filter (All / Components / Views), grouped nav |
| `[name]/page.sui` + `controller.ts` | Detail page — exercise prop knobs, slot compositions, deep-link state, render matrix |
| `catalog.ts` | The component / view registry the index reads |
| `[name]/matrix.ts` | Render-matrix configurations (prop × value grid) |
| `[name]/slot-compositions.ts` | Slot composition presets |
| `[name]/deep-link.ts` | URL-state serialization for the detail page |
| `[name]/snippet.ts` | Copy-as-snippet renderer |

Use it to find a component, browse its props (including enum values from `<prop values="…">`), exercise edge cases (long strings, empty slots, error states), and copy the matching template snippet straight into your view.

## Devtools panes — in-page introspection

`packages/ui-components/src/islands/sui-devtools/` ships five island-bundled panes. Each is a `.sui` + `.controller.ts` pair, lazy-loaded on first mount via the islands bundle.

| Tag | What it shows |
|---|---|
| `<sui-devtools-source>` | The mounted view's template + manifest. Editable for user-scoped views; read-only for project-scoped (Fork affordance for project scope). Embeds `<sui-code-editor>` lazily. |
| `<sui-devtools-components>` | Live component tree of the mounted view |
| `<sui-devtools-lint>` | SUI lint findings for the current file |
| `<sui-devtools-schema>` | Prop schema for selected component |
| `<sui-devtools-signals>` | Live signal values; subscribes + repaints on change |

`apps/shell2/src/main.ts:139` (`// Notify devtools tabs of navigation`) wires navigation events into the panes so they refresh on route changes.

## `dispatch.sh` — worktree + job-spec + agent spawn

`.ship/skills/spawn-agent/scripts/dispatch.sh` is the canonical script that creates a fresh worktree, writes a job spec, compiles the agent, and opens a terminal in the worktree. Used directly by `cmdr` and indirectly by the in-app dispatch dialog.

```sh
bash .ship/skills/spawn-agent/scripts/dispatch.sh \
  --slug docs-sui-arc \
  --agent ship-designer \
  --spec .artifacts/job-spec.md
```

Flags (from `dispatch.sh`):

| Flag | Default | Meaning |
|---|---|---|
| `--slug <name>` | required | Worktree directory name + branch name (`job/<slug>`). |
| `--agent <agent>` | required | Agent profile to compile into the worktree (`ship-designer`, `commander`, etc.). |
| `--spec <file>` | required | Path to the job-spec markdown file. Copied to `.artifacts/job-spec.md` in the worktree. |
| `--dir <path>` | resolved | Worktree base. Precedence: `--dir` > `SHIP_WORKTREE_DIR` env > `ship config get worktrees.dir` > `~/dev/ship-worktrees`. |
| `--base <branch>` | current HEAD | Base branch for the worktree. |
| `--no-open` | false | Skip opening a terminal in the worktree. |
| `--dry-run` | false | Print actions without executing them. |
| `SHIP_AGENT_MODEL` env | unset | Override the agent's default model. |

The script honours `SHIP_DISPATCH_CONFIRM` for non-interactive confirmation prompts and resolves `~` in the worktree base. It is not specific to SUI work — it dispatches any agent — but every SUI authoring job (this docs job included) lands in a worktree created by it.

## Scaffolds

```sh
# Drop a fresh view directory:
node scripts/scaffold-view.mjs settings
node scripts/scaffold-view.mjs settings --path /settings
node scripts/scaffold-view.mjs settings --component settings-toolbar

# Drop a fresh component directory:
node scripts/scaffold-sui-component.mjs my-widget
node scripts/scaffold-sui-component.mjs my-widget --controller
```

Both scaffolds emit the manifest / template / controller / style files documented in `references/authoring-patterns.md` § Scaffolds. Run them from the repo root.

## End-to-end edit loop in dev

```sh
just dev                              # daemon + bundler + shell2 in watch mode
# edit views/home/page.sui or controller.ts
#  → bundler emits module-changed on /_sui/dev/hmr (SSE)
#  → apps/shell2/src/hmr.ts cache-busts /_sui/views/home/view.js
#  → mount() re-runs in place; SSR snapshot stays, no full reload
```

For a clean dev restart use `just dev-restart`. Visual regressions: `just visual` (Playwright vs committed PNGs; pass `--update-snapshots` to refresh after a deliberate change). Perf regressions: `just bench-check` (fails on >15% vs the latest committed baseline; override via `BENCH_REGRESSION_THRESHOLD`).
