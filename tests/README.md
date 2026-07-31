# tests/

In-repo test suites for the Ship daemon. In-crate tests (`crates/*/tests/`,
`apps/*/tests/`) cover their own crate's public API; this directory covers
the daemon's behavior that no single crate owns.

Post-multi-repo split (see `chore/umbrella-resolution` phase 1), the test
suites for other bounded contexts have moved out:

- `tests/e2e/` — lives at `apps/hub/test/e2e/` (j-12: was ship-apps). Owns
  Playwright UI tests against hub/spaces views.
- `tests/integration/` — moved to **sui** at `tests/integration/`. Owns
  SUI mount/dispose resource symmetry against the framework's source.
- `tests/parity/` — moved to **sui** at `tests/parity/`. Sui emit
  byte-equivalence against the Rust reference.
- `tests/perf/` — moved to **sui** at `tests/perf/`. Vendored keyed/sui
  bench harness + baselines.
- `tests/loops/` — dropped (was untracked scratch from local dev).
- `tests/_shared/prod-guard.mjs` — also at apps/hub/test/_shared (j-12 from ship-apps)
  `apps/hub/test/_shared/`. Long-term, both consumers should import from
  a single source in ship-contracts.

## What stays here

| Dir | Job | Runner |
|---|---|---|
| `agents/` | Vitest tests exercising shipd's ConnectRPC + harness adapter registry against a real daemon. | Vitest |
| `contract/` | Rust contract conformance: every proto/fixtures/{events,ops} JSON roundtrips Rust↔JSON, and registry metadata (scope/retention/stability) matches. | `cargo test -p ship-contract-tests` |
| `_shared/` | Shared test utilities. Currently just `prod-guard.mjs` (asserts the test daemon is not the operator's prod `~/.ship`). | — |

## Running

```bash
cargo test -p ship-contract-tests           # tests/contract
bunx vitest run tests/agents                # tests/agents (needs a running shipd)
```

Tests for other repos are run by their own CI; see `.github/workflows/ci.yml`
in `sui` and `ship-apps` for the dispatcher steps.
