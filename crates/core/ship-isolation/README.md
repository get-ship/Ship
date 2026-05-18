# ship-isolation

Agent-process isolation primitives for Ship. **Experimental spike** — no
existing crate depends on this yet. Integration into the orchestrator
happens in a separate branch.

See `.artifacts/local-sandbox-spec/` for the research and design intent
behind this crate.

## What it is

A trait (`Isolator`) and a small policy DSL for sandboxing the agent
process the orchestrator spawns. Distinct from
`ship_substrate::services::dispatch_ports::IsolationStrategy`, which
prepares the *working directory* (git worktree). The two compose:
worktree → process sandbox.

## Backends

| Backend | Status | What it does |
|---|---|---|
| `Noop` | working | Runs the command unsandboxed. Matches today's behavior. |
| `SandboxRuntime` | working (needs Bun + npm package at runtime) | Wraps `@anthropic-ai/sandbox-runtime` via Bun subprocess. Seatbelt on macOS, bubblewrap on Linux. |
| `Youki` (feature `youki-backend`, Linux) | skeleton | Renders an OCI runtime spec; lifecycle wiring TODO. See `STATUS.md`. |
| Cloud / Fly | bootstrap only | See `docs/fly-bootstrap.md`. Sidecar binary builds; spawn helper deferred. |

## Sidecar

The in-sandbox daemon (`ship-sidecar` binary under `bin/`) speaks the
HTTP+SSE protocol defined in `src/sidecar/protocol.rs` and documented
in `docs/sidecar-protocol.md`. Today the binary implements only
`/v1/health` — the rest of the protocol is type-defined but not yet
implemented.

## Build

```
cargo check -p ship-isolation
cargo test -p ship-isolation
cargo test -p ship-isolation --features youki-backend
cargo build -p ship-sidecar
```

## Status

See [`STATUS.md`](./STATUS.md).
