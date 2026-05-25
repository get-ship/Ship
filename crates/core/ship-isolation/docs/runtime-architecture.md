# ship-isolation — runtime architecture

The shape we're building toward. Local-first. Same supervisor code in
every topology. Container is a tool, not an environment extension.

Pair with:
- `strategy.md` — tier model + which strategies map to which tier
- `sidecar-protocol.md` — the v1 wire contract
- `threat-model.md` — what each tier defends
- `architecture.md` — the current crate layout (what exists today)

This doc describes the *target* shape and the bones we need so we
don't paint ourselves into a corner later.

## 1. Scope

In: the runtime for an agent process — how it's started, supervised,
isolated, and observed across the four tiers we support (none /
process / container / VM). Includes the local docker, local VM, and
local rootless-container paths.

Out (for v1.0):
- Cloud as a product surface. Tier 4 is a seam, not a feature.
- File-sync between host and remote sandboxes. Won't ship.
- Remote-as-extension-of-local semantics. If you want a remote dev
  environment, that's a different product.
- BYOC as the primary path. Documented escape hatch, not the default.

## 2. Mental model

```
   user installs Ship on macOS / Windows / Linux
                 │
                 ▼
              ┌─────────┐
              │  shipd  │   compiler · kernel router · mesh ·
              │ (host)  │   ActorStore · HTTP MCP · UI · API
              └────┬────┘
                   │
       ┌───────────┴───────────┐
       │                       │
   in-process call     ConnectRPC bidi (ship.sidecar.v1)
       │                       │
       ▼                       ▼

  Tier 0/1 sandbox        Tier 2/3 sandbox (pod)

  same supervisor         same supervisor + v1 server
  ──────────────         ────────────────────────────
   actor 1..N             actor 1..N
   harness (PTY)          harness (PTY)
   user stdio MCPs        user stdio MCPs
                          (pod-shared by default)
```

Three cardinalities:

- **Actor** — one per running agent. Harness process, PTY, MCP
  children, hook stream.
- **Sandbox / pod** — 1..N actors. The unit of isolation from the host.
  User-configurable granularity (project default; pipeline / agent
  can override).
- **shipd** — one per host. Orchestrator + UI + storage + canonical
  event log.

## 3. The single supervisor invariant

Ship has one supervisor implementation. It supervises N actors,
manages their PTYs, parses harness streams, fires hook events,
spawns MCP children, applies Tier 1 wrap flags when asked.

Where it runs depends on topology:

- **Tier 0/1:** supervisor is a module inside shipd. Orchestrator
  calls it in-process.
- **Tier 2/3:** supervisor lives inside the sandbox, wrapped with a
  v1 ConnectRPC server. Orchestrator calls it over the wire.

**No shadow code.** The "sidecar" is the supervisor + a v1 wrapper.
The wrapper is the only piece that exists for the cross-boundary
case.

This is the invariant that lets us move adjustments later without a
big refactor. New tiers reuse the supervisor + new outer boundary;
new harness adapters land in one place; new MCP integration lands
in one place.

## 4. Boundary protocol

`ship.sidecar.v1` over ConnectRPC over HTTP. Transport varies by tier:

| Tier | Transport |
|---|---|
| 0/1 | not applicable — in-process call |
| 2 local | unix domain socket OR localhost TCP |
| 3 local | vsock OR localhost TCP |
| 4 remote (future) | TCP + TLS |

The wire shape is fixed; the transport plugs in. Adding cloud later
is a transport addition, not a protocol redesign.

What crosses the boundary:

1. **`ship.sidecar.v1.Processes.*`** — orchestrator → in-sandbox
   supervisor (spawn, kill, attach pty, write input, resize, list,
   get).
2. **`ship.sidecar.v1.Events.Stream`** — bidi envelope. Server side
   carries process output, exits, hook fires, log progress, broker
   replies. Client side carries broker responses (permission,
   secret, setting) and management messages.
3. **HTTP/SSE to shipd `/api/v1/*`** — Ship MCP (HTTP) and hook
   ingest. The in-sandbox actors are clients of the existing daemon
   API exactly as host actors are today.

