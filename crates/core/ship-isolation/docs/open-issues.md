# ship-isolation — open issues

Tracked as we work. Each item has: what's open, where it surfaces,
default we're taking, what would invalidate the default. Append-only.

Vocabulary: "sandbox" externally; internal struct `Sandbox` (was `Pod`
in earlier drafts — k8s baggage avoided).

## Backlog — ordered (10 items)

Roughly dependency-ordered. The token foundation (1) underpins MCP (2),
hooks (3), and credentials (4) — those three are one transport problem:
an in-container process reaching the daemon's HTTP API under a scoped
identity. Items 5–6 are the proof-and-primary-path work; 7–10 harden
and broaden.

1. **Scoped tokens — the foundation.** *Decision locked:* per-process
   token minting — supervisor mints a scoped session token per actor
   and pushes it via `Processes.Create.env`, NOT container env.
   Push-not-pull identity; root `[Full]` token never enters a
   container; intra-pod is a trust boundary (pod, not actor, is the
   unit). Build: programmatic `mint_session(agent_id, workspace_id,
   scopes, ttl)`; per-project enforcement teeth (token bound to
   project X hard-rejected for project-Y resources).

2. **MCP transport for containers.** *Decision locked:* one shared
   HTTP MCP server, per-connection actor identity via the scoped
   bearer token. `build_mcp_app`/`run_http_server` already exist
   (same `ShipServer`/tools as stdio — confirmed not a vestige);
   `enforce_mcp_identity` gates per-actor. Build: make it reachable
   from sandboxes; compiler points harness MCP config at the daemon
   MCP URL (`ship_server_entry_http`) for sandboxed actors. Keep it a
   thin module — do NOT absorb MCP into the daemon.

3. **Hooks transport for containers.** Hooks are transport-agnostic
   normalization → `/api/v1/hooks/{session-start,before-tool,
   after-tool,session-end}` taking `ShipHookEvent`; `ship hook X` is
   just an HTTP POST. Build: compile cURL-style hook scripts that POST
   to `$SHIP_DAEMON_URL/api/v1/hooks/X` with the injected token — no
   `ship` CLI required in the container. (Server-mode harnesses
   opencode/codex also feed hooks via stream-synthesis through the
   broadcast the sidecar already pipes — ONE of four transports, not
   the general mechanism.)

4. **Credentials broker.** `Envelope::SecretRequest`/`SecretReply`
   defined in the proto; no handler. Today: env vars + `EphemeralFile`
   bind-mount (recoverable from a process dump). Target: secrets never
   touch the container fs — sidecar asks the daemon at runtime, daemon
   answers from its keychain bridge, secret stays in-memory in the
   harness. T-CRED in the threat model.

5. **Container vertical proof.** Install rootless podman; static-musl
   sidecar build (erases the glibc-ABI / bind-mount caveat — wire is
   the contract, not the binary format); `podman_sidecar_live` e2e:
   prepare → v1 health → spawn actor → output over AttachPty → reap.
   First proof an agent actually runs inside a real container.

6. **VM substrate — the PRIMARY Tier 2 path.** Intent captured in
   `vm-substrate.md`; buildable spec refines from there. youki + a
   Ship-managed Linux VM = zero-install isolation for non-technical
   users. Build: VM-provider abstraction (Lima / Apple-Virtualization
   on mac, WSL2 / Hyper-V on win), VM lifecycle, Linux VM image
   (youki + sidecar), vsock / forwarded-port transport
   (`RuntimeEndpoint::Remote` already accommodates), detect-or-bundle
   recommender (detect docker/podman → adaptor; else bundled youki+VM).

7. **agent-runner image (deliverable).** `ghcr.io/ship/agent-runner`:
   baked sidecar + harness CLIs + common stdio MCP servers. Build /
   sign / version-pin / publish pipeline.

8. **Crash-recovery reconcile.** Persist sandbox state to
   `platform.db` (id, runtime endpoint, state); on shipd restart,
   enumerate running containers, match by name, reconnect v1
   endpoints, reattach observers. (Graceful-shutdown reap is done.)

9. **Hardening.** PTY backpressure (bounded-broadcast drop policy /
   per-subscriber lag), v1 reconnect-on-drop, capability-skew
   detection via `Handshake.Hello.protocol_version`, image-staleness
   surfacing.

