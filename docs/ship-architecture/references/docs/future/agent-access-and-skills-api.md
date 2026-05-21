---
group: Architecture
title: Agent Access — Supervisor-Sidecar, Skills-as-API, One Operation Registry
description: How agents (host and sandboxed) reach the platform — one operation registry projected onto transports, the supervisor-sidecar as the universal per-agent gateway, and smart-skills as credential-free API clients.
audience: internal
status: proposal
---

# Agent access architecture

Status: **proposal**. Supersedes the ad-hoc surface split; extends
[Daemon authentication model](../auth-model.md). Nothing here ships until the
open decisions at the bottom are closed.

## Problem

Today there are three ways to reach the platform — **CLI**, **MCP**, **skills** —
and they overlap. The recurring smell: an MCP tool *plus* a skill that explains how
to call it. As long as both exist for internal agents, one shadows the other.

Three forces make this urgent:

1. **Agents run on the daemon `Full` token.** The auth middleware's `else → Full`
   branch grants admin scopes to any holder of the daemon secret, so the
   role-scoped model only engages for properly-minted MCP sessions. Agents have
   been operating as admin.
2. **Isolation removes the CLI.** A sandboxed agent has no `ship` binary (we will
   not bundle one per sidecar), so the access path cannot depend on the CLI.
3. **Skills become load-bearing but undocumented** if they're the only definition
   of a capability — prose, not a typed contract.

We need **one** access model: the same for host and sandboxed agents, secure by
construction, with no shadowing and no undocumented load-bearing skills.

## Principles

1. **One operation descriptor registry.** Every platform capability is declared
   once — id, input/output schema, required scope(s), side-effects, idempotency.
   (Charter: *"SDK from one operation descriptor; primitive schemas validated by
   one validator."*)
2. **Transports are projections of the registry, never separate declarations.**
   CLI (humans), sidecar (internal agents, via skills), MCP (external apps only).
   No surface re-declares an op — it binds to one. This is what kills shadowing.
3. **The daemon is the one auth gateway.** One credential model resolved once.
4. **The sidecar *is* the supervisor.** Every agent — Tier 0 host included — runs
   behind a per-agent sidecar that holds the credential and is the only path to
   the daemon.
5. **Skills are credential-free API clients.** A skill is *disclosure* (prose) +
   *endpoint bindings* (op id + transport + required scope). A skill never carries
   a token.
6. **Per-agent capability = compile-time skill set + runtime scope** (defence in
   depth).

## The operation registry (single source of truth)

Each operation is declared once:

```
op: agent.dispatch
  input:  <schema>
  output: <schema>
  scopes: [Dispatch]
  effects: spawns a process
  idempotent: false
```

Surfaces are **generated/bound from** the registry, not authored per-surface:

| Surface | Who | Transport | Gating |
|---|---|---|---|
| **CLI command** | humans (agents: exception) | local exec | tool-gate hook denies agents for human-only ops |
| **Skill endpoint** | internal agents | sidecar | required scope on the agent's macaroon |
| **MCP tool** | **external apps only** | HTTP/MCP | adapter; out of the internal path |

Because all three derive from one descriptor, there is nothing to shadow.

## Smart-skills as a formal API

Smart-skills already declare what they *produce* (`artifacts[]` → inferred
`event_subscriptions`). We add what they *call*: an `endpoints[]` block that binds
to operations by id.

```yaml
# SKILL.md frontmatter
name: dispatch-subagent
stable-id: dispatch-subagent
description: Dispatch a worker sub-agent for a scoped task
tags: [orchestration]
artifacts: []            # smart-skill: what it produces (existing)
endpoints:               # what it calls — the API-client contract (new)
  - op: agent.dispatch   # references the ONE operation descriptor
    transport: sidecar   # internal default; `mcp` only for the external adapter
    scope: Dispatch      # gating: the agent's macaroon must carry it
```

- The skill **references** the op by id; it does **not** re-declare schema or
  scopes — those live in the registry, validated once.
- **No credential in the skill.** Skills are copied per agent and per worktree; a
  token in a skill file would leak. The credential lives in the sidecar (below)
  and is supplied at invocation. A copied skill carries only prose + op references
  — safe to copy anywhere.
- **Progressive disclosure is the context win.** Only the in-use skill's prose
  enters the agent's context; the typed contract was never in context (it lives in
  the registry, the sidecar resolves it). MCP would force the whole tool-schema
  list up front.
- **The copy is the feature.** A different per-agent skill set *is* per-agent
  capability: don't compile `dispatch-subagent` into a subagent → it has no
  `agent.dispatch` endpoint, backstopped by a missing `Dispatch` scope.