No third protocol. Nothing new for actors-in-sandbox to invent.

## 5. Sandbox lifecycle

**Default: one pod per project, idle-reaped.**

- Lazy start: first actor or pipeline in a project triggers pod
  creation.
- Stay alive while any actor is running OR for
  `isolation.pod.idle_timeout_minutes` (default ~15) after the last
  actor exits.
- Stop on idle, on explicit stop, on shipd shutdown, on project
  removal.

**Pipeline / agent overrides specify their own pod.**

```yaml
pipelines:
  untrusted-pr-review:
    isolation:
      tier: container
      lifetime: per_run     # opt-in: this pipeline gets its own pod
      strategy: docker
      network: deny          # stronger than project default
```

```
ship use my-agent --isolation=vm   # one-off override
```

**Override is a floor, not a ceiling.** A pipeline or agent can
request *stronger* isolation than the project default but not
weaker. Prevents profile rot from downgrading project security.

**Multiple pods within a project** are supported. An agent's profile
can name its pod explicitly:

```jsonc
{ "name": "pr-reviewer", "pod": "untrusted" }
```

Default pod name is `default`. The project's isolation strategy
applies to its `default` pod; additional pods are configured
explicitly.

## 6. In-sandbox runtime

The same supervisor, packaged for cross-boundary execution. Inside
its sandbox it owns:

- **Process tree supervision** for the actor's full set of children:
  harness binary, user-configured stdio MCP servers, anything the
  agent profile declares.
- **PTY allocation + I/O** via portable_pty (unchanged from today's
  supervisor).
- **Harness adapter logic** — `build_interactive`/`build_headless`,
  per-harness stream parsing for token usage / hook synthesis / cost
  capture. Migrated *into* the runtime from `ship-platform::services::harnesses::*`.
- **Hook event synthesis.** The runtime parses harness streams to
  emit `Envelope::HookFired`. For Claude HTTP hooks the runtime
  exposes a localhost HTTP endpoint inside the pod that mimics
  `/api/v1/hooks/*` and forwards over the v1 stream.
- **MCP child management.** Reads the compiled agent profile's
  `mcp_servers`; spawns each as a child; harness opens stdio pipes
  to them. Pod-shared by default (one filesystem MCP serves all
  actors); per-actor opt-in.
- **The v1 ConnectRPC server** — only when running cross-boundary
  (Tier 2/3). Wrapped around the supervisor's spawn primitive.

What the runtime does NOT contain:

- **A parallel daemon API.** Hooks reach the daemon via the v1
  outbound; MCP traffic reaches the daemon directly over HTTP/SSE
  (existing pattern).
- **A parallel MCP server.** Ship's MCP server is daemon-side at
  `/api/v1/mcp/*` for harnesses that speak HTTP MCP. Stdio shim
  remains as compatibility (see §7).
- **The `ship` CLI binary.** Hook scripts inside the pod, if needed,
  POST directly to the runtime's local hook endpoint. No CLI
  install in the pod's image.

## 7. Agent → daemon: the op surface

**Ship's own surface (skills, not MCP).** A dispatched agent reaches
Ship through one verb — `ship ops call <op>` — taught by generated
capability-skills. The CLI resolves its transport from the env the spawn
path injected (§9): `SHIP_DAEMON_URL` (the host daemon, rewritten
sandbox-reachable) + the scoped `SHIP_TOKEN`. No relays. The in-daemon
MCP tool server is being culled in favor of skills + `ship ops`; the op
surface (`/ship.ops.v1.*`, incl. `isolation.*`) is its replacement.

**User-configured MCP servers (external, unchanged):**

- Listed in the agent profile (`mcp_servers: Vec<McpServerConfig>`).
- Run as children of the in-sandbox runtime; image-baked or
  `npx`/`uvx`-installed at pod start.
- Harness connects via stdio (local pipes inside the pod) — never
  crosses the boundary. Pod-shared by default; per-actor opt-in for
  privacy.

## 8. Filesystem

**Tier 0/1 (host):** the workspace *is* the workspace. Nothing to
do.

