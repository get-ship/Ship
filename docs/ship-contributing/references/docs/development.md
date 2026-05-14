---
group: Contributing
title: Development Setup
description: Dev container, Justfile commands, building, testing, and database management.
order: 2
audience: internal
---

# Development Setup

## Dev Container

Ship provides a Podman-based dev container via `contrib/ship-dev.sh`. It packages all build tooling into an Ubuntu 24.04 image with a non-root user.

### Launcher commands

| Command | Description |
|---------|-------------|
| `ship-dev build` | Build container image from `contrib/Containerfile` |
| `ship-dev start` | Start container and attach tmux (default action) |
| `ship-dev shell` | Attach to a running container's tmux session |
| `ship-dev exec <cmd>` | Run a single command inside the container |
| `ship-dev stop` | Stop container (named volumes preserved) |
| `ship-dev logs` | Tail container logs |

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SHIP_REPO` | `~/dev/Ship` | Path to Ship repo on host |
| `SHIP_CONTAINERFILE` | `$SHIP_REPO/contrib/Containerfile` | Containerfile location |
| `SHIP_DOTFILES` | (unset) | Personal dotfiles directory for starship, helix, tmux, etc. |

### Volumes

Named volumes persist across container rebuilds: cargo registry, cargo git, build target directory, and the bun install cache. The repo itself is bind-mounted from the host. Host config directories (`.ship`, `.claude`, `.config/gh`, `.gitconfig`) are also mounted.

### Ports

| Port | Service |
|------|---------|
| 3000 | Ship Studio (Vite dev server) |
| 3001 | Ship Studio (alt) |
| 3002 | Vite HMR websocket |
| 4321 | Astro docs site |
| 6006 | Storybook |
| 51741 | Ship Studio MCP server |

## Justfile Reference

The project uses [Just](https://github.com/casey/just) as its command runner. Run `just` with no arguments to list all recipes.

### Development servers

```bash
just dev        # Studio dev server (Vite + local D1) on port 3000
just watch      # Watch and rebuild Rust on changes (cargo-watch)
just docs-dev   # Docs site on port 4321
```

### Building

```bash
just build          # CLI + MCP with unstable features (dev default)
just build-release  # Stable release binary (what users get)
just install        # Build release + install to ~/.cargo/bin
just wasm           # Rebuild WASM compiler via wasm-pack
```

### Testing

```bash
just test             # All tests (Rust + web)
just test-rust        # Rust workspace with unstable features
just test-rust-stable # Stable surface only (CI release gate)
just test-runtime     # Runtime crate only (fastest Rust loop)
just test-compiler    # Compiler crate only
just test-web         # Web app tests (vitest)
just typecheck        # TypeScript type check (tsc --noEmit)
```

### Linting and formatting

```bash
just lint       # Rust clippy with unstable features, warnings as errors
just fmt        # Format Rust code (cargo fmt)
just fmt-check  # Check formatting without modifying (CI check)
```

### Database

Ship Studio uses Cloudflare D1 (SQLite) with Drizzle ORM for migrations.

```bash
just db-migrate         # Apply D1 migrations locally
just db-migrate-remote  # Apply migrations to production
just db-reset           # Delete local D1 data and re-migrate
just cf-types           # Regenerate Cloudflare worker types from wrangler.jsonc
```

### Housekeeping

```bash
just clean        # Remove cargo target/ and web dist/
just rebuild      # Full rebuild: clean, build, wasm, bun install, db-migrate
just check-gates  # Verify both stable and unstable configurations compile
```

### Generated TS bindings (`cargo xtask gen-types`)

The `xtask` crate (`crates/xtask/`) emits TypeScript types from Rust
structs via [specta](https://docs.rs/specta). Three files are produced:

| Output | Source registration | Consumers |
|--------|---------------------|-----------|
| `packages/ship-set/src/generated.ts` | `collect_types()` | `@ship/set/types`, `@ship/sdk` |
| `packages/cloud-types/src/generated.ts` | `build_cloud_types()` | Cloudflare worker, cloud event router |
| `packages/sui-runtime/src/generated.ts` | `collect_framework_types()` | `@ship/sui-runtime`, `@ship/sdk-client` |

**When you add a new Rust struct that crosses the daemon ↔ runtime
boundary:**

1. Derive `specta::Type` (gated on the crate's `specta` feature):

   ```rust
   #[derive(Serialize)]
   #[cfg_attr(feature = "specta", derive(specta::Type))]
   pub struct ViewSomething { /* … */ }
   ```

2. Register it in `crates/xtask/src/main.rs` — `collect_framework_types()`
   for daemon ↔ runtime data, `collect_types()` for project config.
3. Run `cargo xtask gen-types` and commit the regenerated `.ts` file.
4. Import the generated type in TS via `@ship/sui-runtime` (re-exported
   from `packages/sui-runtime/src/index.ts`) — never hand-write a
   parallel `interface`.

**CI gating.** `just test` runs `cargo xtask gen-types --check` before
the test suite. The check fails when the on-disk `.ts` files differ
from what the current Rust source would produce — i.e. someone changed
a struct without regenerating. Fix by running `cargo xtask gen-types`
and committing the result.

## Rust Workspace

The workspace is split across `apps/` (binaries) and `crates/core/` (libraries).

**Binaries:**
- `ship-cli` -- the `ship` CLI. Clap for argument parsing. Delegates to runtime and compiler.
- `mcp` -- the MCP server. Stdio and HTTP transports. Exposes runtime operations as MCP tools via rmcp.

**Libraries:**
- `compiler` -- pure transformation. `ProjectLibrary` in, `CompileOutput` out. No filesystem, no network. Builds as native and WASM.
- `runtime` -- state management. Owns the SQLite database, workspace lifecycle, sessions, events, jobs, file claims, skill vars.
- `cli-framework` -- shared CLI metadata.
- `mcp-framework` -- shared MCP app lifecycle.

The `unstable` feature flag gates development-only features. `just build` enables it. `just build-release` does not. CI tests both via `just check-gates`.

## Web App

Ship Studio runs at `apps/shell/` (Cloudflare Worker). The legacy TanStack Start app (`apps/web/`) has been retired — API routes now live in `apps/gateway/`.