10. **Broaden — backends, wizard, CI.** WSL2 / LimaVm / FlyMachines
    impls (declared, no code; cloud = Tier 4, protocol-ready);
    `<sui-isolation-wizard>` + per-OS install helper for non-coders;
    Tier 1 (harness × OS) enforcement matrix in CI. Folds in the
    deferred A1 (harness-adapter relocation into the unified runtime).

## Decided defaults (revisit only on real signal)

### Override semantics — stronger isolation than the project default

When a pipeline or agent declares stronger isolation than the project's
default sandbox provides, **spawn a dedicated sandbox for that
pipeline/agent run.** PodLifetime::PerRun. Lives for the duration of
the requested work; reaped on exit.

Invalidated if: orphaned dedicated sandboxes pile up faster than they
reap, or if user expectations turn out to be "extend the project
sandbox to the stronger tier" rather than "create a sibling."

### Compiled profile distribution across the boundary

`CompiledHarnessBundle { files: Vec<(PathBuf, Bytes)>, metadata }`
travels over the v1 wire in `Sandbox.LoadProfile` (sibling RPC to
`Processes.Create`). In-sandbox runtime materializes files at the
expected harness paths.

Invalidated if: harness configs are too large for a single RPC payload
(unlikely; they're a few KB each), or if some harness expects live
config-file updates during runtime.

### Supervisor code structure

Same crate, compiled into two binaries: shipd embeds the supervisor
module via library dep; `bin/ship-actor-runtime` is the standalone
binary that wraps the same module in v1 ConnectRPC server. The v1
wrapper is the only target-specific code.

Invalidated if: the supervisor's host-only dependencies (UI, DB) leak
into shared code we can't compile cleanly for both targets.

### Harness MCP / hook auth

Per-actor token, derived from the sandbox token, scoped to that
actor's namespace. Harness reads it from env (`SHIP_TOKEN`). Same
credential for MCP HTTP calls and hook events. Daemon validates
scope per request.

Invalidated if: per-tool permission gating needs finer credentials
than per-actor (probably needs per-call token rotation).

### Local token model: stateful `ship_sess_*`, not macaroons (M1)

The actor↔daemon token is a stateful `ship_sess_*` session, minted
per-process host-side and pushed via `Processes.Create.env`
(`SHIP_TOKEN`); the daemon validates by registry lookup and revokes by
dropping from the registry. **This matches the Ship team's macaroon
spec, which explicitly scopes macaroons to delegation *edges* and keeps
`ship_sess_*` for local agent↔daemon** (see `Ship/.artifacts/specs/
cloud-specs/03-capabilities-macaroons.md` — "MCP agent talking to local
daemon… existing `ship_sess_*` works fine… not in M1", and open
question #4: defer the migration).

Why stateful locally: issuer == verifier == the always-present daemon,
so the round-trip is architectural (the actor is an HTTP client of the
daemon regardless of token format) and statefulness is free — instant
revoke, live scope control, no key management. Macaroons earn their
complexity only when the verifier isn't the daemon (cloud/peer), where
offline signature verification avoids a round-trip to an absent issuer.

The seam that keeps the cloud path open: the actor only ever holds an
**opaque bearer** and calls a **configurable `$SHIP_DAEMON_URL`**.
Behind that, validation can become macaroon-aware and the URL can point
at a Ship Cloud relay — without touching the actor or the wire.
Tier-4 sandbox-on-cloud tokens become per-job macaroons (consuming the
planned `crates/core/macaroon`) per cloud-spec 08; **not built now,
purely additive.**

Implemented (A1): `mint_session`/`drop_session` + a process-global
`session_registry()` accessor (mirrors `dispatcher()`), shipd installs
it at boot, the supervisor mints in the docker branch and drops on
actor exit. Per-project enforcement teeth: `mcp_workspace_violation` /
`enforce_mcp_workspace`, wired into `dispatch_agent_session`. NOT yet
done: workspace caveat on sandbox-minted tokens (supervisor lacks the
daemon workspace id), `$SHIP_DAEMON_URL` injection + MCP config (A3),
hook scripts (A4).

Invalidated if: the local round-trip per call becomes a measured
bottleneck (then revisit offline verification locally too).

### Default `idle_timeout_minutes` for project sandboxes

15 minutes. Long enough to cover think-time between agent runs; short
enough to release resources in everyday use.

Invalidated if: users complain about cold-start tax in their actual
working rhythm. Knob is `isolation.sandbox.idle_timeout_minutes`.

### MCP server sharing within a sandbox

Pod-shared by default for stateless servers; per-actor for stateful.
Determined by a field on `McpServerConfig` (defaults to shared, agent
profile can opt out per server).

Invalidated if: stateless-vs-stateful distinction turns out to be too
fuzzy in the wild and we get cross-actor state leaks.

### CLI in container image

Bundled in `ghcr.io/ship/agent-runner`. Hook scripts use `ship hook X`
unchanged. BYOC users opt out by not installing it; their hooks POST
directly.

Invalidated if: image size becomes a real constraint (CLI is ~15MB).

## Shipped since last update

- ✅ ship-sidecar-proto crate extracted (substrate can depend on
  the wire types without cycle)
- ✅ SandboxRegistry + state machine (14 tests)
- ✅ ActorRuntime trait + InProcessActorRuntime
- ✅ ChildHandle factored behind ChildBackend trait
- ✅ SidecarChildBackend + SidecarActorRuntime (live tested)
- ✅ SandboxDispatcher (10 tests including singleton contract)
- ✅ ProcessExecutor wires through dispatcher (cutover)
- ✅ Background sandbox-reaper task
- ✅ Per-sandbox SandboxOwner — reap fires backend shutdown
- ✅ DockerEngine prep migrated from per-actor to per-sandbox
- ✅ GET /api/v1/sandboxes + ship isolation sandboxes CLI

## Open — not blocking but needs an answer before each chunk lands

### Hook synthesis vs hook scripts (Chunk C)

Can in-sandbox runtime synthesize `PreToolUse` from harness stream
parse alone? Need to verify per harness before declaring
`ship hook` deprecated for in-container. Stream-parse works for
analytics (PostToolUse-style data); PreToolUse before-arg-execution
may need a different path.

### Per-pipeline-step sandbox reaping (Chunk B)

`PodLifetime::PerRun` on a pipeline declaration applies to which scope
— the whole pipeline run or per-step? Default-to-pipeline-run; revisit
when a real pipeline asks for per-step isolation.

### Tier 1 wrap availability inside Tier 2 (Chunk C)

bubblewrap inside Docker requires `CAP_SYS_ADMIN`. Need capability
probe inside the in-sandbox runtime that reports which Tier 1 wraps
work in its environment. Recommender accounts for nested constraints.

### Pod state persistence + reconcile on shipd restart (Chunk F)

Sandbox state written to `platform.db` (sandbox id, runtime endpoint,
state). On shipd boot: enumerate running containers, match by name,
reconnect to v1 endpoints, reattach observers. Design TBD before
Chunk F.

## Known gaps — long-tail, address as encountered

- **PTY backpressure / lag policy.** Bounded broadcast channel drops
  on overflow. Slow consumers lose bytes. Per-subscriber buffer with
  explicit lag policy needed eventually.
- **Image staleness detection.** Use `Handshake.Hello.protocol_version`
  to detect capability skew and surface to user. Trivial when we get
  to it.
- **Multiple shipd instances on the same docker.** Container names use
  slug+nanoid so global uniqueness is fine; resource limits not
  cross-aware. Document, don't solve.
- **Long-running coordinator agent prevents idle reap.** Sandbox stays
  alive while any actor runs. Correct behavior; document that
  always-on agents disable idle reap for their sandbox.
- **Workspace concurrent write across sandboxes in same project.**
  Multiple sandboxes bind-mount same workspace; concurrent writes can
  clobber. Same as today's multi-agent-in-host problem amplified.
  Per-sandbox overlay-fs is the eventual fix; defer until signal.

## Future bigger questions (don't think about now)

- **Cloud (Tier 4) as a product surface.** Architecture preserves
  reachability; product surface explicitly out of scope for v1.0.
- **BYOC as primary path.** Documented escape hatch only.
- **Cross-pod-fs coherence.** Won't ship; users pick local or remote,
  not both transparently.
