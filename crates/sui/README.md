# crates/sui/

The SUI view framework — Ship's reactive view stack. Authors write `.sui` view
files; SUI compiles them ahead-of-time to ESM JavaScript modules attached to a
signal-graph runtime, and the daemon serves the compiled assets.

## Crates

| Crate | Workspace member? | Role |
|---|---|---|
| `ship-sui/` | yes | Public re-exports + integration glue. The crate other crates depend on when they want "SUI". |
| `ship-sui-host/` | yes | Embeds SUI into a host process — resolver, bundle pipeline. Used by `shipd` and the LSP. |
| `ship-sui-server/` | yes | HTTP-side SUI: view payload assembly, asset endpoints, view-host JS shim. |
| `sui-compiler/` | yes | `.sui` → ESM JavaScript transform. Pure (no FS, no env). |
| `sui-lint/` | yes | Lint rules for `.sui` views. Re-uses `crates/core/lint`'s Finding/Severity. |
| `sui-signals/` | yes | Signal-graph runtime — the fine-grained reactivity layer. |
| `ship-ui-framework/` | **no** | Build-output target, not a crate. Bundler emits assets here; `shipd` `include_str!`s them. Untracked in git. |

## Layer convention

- `sui-compiler`, `sui-signals`, `sui-lint` are **pure leaves** — no native I/O, WASM-safe.
- `ship-sui-host`, `ship-sui-server` are integration crates — they depend on `ship-substrate` for FS/env access.
- `ship-sui` is the public umbrella.

## Why nested

These seven members form one coherent subsystem. Keeping them flat in
`crates/` hid the cluster from `ls`. Nesting under `crates/sui/` makes the
boundary explicit at directory level: `crates/core/` is the substrate +
platform tier, `crates/sui/` is the view framework, and `crates/{lsp,xtask}/`
are standalone tools.
