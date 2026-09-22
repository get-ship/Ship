# apps/

Product deliverables. Each subdirectory is a binary, web app, IDE extension,
or hosted service that ships as a Ship product. Language is incidental — the
Cargo workspace owns the Rust members, the pnpm workspace owns the TS members,
and both coexist over this same directory.

| App | Lang | Role |
|---|---|---|
| `shipd/` | Rust | The daemon. Hosts the kernel, projections, HTTP API, view server, and leaf Connect wires (git/harness/agents/data — formerly helmd). |
| `shipctl/` | Rust | `shipctl` command. The 11 daemon verbs, doctor, and config. |
| `host/` | Rust/TS | Tauri shell (ship-host). |
| `hub/` | TS | Primary SPA, served by shipd at `/` (j-12 from ship-apps). |
| `spaces/` | TS | Per-space Tauri `--app=<id>` scaffold (j-12 from ship-apps). |

*(Historical: `apps/helmd` was a detached leaf daemon on `:2469`. Deleted j-02 step 4; all wires rehomed to shipd.)*
*(Historical: hub/spaces lived in the `ship-apps` repo; dissolved j-12.)*

## Workspace membership

- Rust members are listed in `/Cargo.toml` `[workspace] members`.
- TS members are picked up by the `workspaces.packages` array in the root `package.json` (`apps/host`, `apps/hub`, `apps/spaces`, `packages/components/ship/*`).

## Where to put what

- **A new daemon HTTP route** → `apps/shipd/src/routes/`.
- **A new `shipctl` subcommand** → `apps/shipctl/src/<cmd>.rs` + wire in `main.rs`.
- **A new agent-facing command** → built from the `bridge` crate in the `fleet` repo; the standalone binary ships there, not in `ship`.
- **A new web app** → new `apps/<name>/` with a `package.json`, then add it to root `package.json` workspaces.
- **Domain component packs** → `packages/components/ship/<pack>/`.
- **Runtime views** → `views/`.
- **Shared library code** → not here — see `crates/` (Rust) or `packages/` (TS).