**Tier 2/3 (local container / VM):** bind-mount the project's
working directory into the pod at the same path. Live, coherent,
hot-reload-friendly. This is the happy path.

```
container fs:
  /workspace                 ← bind-mount of host project dir, RW
    .ship/                   ← compiled agent configs (writable for outputs)
    .artifacts/              ← persistent outputs
    src/, .git/, etc.        ← everything in the user's project
  /ship-runtime/             ← ephemeral, per-pod scratch
    npx-cache/               ← shared MCP install cache
    uvx-cache/
    logs/                    ← per-actor log files
  /tmp/                      ← pod-ephemeral; agents scribble freely
  /tmp/actor-<id>/           ← per-actor private scratch
```

**Tier 4 (cloud, future):** work-unit model. Pod runs against a
snapshot (git-cloned at pod start, or user-uploaded tarball).
Outputs come back via `Envelope` events + artifact uploads. No
file sync, no live coherence, no pretense that the cloud pod's
view matches the host's. Users wanting hot-reload work locally.

The boundary between the local and cloud models is sharp. We don't
build sync; we don't pretend remote feels local.

## 9. Auth — scoped tokens

The isolation enforcement of multi-tenant scenarios (multiple
projects, multiple pods) is a scoped-token model:

- Daemon mints per-pod tokens with a scope claim:
  `{ project: "my-repo", pod: "default", ttl: 24h }`.
