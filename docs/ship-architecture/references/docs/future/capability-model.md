---
group: Architecture
title: Authority Model — service × blast-tier
description: Agent authority as a per-service blast ceiling — the two axes that already exist (proto services × risk tier), no new taxonomy.
audience: internal
status: design
---

# Authority model: service × blast-tier

How a user grants a Ship agent authority. Supersedes the earlier
"per-domain capability" sketch — **there is no capability taxonomy.** Authority
is the two axes that already exist: the **service** an op belongs to (the proto
contract grouping) × its **blast tier** (risk). Builds on
[agent access](/reference/architecture/agent-access/) and the
[auth model](/reference/architecture/auth-model/).

## The decision

- **No new taxonomy.** A second curated grouping of the op surface (use-case
  "capabilities") would drift from the proto-service grouping. Rejected.
- **Blast radius is the lever.** Per-op risk tier, declared in proto. The bug
  that forced this: `dispatch` (spawns/controls agents) and `step_done` (a
  progress note) are both `C1` today — nonsense. The tier must separate them.
- **Four tiers** (today's three can't place `dispatch`: not "any agent", not
  "human-only-forever" since a commander dispatches):

  | Tier | Who | Examples |
  |---|---|---|
  | **C0** read | anyone | list, get, status |
  | **C1** workspace-write | any agent | step_done, log_progress, git write, edit files, collections |
  | **C2** elevated | **granted** agents (commander) | dispatch / stop / steer, settings, mcp, authoring |
  | **C3** critical | operator only (GUI/2FA) | account, permissions, billing, credential mint |

- **Authority = a per-service blast ceiling.** Internally an agent holds
  `{ default: C1, overrides: { runtime: C2 } }` — full control, the "service ×
  tier" matrix. The gate: `op(service S, blast B) → allow if B ≤ ceiling(S)`
  (operator clears all; **C3 is always operator/2FA**, never granted to an agent).
- **Low user surface via presets + a default.** The user never fills 25 cells.
  A global default (C1) + named presets set the map: `worker = {default: C1}`,
  `commander = {default: C1, runtime: C2}`, `admin = {default: C2}`. Presets are
  non-authoritative UI sugar over the ceiling map.

This makes the dispatch case work with **zero per-domain list**: `step_done`
(pipelines, C1) and `dispatch` (pipelines, **C2**) share a service but differ by
tier — a worker with `ceiling=C1` does the first, not the second.

The proto-derived capability registry (`ship_ops_rpc::policy::capabilities()`)
**survives only as the sdk-skill index** (which skills exist), not as the grant
axis.

## Execution — one coupled migration

Re-tiering the gate is coupled (proto tiers ⇄ gate ⇄ baseline); commit only when
consistent + green. Ordered:

0. **Untangle the enum-name mismatch first.** The proto blast option reads
   `BLAST_RADIUS_C0_SAFE` but Rust `BlastRadius::from_descriptor` matches
   `"C0_READ"` — confirm how `read_policies` maps the enum (i32 vs name) so the
   tier values are actually wired, *before* assigning tiers. (Likely a latent
   bug or an i32 path that bypasses the string.)
1. **`ship-ops-rpc` `BlastRadius`**: add `C3Critical`; derive `Ord`
   (C0<C1<C2<C3); `from_descriptor`/`as_descriptor` for all four; keep
   fail-closed to the **highest** tier.
2. **`policy.proto`**: add the `C3` enum value.
3. **Re-tier the ops** (the bug-fix, SSOT): `agent.*`/dispatch/stop/steer,
   `settings.*`, `mcp.*`, authoring → **C2**; `account.*`, `permissions.*`,
   billing/credential → **C3**; keep step_done/log/git-write/collections at C1.
   A conformance test pins the dangerous ones (dispatch is never < C2).
4. **`gate.rs`**: `Caller` holds a ceiling map (`default` + `overrides`); `decide`
   = operator→allow, `C3`→operator-only, else `blast ≤ ceiling(service)`. Drop
   `classify`'s coarse groups. Pure + TDD.
5. **#85 baseline**: `provisioned_grants` stores the ceiling map (default + sparse
   overrides), not a string list. `resolve_grants` returns it; `AuthContext`
   carries it; `caller_from_auth` maps it.
6. **Presets**: named ceiling maps (`worker`/`commander`/`admin`) — a tunable,
   non-authoritative table the grant UI lists. Deferred until the UI needs it.
7. **Tests**: gate/scopes/resolve flip to the ceiling model.

Why not started piecemeal: re-tiering ops without the gate ceiling (steps 1–3
alone) would **deny** commander dispatch (C2) until step 4 lands — a broken
window. The set ships together.

## Open ruling captured

Role default = **operational baseline** (ratified): default ceiling **C1**
(workspace-write everywhere) for any dispatched agent; C2 elevation is explicit
(preset or per-service override); C3 is never an agent's. A bare worker can
`step_done`/`log_progress`/reply out of the box; dispatch and settings need the
commander preset.
