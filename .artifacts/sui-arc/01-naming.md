# SUI Arc — 01 — Naming and Placement

Status: draft for review.
Scope: the rename + cleanup pass that aligns repository names with the layer cut in `00-layers.md`. No behavior changes — paths, package names, and import sites only.

## Prefix convention

| Prefix | Layer | Meaning at a glance |
|---|---|---|
| `sui-*` | 1–2 | SUI framework internals. Pure framework, no Ship knowledge. |
| `ship-sui-*` | 3 | Ship's *integration* of SUI. Host wiring, the bridge between Ship and the framework. |
| `ship-*` | 3 | Other Ship product modules (SDK, design system, shell). Not framework. |
| `ship-set*` | legacy | `.html.j2` + Lit. Frozen. Do not rename. Deleted at SUI parity. |

The visual signal is the prefix. Skim a directory listing and you immediately know whether you're looking at framework, integration, product, or going-away code.

## Cleanup — ghost directories (no `src/`, no `package.json`)

These were deleted from git but pnpm/dist leftovers remain. Zero git-tracked files in any. No live imports found anywhere in the codebase. Safe to delete on disk:

- `packages/ship-set-dom` — only `node_modules/`
- `packages/ship-set-eval` — only `node_modules/`
- `packages/ship-set-sdk-client` — only `node_modules/`
- `packages/ship-set-lsp` — only `node_modules/` and `dist/`
- `packages/ship-set-runtime` — only `node_modules/`, `dist/`, empty `test/`

After deletion, `pnpm install` regenerates whatever's needed for the remaining packages. The `.cache/*.inputs.json` entries in `apps/shipd/assets/.cache/` that mention these names are stale cache keys; they will be overwritten on next build.

## Rename plan — `packages/`

| From | To | Why |
|---|---|---|
| `packages/ui-runtime` | `packages/sui-runtime` | SUI layer 1/2 framework, not generic UI. |
| `packages/ui-dom` | `packages/sui-dom` | SUI layer 1 DOM bindings. |
| `packages/ui-eval` | `packages/sui-eval` | SUI eval rig — framework infra. |
| `packages/ui-lsp` | `packages/sui-lsp` | SUI LSP — framework infra. |
| `packages/ui-sdk-client` | `packages/ship-sdk-client` | Ship product (transport), not framework. Implements SUI's `TransportClient`. |
| `packages/ui-components` | *(keep)* | Ship design system built on SUI. Rename to `ship-ui-components` deferred — not load-bearing for this arc. |
| `packages/compiler` | *(keep — unrelated to SUI)* | `@ship/compiler` is the **agent-config** wasm artifact built by `wasm-pack` from `crates/core/compiler` (see `Justfile:320`, root `package.json:8`). Compiles agent JSONC → Claude/Cursor/Codex/Gemini/OpenCode provider configs. Loaded by Studio at runtime. Belongs to Ship's agent infrastructure, not SUI. If a future wasm build of `crates/sui-compiler` is needed, it lives in a new sibling `packages/sui-compiler-wasm`, not here. |
| `packages/sdk` | *(keep)* | Ship SDK. Not framework. |
| `packages/ship-set` | *(keep)* | Legacy Lit + `.html.j2` infra. Load-bearing until SUI parity. |
| `packages/assets`, `packages/cloud-types` | *(keep)* | Out of scope. |

Package-name moves (the `name` field in `package.json`):

| From | To |
|---|---|
| `@ship/ui-runtime` | `@ship/sui-runtime` |
| `@ship/ui-dom` | `@ship/sui-dom` |
| `@ship/ui-eval` | `@ship/sui-eval` |
| `@ship/ui-lsp` | `@ship/sui-lsp` |
| `@ship/ui-sdk-client` | `@ship/ship-sdk-client` (or `@ship/sdk-client`) |

## Rename plan — `crates/`

