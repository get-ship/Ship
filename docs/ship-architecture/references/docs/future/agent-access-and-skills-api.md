---
group: Architecture
title: Connect-All-In — One Op Registry, One Gate, Skills as Capability Clients
description: Unify internal transport on ConnectRPC projected from one op registry; bring commands into the registry; classify ops by blast radius and gate the dangerous class; generate capability-skills from the registry. Multi-session tracking doc.
audience: internal
status: proposal
---

# Connect-all-in

Status: **proposal**, tracked across sessions. Supersedes the earlier
macaroon-centric draft of this doc (macaroons are out — see Rejected). Extends
[Daemon authentication model](../auth-model.md) and the sidecar RFC
(`crates/core/ship-isolation/docs/sidecar-protocol.md`).

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

The win is a codegen **collapse**, and it only pays because Connect already owns two
planes (gateway + sidecar). Today the daemon runs a *second* codegen world:
`ship_ops!`/axum (Rust) → **specta** → TS, plus a bespoke OpenAPI + fixtures gen.
Meanwhile gateway + sidecar run **buf-from-proto**. Connect-all-in folds the daemon
into that one toolchain:

- **proto is the single source for the contract _and_ the policy.** Method shapes
  live in proto; capability / blast-radius / `sensitive` ride as **custom method
  options** — co-located with the method, generated into both the gate and the docs.
  buf generates Rust server+client, the TS client, OpenAPI, and fixtures — replacing
  specta and the bespoke gens (deleted).
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
  `apps/ship-cli` + hooks, `packages/sdk` (new, untracked).

None break under additive coexistence; only the CLI is force-migrated first, by
choice.

## Deferred decisions (not now, but named)

- **Streaming adaptor** (SSE → Connect server-streams) + GUI move. Dovetails Tauri.
- **Elevated surface** (where the C2 capability lives). Strongest pro-Tauri
  argument: in Tauri the C2 capability can live in the Rust core, never in the
  webview JS heap, so SUI XSS can't exfiltrate it — a browser can't separate that.
  Gated on (a) C2 ops existing locally and (b) the GUI being their home; the audit
  sizes it. Likely too much to take on alongside Connect now.
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

1. **Op-classification + blast-radius audit** — enumerate the legacy mutation
   routes (the registry is read-only today), classify C0/C1/C2, define write perms,
   identify the C2 set. Feeds Decisions 2–3.
2. **proto-from-`ship_ops!` decision** + conformance-test shape (Decision 2).
3. **Connect op service, additive** + migrate CLI + scoped `--help` (Decision 1).
4. **Generate capability-skills** from the registry (Decision 4).
5. **GUI streaming → Connect** + the elevated-surface/Tauri decision (deferred).
