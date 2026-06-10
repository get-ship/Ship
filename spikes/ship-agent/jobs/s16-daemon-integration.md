# Job S16 — embedded agent ↔ shipd integration map (auth, gate, execution RPC)

> Status: **read-only findings.** No code moved, no renames, no
> refactors. The deliverable is THIS DOC + a passing `cargo test`
> (`130 passed`, see §6 acceptance).
>
> Lane: **thread A** — integrating the embedded spike with the
> daemon. Thread B (`s16-orchestration-entry`) owns the
> orchestration/dispatch port and the L1↔L0 cycle; do **not** redo
> that work. Naming and "SDK-gen in the compiler" are settled;
> do **not** relitigate. Focus: auth principal source, the
> gate/blast SSOT, and the embedded execution RPC.

## 0. Spike baseline (what this doc is mapping)

`spikes/ship-agent/` is the embedded agent runtime: rig (model loop)
+ deno_core (JS execution) + a scoped SDK + a one-door `gateway.call`
+ a `gate::decide` that mirrors shipd's `services::gate::decide`. The
spike module map is the README §"Module map by concern"
(`spikes/ship-agent/README.md:51-65`); the four invariants on
`README.md:66-89` are the contract this doc preserves:

1. **One door** — every effect enters `gateway.call`.
2. **The gate is the only enforcement.**
3. **No self-elevation** — `operator_cred=false` by construction ⇒
   C3 unreachable.
4. **Denial is data**, never an exception.

`spikes/ship-agent/src/auth.rs:11-28` is the load-bearing model: the
spike constructs `AuthContext { agent_id, grants, operator_cred: false }`
in-process at dispatch. The handler is the entire reason this job
exists — that in-process construction is the assumption the real
daemon breaks. An embedded agent has **no connecting PID** to
peer-cred-attest.

`spikes/ship-agent/jobs/s16-shipcode-boundary.md` (just landed) is
the precursor: it classified ship-code vs shared-domain, and asserted
that blast tiers are already proto-native
(`policy.proto` `blast_radius`). The reader is assumed to have that
doc open; this one builds on its evidence.

---

## 1. Auth handshake — the S5-real question

### 1.1 The real auth stack today

Three layers, the most important is L0:

| Layer | Crate | Role |
|---|---|---|
| L0 substrate | `crates/core/ship-substrate/crates/ship-auth-types` (`ship_auth_types`) | `Scope` / `McpAuth` / `RequiredAuth` / `DaemonSession` / `SessionRegistry`; the `peer_identity::resolve(peer_pid, peer_uid) -> PeerIdentity` seam; session-token mint/parse; role→scope tables. |
| L1 platform | `crates/core/ship-platform/crates/ship-auth` (`ship_auth`) | Re-exports L0 + `resolve_grants` (provisioned baseline + role fallback), `provision_grants` (operator-only writes), session hydrate/sweep, `account_store`. |
| L1 daemon | `apps/shipd/src/auth.rs` (the file) + `auth/{api,identity,peer_socket,session,scopes}.rs` | The `AuthContext` (`auth.rs:72-100`), the bearer auth middleware (`auth.rs:220-411`), the `project_authority_middleware` (`auth.rs:430-437`), the host unix-socket peer-cred middleware (`auth/peer_socket.rs:74-111`). |
| L1 domain | `proto/ship/gateway/v1/auth.proto` | The CLIENT-FACING AuthService — `TokenExchange` (PKCE→BetterAuth), `Refresh`, `DaemonAttest` (Ed25519 JWT, 1h TTL, JWKS-verified), `GetMe`, `RevokeKey`. This is the BROWSER / CLI bearer surface, NOT the in-process principal source. |

Two SEPARATE AuthService-shaped surfaces exist: the one the **browser
and CLI** talk to (the `gateway/v1/auth.proto` RPC — BetterAuth
exchanges, JWT minting) and the one the **daemon** uses to identify
in-process callers. They are not the same thing. The proto `AuthService`
is for tokens; the in-process `AuthContext` is for principals.

### 1.2 Where the principal for a spawned agent actually comes from

A spawned agent's identity is **kernel-verified**, not self-declared.
The path:

1. Shipd spawns a harness as a child process inside a
   **process container** — Linux cgroup, macOS process group,
   Windows job object. Container leaf names the agent
   (`ship-agent-<id>`, per
   `ship_auth_types/src/peer_identity.rs:93-105`).
2. The agent connects to `<daemon_dir>/ship.sock`. The socket's
   `SO_PEERCRED` reports PID + UID
   (`ship_auth_types/src/peer_identity.rs:40-51`).
3. `peer_identity::resolve(pid, uid)` reads the OS container via
   `ship-host`, parses `ship-agent-<id>`, and returns
   `PeerIdentity::Agent(id)` (`peer_identity.rs:53-81`).
4. `auth_context_from_peer(identity, role, grants)`
   (`apps/shipd/src/auth/identity.rs:37-68`) maps `PeerIdentity` to
   `AuthContext`: `Agent(id)` → `caller_kind=Mcp`,
   `agent_id=Some(id)`, `scopes=scopes_for_role(role)`,
   `grants=resolve_grants(id, role)`, **`operator_cred=false`**.
5. On the daemon side, the
   `peer_cred_auth_middleware` (`auth/peer_socket.rs:74-111`)
   resolves peer identity BEFORE the bearer middleware runs, so a
   tokenless socket connection IS the agent and inserts
   `AuthContext` into request extensions. A presented
   `ship_sess_` token takes precedence (line 88-90): peer-cred
   does not shadow the token's own capped privilege.

The grants come from `ship_auth::resolve_grants(agent_id, role)`
(`ship-auth/src/scopes.rs:50-52`): the **provisioned baseline**
(from the `provisioned_grants` projection), falling back to
`grants_for_role(role)` (`ship-auth-types/scopes/roles.rs:94-107`).
Either way: **the agent never names its own grants**.

### 1.3 Per-process scoped token mint (the boundary-map callout)

