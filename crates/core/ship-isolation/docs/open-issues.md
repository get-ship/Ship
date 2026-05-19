# ship-isolation — open issues

Tracked as we work. Each item has: what's open, where it surfaces,
default we're taking, what would invalidate the default. Append-only.

Vocabulary: "sandbox" externally; internal struct `Sandbox` (was `Pod`
in earlier drafts — k8s baggage avoided).

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
