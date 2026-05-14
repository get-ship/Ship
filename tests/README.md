# tests/

Polyglot end-to-end and integration tests that drive a running Ship system
across multiple binaries. Not Cargo tests — these are TS/Node tests (Vitest +
Playwright) that exercise the daemon, MCP server, and shell as a whole.

In-crate tests (`crates/*/tests/`, `apps/*/tests/`) cover their own crate's
public API; this directory covers behavior that no single crate owns.

## Subdirs

| Dir | Job | Runner |
|---|---|---|
| `e2e/` | Playwright UI tests against the daemon's served views — clicks, navigation, network round-trips. | Playwright |
| `integration/` | Vitest tests crossing crate boundaries via HTTP — exercises shipd's REST + MCP without a browser. | Vitest |
| `loops/` | Event-loop and persist-then-deliver invariant tests — kernel routing, hook correlation, channel round-trips. | Vitest |
| `parity/` | Output-equivalence tests — verify the Rust SUI compiler emits byte-identical output to the TS reference. | Node script |

## Slicing rules

- **Needs a browser?** → `e2e/`.
- **HTTP-only, no browser?** → `integration/`.
- **Asserts on kernel event ordering or hook callbacks?** → `loops/`.
- **Comparing two implementations' output?** → `parity/`.

## Running

```bash
pnpm -F tests test            # all
pnpm exec vitest run tests/integration
pnpm exec playwright test tests/e2e
```

In-crate Rust tests run via `cargo test`; nothing in this directory is invoked by `cargo test`.
