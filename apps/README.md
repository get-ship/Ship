# apps/

Product deliverables. Each subdirectory is a binary, web app, IDE extension,
or hosted service that ships as a Ship product. Language is incidental — the
Cargo workspace owns the Rust members, the pnpm workspace owns the TS members,
and both coexist over this same directory.

| App | Lang | Role |
|---|---|---|
| `shipd/` | Rust | The daemon. Hosts the kernel, projections, HTTP API, view server. |
| `ship-cli/` | Rust | `ship` command. Init, compile, use, mcp, lint, status, workspace. |
| `mcp/` | Rust | Ship's MCP server. Exposes platform tools (`step_done`, `chat_reply`, etc.) over stdio. |
| `docs/` | TS | Astro-based docs site at getship.dev/docs. Consumes `docs/` content via `apps/docs/src/content/`. |
| `site/` | TS | Marketing site at getship.dev. |
| `shell/` | TS | Web shell — the daemon-served UI shell that hosts views. |
| `shell2/` | TS | Next-gen shell (work in progress). |
| `gateway/` | TS | Public ingress for Ship Cloud — auth, routing, billing. |
| `workers/` | TS | Cloudflare Workers monorepo. Per-worker subdirs. |
| `vscode/` | TS | VS Code extension for `.ship/` config editing. |
| `ship-set-vscode/` | TS | VS Code extension for `<ship-*>` web-component packs. |

## Workspace membership

- Rust members are listed in `/Cargo.toml` `[workspace] members`.
- TS members are picked up by `pnpm-workspace.yaml`'s `apps/{docs,shell,site,gateway,workers/*}` globs.

## Where to put what

- **A new daemon HTTP route** → `apps/shipd/src/routes/`.
- **A new `ship` subcommand** → `apps/ship-cli/src/<cmd>.rs` + wire in `main.rs`.
- **A new web app** → new `apps/<name>/` with a `package.json`, then add it to `pnpm-workspace.yaml`.
- **Shared library code** → not here — see `crates/` (Rust) or `packages/` (TS).