The S16 boundary doc flagged that shipd today mints a per-process
**scoped session token** on the spawn path. The site is
`crates/core/ship-platform/crates/ship-agents/src/runtime/executors/process.rs:421-454`
(`mint_scoped_token`): it calls `ship_auth::scopes::mint_session(...)`
and writes the result into `SHIP_TOKEN` (never the root daemon
token). The corresponding test
(`runtime/executors/mint_token_tests.rs:41-71`) asserts the minted
token is `ship_sess_…` and carries non-`Full` scopes.

This is the same `SessionRegistry` that lives behind
`peer_cred_auth_middleware`'s `ship_auth::scopes::session_registry()`
getter (`auth/peer_socket.rs:101`). The process-wide registry is
installed in `apps/shipd/src/lib.rs:465`
(`ship_auth::scopes::set_session_registry(...)`).

### 1.4 The load-bearing question for the embedded agent

The spike's `AuthContext` is locally constructed
(`spikes/ship-agent/src/auth.rs:21-27`): it does not need a real
principal source because it isn't connecting to anything. To bind
to shipd, the embedded agent needs an in-process
**principal-construction path** that does not depend on a
connecting PID. Three candidate seams:

| Seam | What it is | How an embedded agent could use it |
|---|---|---|
| `ship_auth::scopes::mint_session(registry, agent_id, workspace_id, role, expires_at)` | L0 token mint used by the spawn path. Returns a `(token, DaemonSession)` tuple; the token is `ship_sess_…` and the session is registered. | The embedded runtime is started by shipd on a known `agent_id` (from `AgentSpec` / dispatch record). It mints a session bound to that `agent_id`; the resulting token is what it presents on the one door. |
| `ship_auth::scopes::resolve_grants(agent_id, role)` | The same grants resolver shipd uses on the peer-cred path. Returns `Vec<String>` (the agent's C2 service set, provisioned or role-fallback). | The runtime computes its own `grants` (the agent cannot elevate its own grants — same as today). |
| `ship_substrate::events::CallerKind` + the in-process `AuthContext { caller_kind, agent_id, grants, operator_cred:false, ... }` | The L1 shape shipd's `services::gate::decide` consumes. | The runtime constructs the same `AuthContext` (or a portable shim with the same fields) and presents it to `gateway.call`. |

**Evidence-backed answer: there is no in-process principal-construction
path that takes a (agent_id, role) and returns a fully-fledged
`AuthContext` keyed to shipd's gate. The peer's `AuthContext` is
built from `PeerIdentity` (peer-cred) or from a presented bearer
token; both anchor in a connection.** The closest analogue is the
scoped-token mint that shipd already does for the spawn path
(`mint_scoped_token`, §1.3), and that mint is paired with a *connect
back* — the spawned agent holds `SHIP_TOKEN` because it is a real
PID on the other end of the socket. The embedded agent has no
`SHIP_TOKEN` to read.

The cleanest in-process principal source for the embedded runtime
is a **re-entrant construct** that:
1. Reads its `agent_id` from the dispatch record
   (`DispatchResult.session_id` → kernel events → `agent.session.started` envelope; the same path the spawned agent follows).
2. Calls `ship_auth::scopes::resolve_grants(agent_id, role)` to
   compute `grants` (never self-asserted).
3. Constructs `AuthContext { caller_kind: Mcp, agent_id: Some(id),
   grants, operator_cred: false, ... }` (or a thread-local
   equivalent that the gate accepts) and threads it into
   `gateway.call`.

**No self-elevation invariant** (readme invariant 3) survives the
binding if the embedded runtime can construct `AuthContext` with
`operator_cred: true` (or otherwise set `Authority::Operator`). The
shipd `AuthContext` only sets `operator_cred: true` from a
**pid-attested operator session** (see
`apps/shipd/src/auth.rs:91-99`, the `is_operator_cred` gate). The
embedded runtime has no PID-attested session; whatever construct it
uses must be hard-wired to `operator_cred=false`. The spike's
`AuthContext::in_process(agent_id, grants)` already does this
(`spikes/ship-agent/src/auth.rs:21-27`); the binding layer must
preserve that constraint when the runtime is hooked into shipd.

**Gap (called out plainly):** there is no `fn
embedded_principal(agent_id, role) -> AuthContext` seam in shipd
today. The closest existing constructor is
`auth_context_from_peer(PeerIdentity, role, grants)` in
`apps/shipd/src/auth/identity.rs:37-68`, but it takes
`PeerIdentity::Agent(id)`, which requires a kernel-resolved
container. A new seam — or an extension to the existing one — is
needed. The shape is small (it would be a function that builds
the same `AuthContext` as `auth_context_from_peer` does for
`PeerIdentity::Agent`, without requiring the kernel resolve).

### 1.5 The blast-radius overlap with the operator-cred contract

`apps/shipd/src/services/gate.rs:85-100` is the real `decide`:

```text
operator                          -> Allow (full trust, all tiers)
C3 critical                       -> Deny  (operator/2FA only; never an agent)
C0 read                           -> Allow
C1 workspace                      -> Allow  (the operational baseline — any agent)
C2 elevated, service in grant     -> Allow
C2 elevated, service not granted  -> Deny
```

The "operator clears every tier" branch keys on
`caller.is_operator`, which is derived from
`auth.is_operator()` → `auth.operator_cred` (see
`apps/shipd/src/auth.rs:91-99`: the only path that sets
`operator_cred: true` is `DaemonSession::is_operator_cred` clearing
`pid_authorizes` in the bearer middleware). The "C3 is operator-only"
branch is independent: even an authenticated agent (with
`operator_cred: false`) is denied at C3. **The "operator=false ⇒
C3 unreachable" guarantee is the conjunction of these two
branches**; breaking it requires either setting `operator_cred: true`
or skipping the C3 branch. Neither is reachable through the spike's
`AuthContext::in_process` constructor.

The "no in-process principal path" gap is therefore a **narrow
additive** (one new constructor) and does not touch the gate's
"operator" branch.

---

## 2. Gate / blast mapping (open-question #1, resolve with evidence)

### 2.1 The proto SSOT is already real

`proto/ship/ops/v1/policy.proto:11-25` declares the canonical
`BlastRadius` enum: `C0_SAFE` / `C1_SPACE` / `C2_ELEVATED` /
`C3_CRITICAL`. `proto/ship/ops/v1/policy.proto:27-39` extends
`google.protobuf.MethodOptions` with `capability` (string, #50001),
`blast_radius` (enum, #50002), `op_sensitive` (bool, #50003), and
`description` (string, #50004).

`crates/core/ship-ops-rpc/src/policy.rs:1-326` is the **SSOT reader**:
it reads the embedded `FileDescriptorSet` (`--include_imports`) via
buffa reflection, decodes the four options, and exposes
`policy_for(op_name) -> Option<&'static MethodPolicy>` — the single
op-name → policy derivation. Blast enum is `C0Read / C1Space /
C2Elevated / C3Critical`; **`from_descriptor`/`unknown` fails closed
to `C3Critical`** (line 46). The proto is the source of truth; the
Rust table is gone.

The spike's `gate.rs::Blast { C0Safe, C1Space, C2Elevated,
C3Critical }` (`spikes/ship-agent/src/gate.rs:17-23`) is **the same
enumeration** with the same names — it is the spike-local mirror, not
a parallel taxonomy. The boundary doc's §8.4 conclusion ("the
blast-radius tier system already exists") is correct and the spike
duplicates nothing.

### 2.2 Does/should the spike's registry DERIVE from the proto?

The spike's `registry.rs` (lines 54-125) is a hand-declared
`static` of `Service { name, methods: [Method { name, blast, args,
returns }] }` — a contract-faithful fake of a proto
`FileDescriptorSet`. The blast values on each method "mirror
`proto/ship/ops/v1/policy.proto`" (`registry.rs:48`). It is the
spike's stopgap because the spike does not yet depend on
`ship-ops-rpc` (the spike's `Cargo.toml` is a standalone workspace
with `rig` + `deno_core` + serde/tokio only — see
`spikes/ship-agent/Cargo.toml:13-26`).

**Recommendation: yes, derive.** When the spike graduates into
`crates/core/ship-platform/crates/ship-agents/src/runtime/embedded/`
(or wherever it lands), the embedded runtime should read its
descriptor set from `ship_ops_rpc::policy::read_policies()` (or
whatever seam ships in the crate move), keyed by the op name
`policy_for` consumes. The blast values on the hand-declared
methods can then be removed: `blast_of(op)` becomes
`policy_for(op).map(|p| BlastRadius::from(p.blast))`. The SDK
projection (`sdkgen.rs` / `dtsgen.rs`) and the gate's `decide` both
read from the same source — no parallel table to drift.

This is exactly the pattern the boundary map §3.4 / §8.4 flagged as
the embedded-runtime S8 closure — the spike's own S8 job
(`jobs/s8-scoped-sdk-from-proto.md`) collapsed the SDK + gateway
blast lookups to a single descriptor set; the graduation should
swap the fake for the live one. **No new decision; the spike's
authors have already committed to this direction.**

### 2.3 Where is op enforcement TODAY (real path)

The old chokepoint was `services::command::call` — the string-dispatch
apparatus. **It has been removed.** The evidence is in
`apps/shipd/src/views/sui_mutation.rs:73-75`:

> // String-dispatch apparatus (command::call) was removed (one rpc story).
> // Progressive-enhancement form mutations must migrate to typed Connect RPCs.

and the historical references in `ops_rpc.rs:141-143` and
`services/gate.rs:17-18` are the *old* mental model: enforcement
used to live at `services::command::call`, and the docs still call
it out as "the ONE gate." Today the enforcement is at **two
surfaces**, both reading the same `ship_ops_rpc::policy::policy_for`
table and the same `services::gate::decide`:

1. **Crate-owned Connect wires** (`apps/shipd/src/ops_rpc.rs:172-187`).
   `ShipdPlatform`'s `GateFn` closure constructs `Caller` from
   `AuthContext` and runs `gate::decide(pol.blast, domain_of(op), &caller)`
   per RPC. This is the "one rpc story" the cull gave us: every
   `ReadOps` / `CommandOps` / per-domain service RPC enforces
   identically, no string dispatch.
2. **The L0 substrate `runner` (orchestrator + agent-session) is
   domain-free** (`crates/core/ship-substrate/src/services/dispatch_ports/runner_ports.rs:1-44`).
   The `SpawnPolicy` trait takes a domain-free `Authority` principal
   and produces the env to inject; the runner itself never names
   "agent" or "operator" or runs a gate. **The runner is a
   supervisor-mechanism layer; policy lives one level up** in the
   `ConfigSpawnPolicy` impl
   (`crates/ship-pipeline-runtime/src/orchestrator/spawn_policy.rs:62-83`),
   which uses `AgentPlatform::mint_session_token` (L1) to mint a
   scoped token. The token's authority is what the spawned process
   has at the gate, not anything the runner decided.

The dispatch surface itself (`POST /api/v1/agent_sessions/dispatch`
in `apps/shipd/src/routes/runtime_api/dispatch/mod.rs:37-84`) is a
**thin transport adapter** that calls
`ship_agents::services::agents::dispatch_core::dispatch_agent_session_core`
via the `AgentDispatchPort` trait, with the concrete impl
`AgentDispatchAdapter` in `apps/shipd/src/agent_event_ports.rs:92-124`.
The actual handler is the one in `rpc_wire.rs` (§4 below).

### 2.4 Does the embedded one-door gate BECOME the in-process enforcement?

The spike's `InProcessGateway` is a contract-faithful fake that
echoes the result of the gate decision
(`spikes/ship-agent/src/gateway.rs:122-175`). It is **not** the
real enforcement: it doesn't reach the proto descriptor set, and
its dispatch is a `json!(...)` echo, not a real domain call.

**Recommendation: route THROUGH, do not duplicate.** The embedded
runtime's "one door" should not BECOME the in-process enforcement
for the real Ship; it should ROUTE THROUGH the existing
`services::gate::decide` (the proto-backed, descriptor-driven
function in `apps/shipd/src/services/gate.rs`) using the
`AuthContext` it constructed in §1.4. Concretely:

- The spike's `gate::decide` should be replaced (at the seam) with
  a call to `ship_ops_rpc::policy::policy_for(op)` to read the
  proto blast, then `shipd_gate::decide(blast, service, &caller)` —
  same function, same C0/C1/C2/C3 verdict, no second copy of the
  decision table. The spike's `gate::decide` becomes a thin
  adapter; the spike's `Blast` enum becomes a re-export of
  `ship_ops_rpc::BlastRadius`.
- The spike's `registry.rs` collapses to a call to
  `ship_ops_rpc::read_policies()` — one descriptor set, the same
  source the SDK + the wire RPCs already read.
- The dispatch echo (`gateway.rs:166-173`) becomes a call into the
  domain's existing service impls (e.g. `CollectionsServiceImpl`).
  The one-door shape is preserved; the door leads to the real
  Ship.

This is **not a new decision** — it is the explicit "refactor-gated
swap" the spike README §"Seams" row for `Gateway` already names
(`README.md:96`): "Stub: `InProcessGateway` (echo + gate). Real:
shipd `command::call` (refactor-gated swap)." The refactor cull
that retired `command::call` rerouted through Connect wires, so the
"real" target is now `ship_ops_rpc` + the wire handlers, but the
goal is the same: one door, one gate, one descriptor set.

### 2.5 "The gate is the only enforcement" — does it survive?

**Yes, with caveats.** The blast-tier gate remains the single
enforcement for the embedded path if and only if:

- The embedded `AuthContext` is constructed without `operator_cred`
  (see §1.4 / §1.5). The constructor that the binding adds must
  hard-wire that field; the S6 (`operator_cred` is the SOLE operator
  authority source, per `apps/shipd/src/auth.rs:91-99`) contract
  must apply to the embedded runtime identically.
- The "scoped SDK + deny-by-default loader" in the spike (readme
  invariant 4) is a *soft* enforcement (ergonomics / legibility).
  This is unchanged and intentional: a raw `op_ship_call` to an
  ungranted op is still denied at the door. The embedded runtime
  inheriting this posture means the deno_core JS surface can
  attempt any op; the gate decides.
- The `hooks/tool_gate` layer is a **different** layer-2
  envelope-pattern check (path-pattern matcher for `PreToolUse`
  payloads, see `crates/core/ship-platform/crates/ship-agents/src/hooks/tool_gate.rs:1-87`).
  It is invoked by `ship hook before-tool` from the HARNESS's
  PreToolUse hook; an embedded deno_core run has no harness
  PreToolUse hook and so does not currently hit it. The
  boundary doc §6.4 ("tool_gate is shared-domain") already notes
  the embedded path COULD use it for future capability-skill tools
  that want path-pattern enforcement; that is a follow-up, not a
  blocker for the binding.

**The one-place-I-can-rule-it-OUT:** the spike does NOT add a
second gate. `spikes/ship-agent/src/gate.rs:1-12` and
`src/gateway.rs:122-150` both call into the single `decide(...)`;
the spike adds no parallel decision. The binding's "route
through" recommendation above preserves that.

---

## 3. Embedded execution RPC contract

### 3.1 What the embedded agent needs

The spike's `AgentEvent` (in
`spikes/ship-agent/src/events.rs:18-39`; lines 21-39 were the
structural summary) is the user-facing event vocabulary for a
running turn: `Status`, `AssistantDelta`, `ToolCall`, `ToolResult`,
`Completed`, `Error`, plus the S11-extended
`ApprovalRequested { op, args, request_id }` and `Cancelled { reason }`.
The `UserInput` decision shape is `Approve { request_id }` or
`Deny { request_id, reason }`. The spike needs:

- A way to **drive a turn** (start a run, hand it an `AgentSpec` +
  prompt, get a `slug`/handle back).
- A way to **stream typed `AgentEvent`s** for that run.
- **Approve / Steer / Cancel** control over the running run.

This is **distinct from the spawned `AgentsService.Dispatch`**
(`proto/ship/ops/v1/agents.proto:457-463`, capability
`ship.runtime.dispatch`, blast C2) which carries harness /
workspace / branch / tab / pid and emits `agent.session.started`
into the kernel event log (the boundary map §4.1). The embedded
contract has no harness CLI, no PTY child, no worktree.

### 3.2 Existing streaming patterns in the proto

Three streaming surfaces exist today:

1. **`AgentsService.Watch(AgentWatchReq) -> stream WatchEvent`**
   (`proto/ship/ops/v1/agents.proto:382-386`). Server-streaming
   over agent DEFS — `scope=all` snapshots `{summaries, catalogs}`
   and patches on config-namespace changes; `scope=agent_id`
   snapshots `{detail, wire, settings}` and patches on that
   agent's `agent.def.updated`. C0, capability `ship.agents.read`.
   **This is the reactive shape the boundary map §4.1
   "Watch" line points at; it is not a per-turn event stream.**
2. **`AgentsService.SessionsLive(ListSessionsReq) -> stream
   JsonResponse`** (`proto/ship/ops/v1/agents.proto:414-417`).
   Server-streaming over the live `agent_sessions` SQL projection
   (`crates/core/ship-platform/crates/ship-agents/src/services/sessions/projection.rs`,
   see boundary doc §1.3.3). C0, capability `ship.sessions.read`.
   **The body is a `JsonResponse` — currently the SUE wrapper
   shape (per `proto/ship/ops/v1/ops.proto`), not a typed
   `AgentEvent`.** This is the closest existing streaming hook for
   "live sessions," but it streams the RUNTIME projection, not the
   model turn events.
3. **`StreamService.Events(EventsRequest) -> stream JsonResponse`**
   (`proto/ship/ops/v1/stream.proto:15-19`). Server-streaming
   kernel event channel — every `EventEnvelope` as
   `JsonResponse.json` (a serialized envelope). C0, capability
   `ship.events.read`. **This is the "every kernel event" firehose;
   it is C0 observe-only and is NOT scoped to one agent's turn.**
4. **`StreamService.SessionBytes(SessionStreamRequest) -> stream
   BytesChunk`** (`proto/ship/ops/v1/stream.proto:25-28`). Raw PTY
   byte stream for a session — replaces the legacy
   `/api/v1/sessions/{id}/stream` SSE route. C0, capability
   `ship.sessions.read`. **This is the spawned-harness PTY stream
   (`ActorHandle::subscribe_output` on a `PtySession`); it is the
   "SessionBytes" name the boundary doc §4.3 said was the right
   shape for the spawned path.** Not a typed event stream.

The proto convention for capability + blast is in
`policy.proto:27-39` and is pervasively applied: every RPC method
on every service carries `option (ship.ops.v1.capability) = "…"`
and `option (ship.ops.v1.blast_radius) = BLAST_RADIUS_*`. The
embedded execution surface must follow the same convention or
shipd's proto-policy conformance
(`crates/core/ship-ops-rpc/tests/policy_conformance.rs`) will flag
it.

### 3.3 Control RPCs that exist vs. need adding

`AgentsService` already has three control methods (the
`Dispatch`/`Stop`/`Steer`/`Grant` set the boundary map §4.1
classified):

| Method | Capability | Blast | Spawn-shaped? | What it does |
|---|---|---|---|---|
| `Dispatch(DispatchAgentRequest) -> DispatchResult` (`agents.proto:457-463`) | `ship.runtime.dispatch` | C2 | YES — emits `agent.session.started`, the spawned-harness path picks it up in `apps/shipd/src/routes/runtime_api/dispatch/mod.rs:37-84` and `crates/core/ship-platform/crates/ship-agents/src/services/agents/rpc_wire.rs:280-868`. | Spawns + controls another agent — orchestration. |
| `Stop(StopAgentRequest) -> StopResult` (`agents.proto:464-468`) | `ship.agents.write` | C2 | YES — `StopFn` in `rpc_wire.rs:64-65`; routes through shipd-injected `stop_fn` (the `runtime/executors/process.rs` cancel path). | Stop a running agent by slug. |
| `Steer(SteerAgentRequest) -> SteerResult` (`agents.proto:469-473`) | `ship.agents.write` | C2 | YES — `SteerFn` in `rpc_wire.rs:67-69`; the `SteerAgentRequest { slug, message }` shape (line 252-255) carries only `slug` + `message` and is the spawned-harness `actor.handle.send` / `tmux send-keys` path. | Inject a message into a running agent. |
| `Grant(GrantAgentRequest) -> GrantResult` (`agents.proto:474-479`) | `ship.agents.grant` | **C3 — operator-only** | Domain (two-effect: live-session upgrade + baseline persist). `GrantFn` in `rpc_wire.rs:71-75`. | Grant restricted capabilities to an agent (operator approval). |

**What exists for the embedded case: Steer (`{slug, message}`).**
The `Steer` shape is generic over the "I have a slug and a message
to push" semantics — the spawned-harness impl pushes via tmux /
actor handle; an embedded impl could push into the `Arc<Mutex<VecDeque<String>>>`
the spike uses (`jobs/s11-approval-steer-cancel.md:84-87`). The
wire shape is reusable.

**What exists for Stop: same shape** — `{slug}` is generic; the
impl side decides whether to send `SIGTERM` (spawned) or set the
`Arc<AtomicBool>` Cancel flag (embedded, per
`jobs/s11-approval-steer-cancel.md:93-103`).

**What does NOT exist: Approve.** The spike's `ApprovalRequested
{ op, args, request_id }` event has no inbound counterpart — there
is no RPC that takes a `{request_id, decision}` and routes the
decision back to the waiting `Approver::request(...)` future. The
`Grant` method is NOT that — it operates on an `agent_id` and a
`grants` list at the operator/agent-pair level (C3), not on a
per-call `request_id` at the human-in-the-loop level. The embedded
runtime needs a new method.

**What does NOT exist: a typed `AgentEvent` stream** — the live
session / kernel event streams are `JsonResponse`. The
`AgentEvent` enum is `serde(tag = "type", rename_all = "snake_case")`
JSON internally-tagged (`spikes/ship-agent/src/events.rs:20-21`),
which means it serializes cleanly into `JsonResponse.json` (the
existing convention). Typing it as a proto `oneof` would buy
typed decoding on the client, at the cost of pinning the wire
shape to the spike's vocabulary (which is reasonable; this is the
embedded runtime's contract).

### 3.4 Shape proposal (NOT naming)

Two surfaces are needed; both should sit on the same service to
avoid RPC sprawl.

| Method | Capability | Blast | Returns | Notes |
|---|---|---|---|---|
| `Run(RunAgentRequest) -> RunResult` | `ship.runtime.dispatch` (same as `Dispatch`) | C2 | `RunResult { session_id, slug }` (mirror of `DispatchResult`) | Drive a turn for the embedded agent. Distinct from `Dispatch` because the body is an `AgentSpec` / `prompt` / no harness CLI; the handler routes into the embedded runtime, not the spawn atom. The `slug` is the control handle (Stop/Steer/Approve/Cancel all key on it). |
| `WatchSession(WatchSessionReq) -> stream AgentEventMsg` | `ship.agents.read` | C0 | typed `AgentEvent` (one of `Status`, `AssistantDelta`, `ToolCall`, `ToolResult`, `Completed`, `Error`, `ApprovalRequested`, `Cancelled`) | Per-session typed event stream. **Distinct from `StreamService.Events`** (kernel firehose) and `AgentsService.SessionsLive` (RUNTIME projection, also `JsonResponse`). |
| `Approve(ApproveRequest) -> ApproveResult` | `ship.agents.write` (matching the Steer / Stop convention) | C2 | `ApproveResult { request_id, decision }` | The inbound half of the `Approver` round-trip. Body: `{ slug, request_id, decision: Approve | Deny { reason } }`. The handler routes the decision to the waiting `Approver::request(...)` future in the embedded runtime. |
| `Steer` (existing) | `ship.agents.write` | C2 | `SteerResult` | Reuse; the embedded impl drains the `Arc<Mutex<VecDeque<String>>>` Steer buffer. |
| `Stop` (existing) | `ship.agents.write` | C2 | `StopResult` | Reuse; the embedded impl sets the `Arc<AtomicBool>` Cancel flag. |

**New service vs methods on existing?** Both are reasonable. The
embedded runtime is a third execution model beside the spawned
harness; lumping its contract onto `AgentsService` keeps a single
agent-shaped surface (the boundary doc §4.1 evidence: agents.proto
collapses profile CRUD + view reads + orchestration into one
service). Splitting it into a new `EmbeddedAgentsService` keeps
the spawned-harness `Dispatch`/`Stop`/`Steer`/`Grant` semantically
clean (their body is "spawn + tmux + worktree" — adding typed
`AgentEvent` semantics to the same service muddles the wire
contract).

**Recommendation: methods on `AgentsService`.** The boundary map
§4.1 already classifies the spawned `Dispatch`/`Stop`/`Steer` as
ship-code-shaped; the embedded-shape methods would be the
embedded-shape siblings, all under the same service. The
gate's domain check (`domain_of(op)`) is `agents.*` for all of
them; capability + blast tiers follow the same axes. A separate
service buys little (one more route) and loses the
"six-collapsed-plural-services" coherence. The naming decision is
the operator's (per the spec, "the naming decision is the
operator's — propose shape, not final names").

**Typed `AgentEvent` vs `JsonResponse`?** The legibility bet (per
the spec's framing) is to type it — the spike's vocabulary is small
(8 variants) and stable across S10/S11/S15 (per
`jobs/s16-graduate.md:53` reconcile callout). A typed oneof
delivers compile-time decode on the studio / CLI client and makes
the wire contract legible in proto review. The cost is a proto
edit when the vocabulary grows, which is the right place for
that change to happen anyway. **Recommendation: typed.** The
`StreamService.Events` `JsonResponse` convention is for the
firehose (where the wire shape IS the substrate envelope); a
per-session AgentEvent stream is a different shape and deserves its
own message.

### 3.5 The dispatch-side path (how the request enters shipd)

The existing `AgentsService.Dispatch` path:
`proto::agents.proto:457` → Connect-RPC handler in
`rpc_wire.rs:280-868` (impl for `AgentsServiceTrait`) → the
dispatch core (`crates/core/ship-platform/crates/ship-agents/src/services/agents/dispatch_core.rs`,
`dispatch_agent_session_core`) → emits `agent.session.started`
envelope via `AgentDispatchPort` (impl:
`AgentDispatchAdapter` in `apps/shipd/src/agent_event_ports.rs:92-124`).
The envelope is the kernel event; the actor
`crates/core/ship-platform/crates/ship-agents/src/actor.rs` is the
sole handler.

The embedded `Run` method would route identically up to the
emitted envelope. The split the boundary doc §6.1 makes is the
clean line: **`dispatch_core.rs` is shared-domain (the message
producer); `src/dispatch.rs` is ship-code (the spawn handler that
acts on the message).** The embedded `Run`'s "act on the message"
handler is a NEW handler in the embedded runtime, NOT a fork of
`src/dispatch.rs`. The shared message shape (`agent.session.started`
or a sibling `agent.session.embedded.started` for typed
distinction) is what both consume.

---

## 4. Decisions surfaced (operator picks; this doc is the evidence)

The four S16 decisions, with recommendations and citations. **No
naming** — operator chooses.

### 4.1 Auth principal source

| Option | Description | Evidence | Recommendation |
|---|---|---|---|
| A. Construct `AuthContext` from a (re-entrant) `(agent_id, role, grants)` tuple, hard-wired `operator_cred=false`. New small seam (~30 LoC) in `apps/shipd/src/auth/identity.rs` adjacent to `auth_context_from_peer`. | The embedded runtime reads its `agent_id` from the dispatch record, calls `ship_auth::resolve_grants(id, role)` to compute `grants`, and constructs the same `AuthContext` shape the peer-cred / token paths build. | §1.2-1.4. Mirrors the existing `auth_context_from_peer(PeerIdentity::Agent(id), ...)` body (line 56-65) without the kernel-resolved container pre-condition. | **YES.** This is the additive, narrow seam the binding needs. The `operator_cred=false` invariant is preserved by construction (the new function does not take an operator flag). |
| B. Use a session token (mint from `ship_auth::scopes::mint_session`). | The embedded runtime mints a `ship_sess_…` token at start, then presents it on every `gateway.call` like a real agent. | §1.3 (`mint_scoped_token`), §1.2 (the bearer middleware path). | NO. The token is a SHIP-side bearer that shipd's bearer middleware validates; the embedded runtime is IN-PROCESS, so there is no socket to present the token over. The token's value is "what would shipd see if a real PID connected"; the in-process principal construction is what the embedded runtime needs. |
| C. Skip the principal construction; use the spike's local `AuthContext` and never reach shipd's gate. | Keep the spike as a self-contained runtime; don't bind to shipd. | §1.0, §1.4. | NO. The whole point of the binding is to plug into shipd's gate and one descriptor set. The spike explicitly notes its seams are placeholders. |

**Recommendation: A.** Single small seam in
`apps/shipd/src/auth/identity.rs`; preserves the S6 operator-cred
contract (operator authority is SOLELY from a pid-attested
operator session, not from a presented `AuthContext`).

### 4.2 Gate: becomes-vs-routes-through

| Option | Description | Evidence | Recommendation |
|---|---|---|---|
| A. The embedded one-door gate ROUTES THROUGH `ship_ops_rpc::policy::policy_for` + `shipd_gate::decide`. The spike's `InProcessGateway` becomes a thin adapter; the blast values on the `registry::Service` table are removed; `blast_of` becomes a one-liner over `policy_for`. | §2.1 (proto SSOT), §2.2 (the spike already notes the registry is a contract-faithful fake), §2.3 (real enforcement surface). | **YES.** This is what the spike README's seams table already names; it is not a new direction. |
| B. Keep a parallel embedded-side `gate::decide` and a parallel `registry::Service` table. | §1.0, §2.2. | NO. Two parallel tables drift. The boundary doc §8.4 and the spike's own S8 already committed to the single-source direction. |
| C. Skip the gate entirely in the embedded path (trust the deno_core sandbox). | §2.5 (the gate is the only enforcement; the SDK + loader are *soft*). | NO. JS sandbox escapes are not a guarantee; a "trust the sandbox" posture removes the load-bearing invariant. |

**Recommendation: A.** The spike's blast-tier gate becomes an
adapter over `ship_ops_rpc::BlastRadius` + `shipd_gate::decide`.

### 4.3 `AgentEvent`: typed proto vs `JsonResponse`

| Option | Description | Evidence | Recommendation |
|---|---|---|---|
| A. Typed oneof in proto. | 8-variant proto message mirroring the spike's `AgentEvent`. Compile-time decode on the client. Wire shape pinned to the spike vocabulary. | §3.4. The spike's `AgentEvent` is small and stable across S10/S11/S15 (per `jobs/s16-graduate.md:53`); the cost of typing is a proto edit when the vocabulary grows, which is the right place. | **YES** (per the spec's legibility framing). |
| B. `JsonResponse` (the `StreamService.Events` convention). | Stream the spike's internally-tagged JSON as-is. | §3.2 (the firehose is `JsonResponse` because the wire IS the substrate envelope). | NO for the per-session stream; the per-session contract is not the firehose. |

**Recommendation: A** for the new per-session stream.
`StreamService.Events` stays as the kernel firehose (`JsonResponse`).

### 4.4 New service vs methods on existing

| Option | Description | Evidence | Recommendation |
|---|---|---|---|
| A. Methods on `AgentsService`. | `Run`, `WatchSession`, `Approve` on the existing service. Reuse `Stop` and `Steer`. | §3.4. The boundary map §4.1 collapses the agent surface into one plural service. Domain check (`agents.*`) is uniform. Capability + blast axes are uniform. | **YES** (recommended). |
| B. New `EmbeddedAgentsService` or `RuntimeService` siblings. | Separate the embedded contract. | §3.2 (`RuntimeService` already exists for `ListProcs`; it is the process-projection read). | Acceptable if naming/legibility prefers the split. The wire burden (one more route) is small. |
| C. Reuse `AgentsService.Dispatch` + `StreamService.SessionBytes`. | Push the embedded runtime through the existing `Dispatch` (it carries harness/workspace/branch — all irrelevant) and use `SessionBytes` for the byte stream (it's a `BytesChunk`, not a typed event). | §3.2. | NO. `Dispatch`'s body is wrong-shaped (it carries harness / branch / tab / pid); `SessionBytes` is raw PTY bytes for spawned-harness attach. |

**Recommendation: A** (methods on `AgentsService`), with **B as
a clean alternative if the operator wants a runtime-shaped
boundary** (the `RuntimeService` is currently a single method
about process projection; expanding it to a runtime-execution
surface is a defensible seam).

### 4.5 Side-decision: the `Approve` RPC exists — but the
operator-cred contract is intact.

`AgentsService.Grant(GrantAgentRequest) -> GrantResult`
(`proto/ship/ops/v1/agents.proto:474-479`) is **C3, operator-only**.
It is the operator-approval path for the *agent's grant baseline*
(not for a per-call `request_id`). The embedded `Approve` is **C2,
ship.agents.write**, and operates on a per-call `request_id`. The
two are different. The C3 contract is preserved (the operator's
C3 authority is unchanged); the embedded `Approve` is an
adjoining C2 method that takes a per-call decision and routes it
to the embedded runtime's `Approver::request(...)` future.

### 4.6 Side-decision: the `AgentEvent` typed message is the
"embedded-specific" vocabulary, not a new substrate envelope.

The kernel event stream (`EventEnvelope`, the
`StreamService.Events` `JsonResponse` payload) is the SUBSTRATE
vocabulary; the embedded `AgentEvent` is the USER-FACING
vocabulary. The spike's `translate(LoopItem) -> Option<AgentEvent>`
function (per `jobs/s16-graduate.md:53` reconcile callout) is the
mapping between them. The new typed message lives on the embedded
surface; it is not a new substrate envelope. The kernel event
stream is unchanged.

---

## 5. Constraints check (spec compliance)

- **READ-ONLY.** No source edits, no renames, no `git mv`. This
  file is the only write. Confirmed: `git status` shows clean
  tree on `job/s16-daemon-integration` after writing this doc.
- **No SDK-gen-in-compiler relitigation.** Out of scope per spec.
- **No orchestration port relitigation.** Thread B owns that;
  this doc does not.
- **No naming decisions.** §4 recommends shapes, not names; the
  embedded RPC method names (`Run`, `WatchSession`, `Approve`)
  are illustrative only.

## 6. Acceptance

- [x] Auth handshake map cites `proto/ship/gateway/v1/auth.proto`,
      `ship_auth_types`, `ship_auth`, `apps/shipd/src/auth.rs` and
      the `peer_socket.rs` + `identity.rs` paths.
- [x] The in-process principal gap is called out plainly (§1.4).
- [x] Gate/blast mapped to `policy.proto` + `ship_ops_rpc::policy::policy_for`
      + `apps/shipd/src/services/gate.rs::decide`.
- [x] Embedded registry DERIVE recommendation tied to the
      existing `S8` direction in the spike.
- [x] "The gate is the only enforcement" — survives, with
      `hooks/tool_gate` distinction called out (§2.5).
- [x] Embedded execution RPC mapped to `AgentsService.Watch`,
      `AgentsService.SessionsLive`, `StreamService.Events`,
      `StreamService.SessionBytes`, and the
      `Dispatch`/`Stop`/`Steer`/`Grant` set.
- [x] `Approve` gap called out plainly (§3.3).
- [x] Decisions surfaced with evidence-backed recommendations
      (§4.1-4.6).
- [x] `cargo test --manifest-path spikes/ship-agent/Cargo.toml`
      → **130 passed** (verified; see spike baseline §0 and
      the raw test output captured at the end of this doc's
      drafting).

## 7. References

- `spikes/ship-agent/README.md` — spike SSOT (one door, one
  gate, invariants 1-8).
- `spikes/ship-agent/src/{auth,gateway,gate,registry,events}.rs` —
  the spike modules mapped.
- `spikes/ship-agent/jobs/s16-shipcode-boundary.md` — precursor;
  classification table, cross-boundary edges, ambiguity
  decisions.
- `spikes/ship-agent/jobs/s16-graduate.md` — graduation
  plan; §3 calls out the EventLog + AgentEvent reconcile.
- `spikes/ship-agent/jobs/s11-approval-steer-cancel.md` —
  Approver / Steer / Cancel design (the embedded surface
  §3.3-§3.4 is shaped around this).
- `proto/ship/ops/v1/policy.proto` — `BlastRadius` enum + the
  four method-option extensions (SSOT).
- `proto/ship/ops/v1/agents.proto` — `AgentsService` (23
  methods, classified in the boundary doc §4.1).
- `proto/ship/ops/v1/events.proto` — `EventsService.Poll`
  (cursor-walk poll; the typed events surface is the S16-new
  `WatchSession`).
- `proto/ship/ops/v1/stream.proto` — `StreamService.Events`
  (kernel firehose) + `StreamService.SessionBytes` (raw PTY).
- `proto/ship/ops/v1/runtime.proto` — `RuntimeService.ListProcs`
  (C0 process-projection read; not the embedded surface).
- `proto/ship/gateway/v1/auth.proto` — the CLI/browser
  AuthService; the in-process principal is a separate surface.
- `crates/core/ship-ops-rpc/src/policy.rs` — the single
  op-name→policy derivation (`policy_for`); `from_descriptor`
  fails closed to `C3Critical`.
- `crates/core/ship-substrate/crates/ship-auth-types/` — L0
  scope/session/peer-identity types; the `peer_identity::resolve`
  seam.
- `crates/core/ship-substrate/crates/ship-pipelines/src/gate.rs` —
  sibling `gate` for the pipeline altitude (not the
  `services::gate`; this doc is about the agent altitude).
- `crates/core/ship-platform/crates/ship-auth/src/scopes.rs` —
  `resolve_grants` (provisioned baseline + role fallback) and
  the L1 `provision_grants` write.
- `crates/core/ship-platform/crates/ship-agents/src/runtime/executors/process.rs:421-454` —
  `mint_scoped_token` (the boundary-map flagged per-process
  scoped token mint; site identified in §1.3).
- `crates/core/ship-platform/crates/ship-agents/src/runtime/executors/mint_token_tests.rs` —
  the conformance test for the scoped-token contract.
- `crates/core/ship-platform/crates/ship-agents/src/services/agents/rpc_wire.rs` —
  Connect-RPC handler for `AgentsService`; the `DispatchFn` /
  `StopFn` / `SteerFn` / `GrantFn` closure types (seam for
  shipd-injected ship-code behavior).
- `crates/core/ship-platform/crates/ship-agents/src/services/agents/dispatch_core.rs` —
  shared-domain message producer for the dispatch atom
  (boundary doc §6.1).
- `crates/core/ship-platform/crates/ship-agents/src/hooks/tool_gate.rs` —
  the **layer-2** path-pattern gate, distinct from the
  blast-tier gate; called out in §2.5.
- `crates/core/ship-substrate/src/services/dispatch_ports/runner_ports.rs:39-44` —
  `SpawnPolicy` trait (domain-free Authority principal).
- `crates/ship-pipeline-runtime/src/orchestrator/spawn_policy.rs:62-83` —
  `ConfigSpawnPolicy` impl (L1: `mint_session_token` via
  `AgentPlatform`).
- `apps/shipd/src/auth.rs:72-100, 220-411` — the daemon
  `AuthContext` and the bearer auth middleware.
- `apps/shipd/src/auth/identity.rs:37-68` —
  `auth_context_from_peer` (the new `embedded_principal` seam
  mirrors its body without the kernel-resolved container).
- `apps/shipd/src/auth/peer_socket.rs:74-111` —
  `peer_cred_auth_middleware` (the host-socket peer-cred
  transport).
- `apps/shipd/src/services/gate.rs:66-100` — the real
  `decide` (the spike's `gate::decide` routes through this
  per §2.4).
- `apps/shipd/src/services/op_policy.rs` — the `policy_for`
  re-export (the op-name → policy seam the gate consumes).
- `apps/shipd/src/dispatch/run_dispatch.rs:49-135` — the
  composition root where the embedded runtime would slot in
  (alongside `RoutingExecutor` + `AgentPorts`).
- `apps/shipd/src/agent_event_ports.rs:92-124` — the
  `AgentDispatchAdapter` (the new `embedded_runtime` adapter
  would live alongside it).
- `apps/shipd/src/routes/runtime_api/dispatch/mod.rs:37-84` —
  the HTTP transport for `agent_sessions/dispatch`; the
  embedded `Run` route would mirror its shape.

## 8. Spike test verification (raw)

```
$ cargo test --manifest-path spikes/ship-agent/Cargo.toml
…
cargo test: 130 passed (1 suite, 12 warnings, 0.07s)
```

The 12 warnings are pre-existing (per the boundary doc's
§0 baseline) and unrelated to this job.