## The supervisor-sidecar (universal per-agent gateway)

There is **one** `Sidecar` abstraction. The isolation tier changes only *where it
runs*, never the model:

| Tier | Where the sidecar runs |
|---|---|
| 0 / 1 (host) | in-process / a thin local process the supervisor spawns alongside the agent |
| 2 (container) | PID 1 in the container |
| 3 (microVM) | inside the guest, over vsock |
| 4 (cloud) | in the vendor sandbox |

The sidecar:

- **Holds the agent's macaroon** (injected at spawn — never written to disk in a
  skill or compiled config).
- **Exposes the invoke channel** to the agent: a `ship`-shim on `PATH`, stdio, or
  a typed RPC (`remoc`). The harness invokes through it; it never reaches the
  daemon directly.
- **Routes to the daemon and enforces scope.**

**The supervisor does not spawn agents — it spawns sidecars that spawn agents.**
This is the load-bearing change for the host path: `ProcessExecutor` (Tier 0/1)
must wrap the agent in a sidecar, exactly as Tier 2–4 does, so the host path stops
bypassing the model (which is how agents ended up on the `Full` token).

## Credential model

- **Macaroon minted at dispatch:** `role → scopes` (`scopes_for_role`) + **caveats**
  (`workspace`, `is_sandbox`, `is_subagent`, expiry). Attenuatable — a sandbox can
  narrow it, never escalate — and offline-verifiable at the edge.
- **Held by the sidecar**, presented on every call, resolved once at the daemon
  gateway regardless of transport.
- **`else → Full` dies.** The daemon token becomes the admin/bootstrap macaroon
  (human CLI only). Agents are always role-scoped — the role-aware mint already
  threads the role at spawn; this removes the Full fallback.

## Security model (the must-get-right)

- **Skills carry no secret.** Credential lives in the sidecar, injected per-spawn.
  Copying a skill copies prose + op references, never a token.
- **No escalation.** Macaroon caveats prevent a sandbox/subagent from minting a
  broader credential; the registry's per-op required scope is the floor.
- **Human-only ops** stay on the CLI and are denied to agents by the tool-gate
  hook. The hook and the gateway **share one scope check** (the hook asks the
  daemon "may principal X call op Y?" — the same answer the gateway gives), so CLI
  and MCP/sidecar can't drift on what an agent may do.
- **Per-agent gating, concretely:** a sandbox agent is compiled without the
  `spawn-sandbox` skill *and* its macaroon lacks `SpawnSandbox`; a subagent is
  compiled without `dispatch` *and* lacks `Dispatch`. Compile-time set + runtime
  scope = two independent stops.

## Out of scope (future)

- **External MCP adapter.** The public-app door — a *different shape*: it carries
  its own auth, can present an MCP-app UI, and could expose a dispatch interface on
  any comparable host. Powerful, but it is not the internal access path and does
  not justify running MCP for Ship's own agents.
- **Macaroons for teams / cloud.** Per-tenant attenuation; `better-auth` API keys
  as the human/account root, traded for a macaroon at the gateway.
- **`remoc`** as the trusted-internal transport (sidecar↔daemon, Tauri shell↔daemon)
  where attenuation isn't needed because both ends are inside the trust boundary.

## Open decisions (must close before building)

1. **Does the one operation-descriptor registry exist today, or are ops declared
   per-surface?** This decides whether the work is "extend the skill schema" or
   "first build the registry, then extend." (Charter says it should exist; verify.)
2. **Sidecar→agent channel:** stdio vs `ship`-shim on `PATH` vs `remoc` — likely
   all exposed, with one wire protocol (`remoc`) to the daemon.
3. **Is the human CLI a privileged path, or also a scoped principal** (an
   `operator` macaroon)? Decides whether `else → Full` shrinks to "CLI only" or
   dies entirely.
4. **Macaroon library + verification location** (daemon-only vs offline at the
   edge for sandboxes).

## Migration sketch (incremental, behavior-preserving)

1. Confirm/build the operation-descriptor registry as the single source.
2. Add `endpoints[]` to `skill.schema.json` — declaration only, no behavior change.
3. Introduce the `Sidecar` abstraction for Tier 0/1 (supervisor spawns
   sidecar→agent); inject the macaroon; expose the invoke channel.
4. Mint role-scoped macaroons at dispatch (role is already threaded) and remove
   `else → Full`.
5. Route skill endpoints through the sidecar; retire internal MCP.
6. Keep MCP behind a flag as the external adapter only.