- The pod's in-sandbox runtime gets the token via env at startup.
- Per-actor tokens are derived for actors inside the pod (scoped
  to the actor's events / namespaces).
- Every daemon API call (HTTP) and every v1 stream message
  validates the token's scope against the action.

Token leaks are bounded to one pod, not the whole daemon. This is
the actual security primitive — without it, BYOC and multi-tenant
have no story.

**Two identity legs, split by where the kernel boundary falls:**

- **Host (Tier 0/1):** a local agent shares the daemon's kernel, so
  the daemon reads its identity from the connection's peer credentials
  (`SO_PEERCRED`) over the host unix socket — tokenless. (Host-socket
  work; canonical write-up in the auth-model doc.)
- **Isolated (Tier 2+):** a container/microVM agent's PID is not
  resolvable in the daemon's namespace and there is no shared socket, so
  peer-cred *cannot* apply. The supervisor mints a scoped session token
  (role `worker` — never the daemon/Full token), injects it as
  `SHIP_TOKEN` plus the reachable origin as `SHIP_DAEMON_URL`, and drops
  it from the registry on actor exit (immediate revocation).
  `ship ops` dials that origin with that token. This is the
  load-bearing isolated-agent identity — peer-cred is host-only by
  physics, not by preference.

## 10. Capability matrix + recommender

The current capability probe extends with a **deployment row**:

| | Tier 0 | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
|---|---|---|---|---|---|
| shipd on macOS | ✓ | Seatbelt | docker / podman | lima | (future) |
| shipd on Linux | ✓ | bwrap / srt | docker / youki | lima / WSL2 | (future) |
| shipd on Windows | ✓ | — | podman / WSL2 | WSL2 | (future) |
| shipd in container | ✓ | bwrap (capability dependent) | none (no DnD) | none | (future) |

Recommender returns the achievable tier set for the current
deployment. UI surfaces it honestly — "shipd is itself in a
container; agent isolation tier capped at Tier 1."

## 11. Seams we keep open (so cloud doesn't require a refactor)

These are decisions made now to keep cloud reachable later without
restructuring:

1. **Transport pluggability** — the v1 client/server abstracts
   transport. Adding TCP+TLS for remote is a transport
   implementation, not a protocol redesign.
2. **Pod = sandbox = N actors.** The cardinality is already
   correct for cloud "pods of collaborating agents."
3. **Profile distribution over the wire** (not bind-mount). Daemon
   serializes the compiled agent profile into the v1
   `ProcessCreateRequest` (or a sibling `Pod.LoadProfile` RPC).
   Works for local pods and would work for remote.
4. **Workspace ingest is a separate concern from the runtime
   protocol.** Bind-mount is one ingest strategy; git-snapshot is
   another. The runtime doesn't care.
5. **Scoped tokens** generalize across local + remote — same model,
   different issuance flow.
6. **Event stream is bidi `Envelope`s.** Adding new envelope variants
   (cloud-specific metrics, vendor failures, etc.) is additive.

## 12. Sandbox abstraction (new state in shipd)

The daemon grows a `SandboxRegistry`:

```
struct Pod {
    id: PodId,
    project: ProjectId,
    name: String,                // "default", "untrusted", ...
    isolation: IsolationProfile, // tier + strategy + network + resources
    lifetime: PodLifetime,       // Idle { timeout } | PerRun | Manual
    runtime_endpoint: Option<RuntimeEndpoint>, // None until pod is running
    actors: Vec<ActorId>,
    state: PodState,             // Pending | Running | Idle | Reaping | Stopped
}
```

Orchestrator's spawn-actor path:

```
spawn_actor(actor_profile):
  pod = registry.pod_for(actor_profile)
        // → project's "default" pod, or actor's declared pod
  if pod.state != Running:
    pod.start()   // lazy
  pod.runtime.processes.create(actor_profile.into_request())
```

Pod start:

- Tier 0/1: instantiate supervisor in-process; register endpoint =
  in-process handle.
- Tier 2/3: prepare the underlying sandbox (Docker / youki / lima),
  set runtime endpoint to its v1 listener, open the bidi event
  stream.

Pod stop (reap):

- Drain in-flight actors (graceful) or hard-kill (timeout).
- Tear down the underlying sandbox.
- Persist final state to platform.db (we already do this for
  individual actors).

## 13. What the substrate refactor looks like

Smaller than my earlier estimates because supervisor stays one piece:

1. **Move harness adapters into the supervisor crate** (out of
   `ship-platform::services::harnesses`). Trait + per-harness modules.
   Today's stream parsing moves with them.
2. **Add the v1 server wrapper** around the supervisor — only
   compiled in when the runtime is built as the sandbox-side
   binary. Single new crate or feature gate.
3. **Pod abstraction in shipd** — `SandboxRegistry`, lifecycle
   management, per-pod runtime endpoints. Replaces the current
   per-actor `SandboxOwner` model.
4. **Compiled profile distribution** via v1 RPC — replaces any
   bind-mount race for `.ship/agents/<id>/`.
5. **Scoped token issuance + validation** in shipd's auth layer.
6. **Daemon MCP HTTP endpoint** at `/api/v1/mcp/*`. `ship mcp serve`
   becomes a stdio compatibility shim that calls the new endpoint.
7. **Wizard UI** (`<sui-isolation-wizard>`) plus the install-helper
   for youki / docker prep per OS.

Each is bounded. The substrate's `Transport` polymorphism survives;
`PtyTransport` and `HeadlessTransport` keep working as supervisor
internals. The `Transport` abstraction we collapse is the
**orchestrator's spawn pathway** (one branch: in-process vs v1
client), not the supervisor's internal spawn.

## 14. Out of scope (preserves the happy path)

- File sync, file watch, file mirror across sandbox boundary.
- Cloud pods as a product surface.
- BYOC as the primary path (escape hatch only).
- Cross-pod fs coherence in heterogeneous pod sets.
- PTY session continuity across pod restarts.
- Sub-file workspace filtering (whole-workspace mount or nothing).
- Auto-installation of the harness CLIs (image bakes them; user
  installs them if BYOC).
- Vendor-specific cloud sandbox APIs (Fly Machines, etc.) — schema
  exists, no backend.

## 15. Open questions to resolve before code lands

- **Default `idle_timeout_minutes`** for project pods (~15? user
  setting?).
- **Wizard cadence** — hard ask at first launch or settings opt-in.
- **MCP server sharing default within a pod** — shared by default
  with per-agent opt-out? Or per-actor by default?
- **`ship mcp serve` deprecation** — breaking change to compiled
  configs. Migration path?
- **Pod state persistence** — written to platform.db so shipd can
  reconcile on restart? Or ephemeral?

These don't block design; they're product decisions to make as the
shape lands.
