# apps/

Product deliverables. Each subdirectory is a binary, web app, IDE extension,
or hosted service that ships as a Ship product. Language is incidental — the
Cargo workspace owns the Rust members, the pnpm workspace owns the TS members,
and both coexist over this same directory.

| App | Lang | Role |
|---|---|---|
| `shipd/` | Rust | The daemon. Hosts the kernel, projections, HTTP API, view server. |
| `shipctl/` | Rust | `shipctl` command. The 11 daemon verbs, doctor, and config. |
| `docs/` | TS | Astro-based docs site at getship.dev/docs. Consumes `docs/` content via `apps/docs/src/content/`. |
| `site/` | TS | Marketing site at getship.dev. |
| `shell/` | TS | Web shell — the daemon-served UI shell that hosts views. |
| `gateway/` | TS | Public ingress for Ship Cloud — auth, routing, billing. |
| `workers/` | TS | Cloudflare Workers monorepo. Per-worker subdirs (`platform/`, `project/`, shared `drizzle/`, `types/`). |
| `vscode/` | TS | VS Code extension for `.ship/` config editing. |

## Workspace membership

- Rust members are listed in `/Cargo.toml` `[workspace] members`.
- TS members are picked up by the `workspaces` array in the root `package.json` (`apps/{docs,shell,site,gateway,workers/*}`, `packages/*`, `packages/components/*`).

## Where to put what

- **A new daemon HTTP route** → `apps/shipd/src/routes/`.
- **A new `shipctl` subcommand** → `apps/shipctl/src/<cmd>.rs` + wire in `main.rs`.
- **A new `ship-code` agent command** → `crates/code/ship-code/src/bin/<cmd>.rs` + wire in `ship_code.rs`.
- **A new web app** → new `apps/<name>/` with a `package.json`, then add it to `pnpm-workspace.yaml`.
- **Shared library code** → not here — see `crates/` (Rust) or `packages/` (TS).
