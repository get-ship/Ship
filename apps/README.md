# apps/

Product deliverables. Each subdirectory is a binary, web app, IDE extension,
or hosted service that ships as a Ship product. Language is incidental — the
Cargo workspace owns the Rust members, the pnpm workspace owns the TS members,
and both coexist over this same directory.

| App | Lang | Role |
|---|---|---|
| `shipd/` | Rust | The daemon. Hosts the kernel, projections, HTTP API, view server. |
| `shipctl/` | Rust | `shipctl` command. The 11 daemon verbs, doctor, and config. |

## Workspace membership

- Rust members are listed in `/Cargo.toml` `[workspace] members`.
- TS members are picked up by the `workspaces.packages` array in the root `package.json` (`apps/host`).

## Where to put what

- **A new daemon HTTP route** → `apps/shipd/src/routes/`.
- **A new `shipctl` subcommand** → `apps/shipctl/src/<cmd>.rs` + wire in `main.rs`.
- **A new agent-facing command** → built from the `bridge` crate in the `fleet` repo; the standalone binary ships there, not in `ship`.
- **A new web app** → new `apps/<name>/` with a `package.json`, then add it to `pnpm-workspace.yaml`.
- **Shared library code** → not here — see `crates/` (Rust) or `packages/` (TS).
