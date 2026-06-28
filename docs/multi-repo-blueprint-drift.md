# Blueprint drift handoff — `chore/umbrella-resolution` phase 2

The ship repo just shipped the post-multi-repo cleanup (5 commits on
`chore/umbrella-resolution`, all local; remote-push pending the operator's
network auth). The helm blueprint's `system: <id>` claims drifted from
this repo's actual contents. This file enumerates the drift for the helm
agent to act on.

## What shipped (commits on `chore/umbrella-resolution`)

```
497b90aee chore(just): drop recipes that referenced moved-out code
0a8c5ff1e feat(xtask): gen-sdk --out <path> flag for cross-repo output
b6a423b00 chore(ship): gitignore the local untracked residue
e102b889f ci(ship): prune dead steps + scope to the new ship repo surface
42bd98126 test(ship): prune test/* — ship-correctness only; rest moved out
```

Plus, in sibling repos:

- `ship-apps` (branch `refactor/decouple-sui-out`) — commit
  `eb7cea2 test(hub): import e2e suite from ship monorepo cleanup`
  imports the full e2e suite + visual baselines + `_shared/prod-guard.mjs`
  at `apps/hub/test/{e2e,_shared}/`.
- `sui` (branch `feat/framework-boundaries`) — commit
  `9bca414 test(sui): import integration + parity + perf suites from ship
  monorepo` imports the integration (resource symmetry), parity, and perf
  suites at `tests/{integration,parity,perf}/`.

## Blueprint drift — components whose `system:` claim is wrong

`data/components/<id>.json` records the system's claim. The post-cleanup
reality is:

### Currently `system: ship` — should change

| id | current `system` | where it really lives now | action |
|---|---|---|---|
| `hub` | ship | `ship-apps/apps/hub/` | re-point to `ship-apps` (likely as `type: app`); or define a new `ship-apps` system entry |
| `host` | ship | `ship-apps/apps/host/` (Tauri webview) | re-point to `ship-apps` |
| `vscode` | ship | **removed** (chore commit `1a3077a0a`) | mark as removed or retire the record |
| `settings` | ship | **removed** (the shell's settings were ported to the hub) | retire record |
| `authority` | ship | **removed** | retire record |
| `marketplace` | ship | **removed** (no marketplace app) | retire record |
| `desktop` | ship | **removed** | retire record |
| `ship-gateway-client` | ship-cloud | `ship/crates/core/ship-gateway-client/` (still in this repo) | **stay** — the client crate is part of ship's contract surface even though the gateway moved to ship-edge. Note the asymmetry in the blueprint: gateway `system: ship-cloud` (correct), but the matching client stays in ship. |

### Currently `system: ship` — correct (legitimate ship code)

| id | rationale |
|---|---|
| `shipd` | `apps/shipd/` — the daemon lives here, this is the right system |
| `ship-cli` | `apps/ship-cli/` — the CLI binary for the daemon, this is the right system |
| `helmd` | (system: helm, not ship, but `apps/helmd/` lives in this repo) — `helmd` is a thin spike crate for git land/release ops, owned by ship |
| All `ship-substrate*` and `ship-*` L0/L1 crates | belong to ship as the platform host |
| `triggers`, `collections`, `account`, `agents`, `analytics`, `artifacts`, `cli`, `desktop` (the substrate crate, not the app), `durable-execution`, `pipelines`, `search`, `sessions`, `ship-auth-types`, `ship-js`, `ship-kernel`, `ship-pipelines`, `ship-query`, `ship-runner`, `ship-store`, `ship-substrate`, `ship-types`, `ship-substrate-*` | all live in `crates/` and are part of the ship platform |

### Currently `system: ship-cloud` — verify

| id | current | reality | action |
|---|---|---|---|
| `cloud-sync` | ship-cloud | `ship-edge/` likely | verify, may need re-point |
| `gateway` | ship-cloud | `ship-edge/gateway/` (178 source files) | **correct** |
| `gateway-publish` | ship-cloud | `ship-edge/gateway/` likely | verify |
| `workers` | ship-cloud | `ship-edge/workers/` (158 source files) | **correct** |
| `ship-gateway-client` | ship-cloud | `ship/crates/core/ship-gateway-client/` (the client crate, lives in ship) | **drift** — see above |
| `ship-cloud` | ship-cloud | The cloud-as-RPC-service system | verify, may need a separate `ship-cloud` repo entry (the cloud repo is a future split per `architecture-decisions.md` § "Multi-repo bounded contexts") |
| `spaces` | ship-cloud | actually `ship-apps/apps/spaces/` | **drift** — re-point to `ship-apps` |

### Currently `system: marketing-site` — correct

| id | rationale |
|---|---|
| `site` | lives in `ship-marketing/` (separate repo) |
| `docs` | `ship-apps/apps/docs/` (Astro site) — actually a `ship-apps` system claim might be more accurate; verify |

## Components the blueprint is missing

The blueprint has no records for some code that just landed or moved in:

- `ship-apps` system — there's no top-level `data/systems/ship-apps.json`.
  The intent capture work the operator has queued will need this to claim
  the hub, spaces, docs, ship-* component packs, and ship-host.
- `test-setup` (lived at `packages/test-setup` in the monorepo, moved to
  `sui/packages/sui/test-setup/`). The SUI test harness deserves a
  component record if it's load-bearing for the framework's CI.
- `arcade-engine` (moved to `ship-apps/packages/arcade-engine/`).
- `assets` (moved to `ship-apps` or dropped — verify).
- `gateway-sdk` (in `ship-contracts/packages/gateway-sdk/`, no contract
  record exists).
- `vscode` (remove), `authority` (remove), `settings` (remove),
  `marketplace` (remove), `desktop` (remove).
- `view-authoring-frontend` (lived at `packages/components/ship/...` —
  now in ship-apps component packs).

## What the operator needs to do

1. Define `data/systems/ship-apps.json` (or pick the system name you
   want — `ship-apps` is consistent with the rest). Same for
   `data/systems/sui.json` (already exists) and `data/systems/helm.json`
   (already exists).
2. Re-point the drifted `system:` claims above.
3. Add records for missing components (gateway-sdk, test-setup, arcade-
   engine, etc.) and retire records for removed ones (vscode, authority,
   settings, marketplace, desktop).
4. The `intent capture` — what each component is FOR — is yours (per the
   operator's earlier note). The mechanical organization is the table
   above.

## Open questions for the operator

- `host` is Tauri (Rust + thin JS webview). Does the blueprint claim
  `host` as `ship-apps:host` or split it (the Rust binary in `ship`, the
  webview JS in `ship-apps`)? Current code co-locates both in
  `ship-apps/apps/host/`.
- `helmd` is currently `system: helm` but lives at `ship/apps/helmd/`.
  Is `helmd` a ship-owned tool that uses the helm blueprint, or a helm
  component? The intent is the former (it's shipd's land/release helper),
  but the system name says otherwise.
- `settings` was a shell/hub surface that got ported; should the
  blueprint describe it as a `hub` sub-feature rather than a top-level
  component?