| From | To | Why |
|---|---|---|
| `crates/sui-core` | `crates/ship-sui-host` | It is host integration (reads `SHIP_DEV_PROJECT_DIR`, resolves project paths, orchestrates compile+bundle), not framework internals. |
| `crates/sui-server` | `crates/ship-sui-server` | Axum handlers. Host. |
| `crates/ship-ui-framework` | `crates/ship-sui` | Single short public facade name for shipd's imports. Replaces awkward `ship-ui-framework`. |
| `crates/sui-compiler` | *(keep)* | Framework. Already correct. |
| `crates/sui-signals` | *(keep)* | Framework. Already correct. |
| `crates/sui-lint` | *(keep)* | Framework. Already correct. |

Crate-name moves (the `name` field in `Cargo.toml`):

| From | To | Lib name |
|---|---|---|
| `sui-core` | `ship-sui-host` | `ship_sui_host` |
| `sui-server` | `ship-sui-server` | `ship_sui_server` |
| `ship-ui-framework` | `ship-sui` | `ship_sui` |

## Update sites

Each rename touches the following surfaces. The migration order below batches them so the workspace stays compilable after each step.

### TS package renames (per package)

1. `package.json` → `name` field, plus directory.
2. `pnpm-workspace.yaml` — no change (uses `packages/*` glob), but `pnpm-lock.yaml` regenerates.
3. Every importer's `package.json` `dependencies` / `devDependencies`.
4. Every `import ... from '@ship/<old>'` site.
5. Path aliases in `tsconfig.json`s if any.
6. Vite `resolve.alias` entries in `apps/shell2/vite.config.ts` (and `apps/shell`, `apps/docs`, `apps/site`).
7. Rust serving code that references package names in URL paths or asset filenames.
8. Cache keys under `apps/shipd/assets/.cache/*.inputs.json` (let them regenerate).

### Rust crate renames (per crate)

1. Directory rename.
2. `Cargo.toml` `[package].name` (and `[lib].name` if explicit).
3. Workspace `Cargo.toml` `members` list.
4. Every dependent crate's `Cargo.toml` `[dependencies]`.
5. Every `use ::<oldname>` import.
6. `cargo build` after each rename to surface remaining references.

## Migration order

Mechanical, one rename per commit, build green after each:

1. **Delete ghost dirs** (five `packages/ship-set-*` empties).
2. **TS renames (framework first):**
   1. `ui-runtime` → `sui-runtime`
   2. `ui-dom` → `sui-dom`
   3. `ui-eval` → `sui-eval`
   4. `ui-lsp` → `sui-lsp`
3. **TS rename (transport):**
   1. `ui-sdk-client` → `ship-sdk-client`
4. **Rust renames (host):**
   1. `sui-core` → `ship-sui-host`
   2. `sui-server` → `ship-sui-server`
   3. `ship-ui-framework` → `ship-sui`
5. **Update `apps/shipd` imports** to the new crate names (single commit; should be the smallest patch).
6. **Regenerate** lockfiles and `apps/shipd/assets/.cache/` on next build.

## Out of scope for this file

- Renaming `packages/ui-components` (waits until SUI parity, then becomes `ship-ui-components`).
- The render/stack split inside `packages/sui-runtime` (default entry vs `/data` sub-entry). That's a runtime layering decision specified in `00-layers.md` and `08-data-resource.md`; the package directory does not split.
- A wasm build of `crates/sui-compiler`. If/when added, lives at `packages/sui-compiler-wasm`. Out of scope here.
- Anything about `ship build`, dist artifact layout, or how installed apps are stored. Layer 4.

## Acceptance

This file is satisfied when:

- The five ghost dirs are gone.
- Every TS framework package uses the `sui-*` prefix.
- Every Ship-host crate uses the `ship-sui-*` prefix.
- `cargo build` and `pnpm install && pnpm -r typecheck` pass on a clean checkout.
- A new contributor reading the directory listing of `packages/` and `crates/` can correctly classify each entry as framework / integration / product / legacy without opening any file.
