---
group: Architecture
order: 3
title: The Op Surface — One Registry, One Gate, Skills as Capability Clients
description: Ship's L1 ABI — every daemon op is a ConnectRPC method in a per-domain ship.ops.v1 service authored from proto (the SSOT for shape AND policy); one capability gate enforces blast-radius × caller × grant; sdk-skills and proto fixtures generate from the same source.
audience: internal
status: implemented
---

# The op surface

Ship's L1 ABI. Every daemon operation — read or mutation — is a ConnectRPC method
in a **per-domain `ship.ops.v1` service**, authored from proto (the single source
for the op's shape *and* its policy), implemented once in the daemon, consumed by
the CLI, the GUI, and agents as generated clients. One capability gate enforces
**blast-radius × caller × grant** at the dispatch chokepoint. The same proto source
generates the sdk-skills (the agent's API docs + client) and the contract fixtures.

The landed daemon auth model is documented in
[Daemon authentication model](/reference/architecture/auth-model/); this page is the
op surface itself — how it's authored, gated, and projected — plus the last mile.

<ship-alert tone="info">
**Built (on main):** 24 authored per-domain `ship.ops.v1` services covering the full
op surface (migrated off the legacy `schema::OPS`/`COMMANDS` Rust tables), each mounted
additively beside the still-generated `ReadOpsService`/`CommandOpsService`. One
capability gate (`services::gate::decide` — caller × blast-tier × per-service ceiling,
4 tiers C0–C3, read from the proto descriptor, fail-closed to C3); an agent's elevated
services are operator-approved (provisioned baseline), `agent.grant` (C3) the approval
point — see [auth model](/reference/architecture/auth-model/); the `@ship/sdk`
Connect-Web client + event-bus/PTY
stream adapters; the **sdk-skill generator** (progressive-disclosure SKILL.md +
per-op pages) and the **proto contract-fixture** pipeline (themed corpus → per-op
fixtures, drift-checked).
**The last mile (delivered):** the `view.*`/`create.*` migration landed alongside
the studio rewrite; the GUI flip (#72) flipped onto per-domain clients in #116; the
REST cull (#137) deleted the mutation routes; the utoipa removal (#135) deleted the
OpenAPI annotations; the CLI flip (#141) moved `ship` off `/api/v1`. What remains:
the `specta` prune (#136 still tracks the last residual). The surface is no longer
additive — the legacy paths are gone.
</ship-alert>

## Goal

One internal transport, one op surface, one capability gate. Today CLI / MCP /
GUI each re-express the daemon's ops — three hand-maintained mirrors, the
recurring noise. Unify on **ConnectRPC projected from one op registry**, tag every
op with a capability + blast-radius, enforce at **one** gate. External stays an
adapter (MCP for apps; REST only for protocol-mandated edges).

## Grounding (what already exists)

- **`ship.gateway.v1`** (ConnectRPC, on main): cloud auth via Better Auth API keys
  (`ship_`, PKCE), `DaemonAttest` (Ed25519 JWT / JWKS), billing. The dangerous
  cloud ops live *here*, a separate surface with its own auth.
- **`ship.sidecar.v1`** (ConnectRPC over vsock/socket, `job/local-sandbox-spec`):
  the Tier 2–4 host↔sidecar plane — process control + a bidi `Events` stream
  (permission/secret/hook/log_progress mediated, credential never enters the
  sandbox). **Different plane from the op surface — do not entangle.**
- **`ship-gateway-client`** (`job/missing-primitives`): Rust Connect codegen via
  `connectrpc-build`. This is the tooling foundation for ops-plane Connect — *not*
  the isolation branch.
- **`ship_ops!` registry**: 54 ops, **all `.read`** (perm `ship.<domain>.read`).
  A read-only projection surface feeding the GUI/SDK (`gen_sdk → data_ts.rs`).
  Commands (mutations) are **not** here — they're in legacy HTTP routes.
- `buf.gen.yaml` + `proto/ship/gateway/v1/auth.proto` on main.

## Decision 1 — Connect for internal services

CLI and GUI become **generated Connect clients** of one op service. Internal MCP
is retired; MCP survives only as the external adapter. The dependency is the
`connectrpc-build` pattern (from `missing-primitives`) + buf — a small, targeted
adoption, decoupled from the isolation branch.

## Decision 2 — proto as the single source; `ship_ops!` macro subsumed

**LOCKED: option (a)** — proto owns shapes *and* policy (custom method options).

The win is a codegen **collapse**, and it only pays because Connect already owns two
planes (gateway + sidecar). Today the daemon runs a *second* codegen world:
`ship_ops!`/axum (Rust) → **specta** → TS, plus a bespoke OpenAPI + fixtures gen.
Meanwhile gateway + sidecar run **buf-from-proto**. Connect-all-in folds the daemon
into that one toolchain:

- **proto is the single source for the contract _and_ the policy.** Method shapes
  live in proto; capability / blast-radius / `sensitive` ride as **custom method
  options** — co-located with the method, generated into both the gate and the docs.
  buf generates Rust server+client, the TS client, OpenAPI, and fixtures — replacing
  specta and the bespoke gens (deleted). **Known cost:** prost drops custom options
  in normal codegen, so the Rust gate reads them via a `FileDescriptorSet` /
  prost-reflect step — folded into #74. Upside: capability metadata travels in the
  proto, visible to every generated client (TS/docs/SDK) for free.
- **Rust owns implementations + enforcement, not types.** The Connect-generated
  **service trait forces a handler for every method**, so 1-1 dispatch conformance
  is a *compile error*, not a runtime test. The gate reads the method option and
  checks the credential.
- **The `ship_ops!` macro is subsumed** — proto + the generated trait + the handler
  impls + method-option policy *are* the registry. One IDL, one toolchain, three
  planes.

Cost (honest): wire DTOs are rewritten as proto messages, and handlers work with
prost-generated types (less ergonomic than hand-rolled structs). Worth it **only
because** the streaming/polyglot/sidecar reasons already commit you to Connect — if
they didn't, Rust→specta is fine and proto is pure overhead. The benefit is deleting
a parallel toolchain, not "proto > specta" in the abstract.

## Read-op porting (decided during #74 build)

The registry already exists on main (`schema::OPS` + per-domain
`try_dispatch(name, args, …) -> Value`). Port the 54 **read** ops as **per-op
Connect methods returning a shared `JsonResponse { string json }`**: each method
carries its own `capability`/`blast_radius` proto options (the gate stays per-op,
#67a holds for policy), but the body is the existing projection `Value`, wrapped —
the impl calls `try_dispatch`. Ports all reads cheaply, no per-op message modeling;
upgrade to a typed response message per-op only where a stable contract earns it.
**Commands get fully-typed messages from the start** (they're the contracts that
matter). Requires `OpsImpl` to hold `ApiState`.

## Decision 3 — capability + the gate

Every op carries a **capability** and a **blast-radius class**. One central gate at
the daemon decides who may call what. `OpDescriptor.sensitive` is the seed of the
classification.

**Blast-radius classes:**

- **C0 — Safe (read / own-scope).** All 54 current ops. Harmless to Ship. Fine over
  any authenticated surface. (`sensitive` marks content-exposure for redaction/log
  hygiene — orthogonal to blast radius.)
- **C1 — Workspace-mutating (user's own resources).** dispatch, create/edit/delete
  view·skill·agent·pipeline, write artifact, activate workspace. Damage here is
  *the user's risk* (their project, their choice not to isolate). Fine for a
  host agent within scope.
- **C2 — Ship/cloud-dangerous (the elevated class).** Credential mint/rotate/revoke,
  changing **another** actor's config/permissions/role, registry publish, team /
  billing. Must require a capability that **never lands on the agent-readable
  filesystem** — minted into the human GUI session, optionally with per-op confirm
  / 2FA.

**The gate, grounded:** C2 cloud ops already live on `ship.gateway.v1` (separate
surface, Better Auth, human login) — so a compromised *host* actor cannot reach
them today. The audit's job is to confirm no C2 op hides in the *local* surface
(or the legacy routes) reachable with the on-disk daemon token, and to gate any
that do. Transport is **not** the gate — it's surface-area reduction
(defense-in-depth), same shape as MCP tool-gating. The gate is capability +
where the credential lives.

## Decision 4 — skills as generated capability clients

Generate the **machine-read half** of a capability-skill (`endpoints[]`/grant block
+ op list + a recipe stub seeded from op descriptions) from the registry, at
**use-case/domain granularity** — never one-skill-per-op (the context-bloat trap).
Prose is authored/LLM-refined. Then the loop closes:

> ops → generated capability-skills → an agent's compiled skill set = its grant set
> = its credential scope.

One source (`ship_ops!`) drives the wire, the CLI, the GUI client, *and* the agent
capability layer. Capability-skills are a distinct artifact (authoritative grant
block), not regular prose skills; the schema work formalizes that.

## Lower layers — nothing to revisit

Connect is the **process boundary**; the substrate is **in-process tokio**. The
event bus (kernel router, projections) routes over tokio mpsc/broadcast to
in-process actors — unaffected by the transport choice. The only seam is the
**internal-bus → external-stream adaptor**: subscribe to the kernel stream →
filter by the connection's capability (and directed-event visibility) → emit as a
Connect server-stream. That adaptor must handle broadcast **lag/backpressure**
(`RecvError::Lagged` → drop or disconnect policy). It's the SSE replacement, and
it's deferred (see below). No new substrate work for the request/response migration.

## Migration — additive, CLI first, streaming last

Connect, REST, and SSE coexist on one axum server (distinct paths) — so this is
**additive, not big-bang**:

1. Stand up the Connect op service alongside the current HTTP/SSE. Nothing breaks.
2. Migrate the **CLI** first (request/response; the capability gate matters most
   there). Fold dynamic scoped `--help` (registry-driven) in here.
3. **GUI stays on HTTP/SSE untouched** while the UI thread continues; it moves to
   Connect clients **last**, when the streaming adaptor is built.
4. MCP keeps its HTTP client until retired (internal) / repositioned (external).

### Consumers of the daemon wire (inventory)

- `apps/shell` (GUI) — fetch + **two SSE streams**: terminal PTY bytes
  (`/sessions/{id}/stream`), view-edit compile events (`sui_view_edit_sse`).
- `apps/mcp` (`network_client.rs`), `crates/sui/ship-sui-server` (browser runtime),
  `apps/ship-cli` + hooks, `@ship/sdk` (`packages/deck/sdk` — now carries the
  generated Connect-Web client at `src/connect.ts`).

None break under additive coexistence; only the CLI is force-migrated first, by
choice.

## Deferred decisions (not now, but named)

- **Streaming adaptor** (SSE → Connect server-streams) — ✅ landed for the event-bus
  (`StreamService.Events`) and PTY (`StreamService.SessionBytes`); the GUI consumes
  them in step 5. `ViewStream`/datastar view-morph is legacy and dies with the j2
  shell, not migrated.
- **Elevated surface** (where the C2 capability lives). The posture is settled —
  **nothing on disk is ever C2**: the on-disk token is C0/C1, C2 lives in the GUI/2FA
  session. In Tauri the C2 credential can live in the Rust core, never the webview JS
  heap (SUI XSS can't exfiltrate it); a browser can't separate that. The
  daemon-token-downgrade (`Full` → C0/C1) is the concrete first step.
- **Macaroons** — only if edge-verification-without-a-DB becomes a *measured* cost
  at cloud scale. Not launch, not near-term.

## Rejected (don't re-derive)

- **Macaroons as the credential model.** Better Auth (accounts/teams) + DaemonAttest
  (machine identity) + host-mediated sidecar (agent holds no credential in
  isolation) + scoped session tokens already cover it. Macaroons would be a parallel
  system — a "one way to do things" violation.
- **remoc.** Niche, Rust-only, weak schema evolution; Connect is already chosen and
  in repo, polyglot, versioned.
- **Per-agent supervisor *process*.** Supervision is per-environment; identity is
  per-agent *data* (see the host-mediated sidecar model).
- **Transport as the privilege axis** (http=safe / connect=privileged). The agent
  has bash and can speak any wire; privilege rides capability + credential location.
- **Coupling the isolation-branch merge to Connect-all-in.** Different planes;
  ops-plane Connect needs only `connectrpc-build`.

## Tier-1 note (launch default, host agents)

Tier 1 = in-process, no sidecar. The host agent shares uid/filesystem with the
daemon, so the on-disk daemon token is readable; the `.claude/settings.json`
denylist is a band-aid, not a boundary. Posture: **host agent is trusted-as-the-user
for C0/C1; untrusted/risky work goes to Tier 2+ isolation; C2 is gated on the human
capability regardless.** The `SHIP_TOKEN` CLI fix is hygiene/observability, not
load-bearing security.

## Tracked plan

1. ✅ **Op-classification + blast-radius audit** (C0/C1/C2).
2. ✅ **proto-from-registry** + conformance (Decision 2).
3. ✅ **Connect op service, additive** + migrate CLI + scoped `--help` (Decision 1).
4. ✅ **Generate capability-skills** from the registry (Decision 4).
5. ✅ **proto-SSOT per-domain migration** — the op surface is now **24 hand-authored
   `ship.ops.v1` services** (rules, artifacts, collections, permissions, mcp, skills,
   pipelines, git, agents, agent, project, settings, annotation, feedback, sessions,
   workspaces, account, chat, dashboard, events, specs + analytics/runtime/view_errors),
   one rpc per op, policy as proto method options read by the gate. Authored, not
   generated from the Rust tables; mounted additively beside the generated umbrella
   until the flip. Recipe: author `<domain>.proto` (fresh request names — the generated
   protos still hold `*Request` names in-package; reuse shared returns from `types.proto`,
   `JsonResponse` from `ops.proto`); impl `<Domain>Service for OpsImpl` (C0 reads call the
   platform service directly; writes route through `services::command::call`, the one
   gate); register additively. Exemplars: `rules_rpc.rs`, `artifacts_rpc.rs` (typed
   returns), `collections_rpc.rs` (opaque JSON).
6. ✅ **sdk-skills** — progressive-disclosure generator (`xtask gen-skills`): a tiny
   `SKILL.md` TOC + one `references/docs/<op>.mdoc` per op, doc-site-frontmatter ready,
   at `skills/sdk/ship-<domain>/`. Operational domains (mesh/actor/event) excluded (#95).
7. ✅ **proto contract fixtures** — `xtask gen-fixtures` validates a coherent themed
   corpus (`proto/fixtures/theme.json`) against each proto type and emits per-service
   fixtures (75 typed ops); cross-language contract tests + a CI drift guard. Serves
   contract testing, fuzz seeds, and design handoffs from one source.
8. ⏳ **The last mile — on `sui-framework`:** `view.*`/`create.*` migration (coupled to
   the studio rewrite), the **GUI flip** (#72) off the umbrella onto the per-domain
   clients, then the **deletions** below. Gated there because the flip + view/create are
   the studio-controller contract that branch owns.

### Then the deletions (the payoff this arc exists for)

The migration was **additive** during the arc — Connect stood beside REST/OpenAPI/
SSE/specta. The deletion phase has now landed:

- **REST + OpenAPI** (#137, #135) — mutation routes deleted, `utoipa` annotations
  removed. No strangler-fig needed (no external consumers).
- **specta** (#136 still tracks the residual) — once TS types come fully from buf,
  `derive(specta::Type)` strips from the domain crates.
- **`Scope`** — the enum dies with the REST route-auth (`RequiredAuth::Scope`) +
  artifact perms that still consume it. The REST route-auth piece is already gone;
  artifact perms are the last consumer.

### Capability provisioning (the grant source)

`grants_for_role` is the interim mint source. The target is **operator-approved,
per-agent provisioned grants**: a skill declares `requires:`; the operator approves
at install/config (marketplace prompt or inline SUI notification) or at runtime
(`agent.grant`); the approved set is persisted and minted at dispatch. Authority is
the operator approval → provisioned set → minted token — **not** a compile-time
signature (an agent controls its own skill files, so a signature proves "the daemon
compiled it," not "the human authorized it").
