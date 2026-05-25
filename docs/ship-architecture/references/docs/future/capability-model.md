---
group: Architecture
title: Capability Model — design + migration
description: Per-domain capabilities as the single grant axis — derived from proto, no second curation, with the bounded migration from the coarse-group POC.
audience: internal
status: design
---

# Capability model

The decided design for how a user grants a Ship agent authority, and the bounded
migration from today's proof-of-concept (the two hardcoded coarse groups). Builds
on [agent access](/reference/architecture/agent-access/) (the transport + gate)
and the [auth model](/reference/architecture/auth-model/) (the grant plumbing).

## Decisions (settled in design)

1. **A capability = an op domain.** `git`, `pipelines`, `rules` — the first
   segment of an op name (`git.commit` → `git`). This is the **one curation that
   already exists**: the app→crates→proto-service split. We do **not** curate a
   second taxonomy on top of it — that's the thing that drifts.
2. **The registry is derived from proto**, never hand-listed:
   `ship_ops_rpc::policy::capabilities()` (already built). Capability ↔ sdk-skill
   is 1:1 (`git` ↔ `ship-git`). A conformance test asserts every command's
   capability is a registry entry (or an explicit operational exclusion), so the
   gate axis, the capability list, and the sdk-skills cannot desync.
3. **Capability is orthogonal to blast-radius.** Blast (C0/C1/C2) is an *internal
   risk* calc (how dangerous), not a grantable thing. The gate multiplies them:
   `blast (auto-safety) × capability (granted scope)`.
4. **Skills only `require`; the user grants; the runtime manages.** A skill/app
   `requires:` the capabilities it needs (by id). Granting is the user's act
   (operator-approved); the runtime holds the grant (#85 provisioned baseline)
   and the gate enforces it. **Skills do not `grant:`** — that field is the
   misleading POC artifact and is removed.
5. **sdk-skills are an implementation detail.** Users never add them. Granting a
   capability attaches its sdk-skill (the `ship ops call` interface). `requires: [git]`
   → the compiler pulls in `ship-git` by id.
6. **Presets, not a second taxonomy.** The ~8 use-case bundles ("Developer" =
   `{git, pipelines, runtime}`) are **named grant-sets** for the UI so a user
   isn't ticking 25 boxes. They expand to capabilities at grant time and carry
   **no authority of their own** — tunable freely, can't desync the gate. Built
   when the UI needs them; deferred.

## The gate (target)

`decide(blast, capability, caller)`, pure:

```text
operator                        -> Allow (full trust)
C2 (elevated)                   -> Deny  (agents never; operator above)
C0 (read)                       -> Allow (observation is non-harmful, #95)
C1 (write), capability granted   -> Allow
C1 (write), capability missing   -> Deny
```

The tightening vs the POC: today most domains are `ship-workspace`,
**unrestricted — any agent writes them**. Target: an agent writes in domain `X`
only if granted capability `X`. Reads stay open.

## The bounded migration (from the coarse-group POC)

One coordinated change — the grant *tokens* flip from coarse groups
(`ship-orchestration`/`ship-authoring`) to domains, on the **same plumbing**
(`provisioned_grants` store, `AgentGrantsProvisioned` event, `resolve_grants`,
`AuthContext.grants`). No new system; a vocabulary swap + the gate de-hardcode.

| # | File | Change |
|---|------|--------|
| 1 | `shipd/services/gate.rs` | replace `classify`→`Domain{group,restricted}` with `capability_of(name)` (= `domain_of`); `decide(blast, capability, caller)` does `cap ∈ caller.capabilities`; `Caller.restricted_grants` → `capabilities`. Conformance test: every command's capability ∈ registry (or operational). |
| 2 | `shipd/services/command.rs` | call site: `capability_of(name)` + new `decide` signature. |
| 3 | `compiler/resolve.rs` | `include_capability_skills`: `requires:[cap]` → include pool skill `ship-<cap>` by id; `requested_grants` = union of `requires`; `unmet` = requires with no `ship-<cap>` in pool. |
| 4 | `compiler/types/skill.rs` | remove the `grants` field + `grants_from_skills`; rework `unmet_requirements`. ~6 construction sites. |
| 5 | `ship-platform/auth/scopes.rs` | `grants_for_role` → the role-default resolution (below); `resolve_grants` unchanged (returns the baseline, now domains). |
| 6 | callers of `resolve_grants` | `supervisor/platform.rs`, `shipd/auth/peer_socket.rs`, `shipd/auth/scopes.rs` — unchanged code, now carry domains. |
| 7 | tests | gate, scopes, resolve tests flip from coarse-group literals to domain literals. |

Committed only when the whole set is consistent + green — flipping a subset
leaves a broken window (baseline has domains, gate checks groups → all denied).

## Open decision — role defaults

With per-domain capabilities, **what does an agent hold before any explicit
grant?** Three options:

- **(a) Operational baseline + explicit rest (recommended).** Every dispatched
  agent intrinsically holds the *operational essentials of being an agent* — its
  own session/pipeline lifecycle (`pipelines` step/log), `chat` (reply to the
  user), workspace reads. Everything else (`git`, `settings`, authoring…) is
  explicitly granted, seeded from the agent's skills' `requires:` at install
  (operator-approved). Matches "user grants explicitly" while keeping a worker
  able to function out of the box.
- **(b) Zero default.** An agent holds nothing until granted. Purest, but a bare
  worker can't even `step_done` until the operator grants `pipelines` — likely
  too strict for the dispatch flow.
- **(c) Keep role→capability defaults.** `grants_for_role` maps each role to a
  domain set. Convenient, but re-introduces an editorial mapping (the thing we're
  trying to avoid) and makes the role, not the user, the grantor.

Recommendation: **(a)** — define a small `AGENT_BASELINE` capability set (derived,
documented) as "what being an agent means," and treat all else as granted. This
is the one ruling needed before executing the migration.
