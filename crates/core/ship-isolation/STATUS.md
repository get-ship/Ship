# ship-isolation — status

**Phase:** Phase B complete; entering Phase C. Tier 1 honest
(HarnessNative + SrtWrapper, per-(harness × OS) adapter table,
supervisor enforcement). Tier 2 has one strategy lifecycle-wired
end-to-end (DockerEngine via SandboxOwner). Protocol v1 frozen
in protobuf via ConnectRPC, and the in-sandbox **sidecar binary
now speaks it natively** — `bin/ship-sidecar/` rewritten around
`Handshake` + `Processes` + `Events.Stream` ConnectRPC handlers
with 8 end-to-end client tests.

**Host driver cutover complete.** `backends::youki_sidecar`
drives the sidecar through the generated `ProcessesClient` +
`EventsClient`. `sidecar::protocol` (the deprecated JSON
shape) is deleted. The `sidecar-mode` Cargo feature now only
pulls in `reqwest` (for the `/v1/health` probe).

**Next chunk:** wrap `SidecarSandbox` in an `Isolator` impl
plus `prepare/exec/shutdown` so YoukiRootless lifecycle binds
to `ActorHandle.sandbox` the same way DockerEngine does today.

See:
- `docs/strategy.md` — design intent
- `docs/sidecar-protocol.md` — v1 protocol RFC
- `docs/sidecar-parity.md` — per-feature audit
- `proto/ship/sidecar/v1/sidecar.proto` — the canonical schema
- `.artifacts/local-sandbox-spec/settings-and-envs.md` — settings/envs

## Verified working

- **Recommender + capability probe** — pure-Rust pipeline from host
  detection to resolved strategy + tier + reason. 81 unit tests in
  ship-isolation.
- **Tier 1 split** — HarnessNative passes the harness's own sandbox
  flag (claude `--sandbox`, gemini `--yolo`, codex Seatbelt no flag);
  SrtWrapper wraps with `@anthropic-ai/sandbox-runtime` only when the
  harness has no native mechanism. Per-(harness × OS) adapter table
  picks per agent run. No double-wrapping.
- **DockerEngine** — `prepare()` spins a container, `wrap_for_external_spawn`
  rewrites the launch to `docker exec`, `SandboxOwner::Drop` fires
  `shutdown()` on actor exit. Tested live in `tests/docker_lifecycle.rs`
  (auto-skip when no engine).
- **Sidecar v1 protocol** — `.proto` schema with Handshake / Processes
  (Create/Run/List/Get/Stop/Kill/WriteInput/ResizeTerminal/AttachPty) /
  Events.Stream (bidi envelope for stdout/exit/permission/secret/setting/
  hook/log_progress). 7 codegen smoke tests; per-tier transport
  documented (unix socket / vsock / TCP+TLS).
- **Sidecar binary** — `bin/ship-sidecar/` speaks the v1 proto
  natively. Three handler impls (Handshake / Processes / Events)
  over a process registry keyed by string id, with a broadcast
  envelope bus. **PTY support is live**: `ProcessCreate{ pty: true }`
  allocates a portable_pty pair, `WriteInput` writes to the master
  end, `ResizeTerminal` propagates SIGWINCH to the slave, and
  `AttachPty` server-streams `PtyChunk` frames from a per-process
  broadcast channel. `Handshake.Hello` advertises
  `{processes, pty, events.acp}`. 10 end-to-end tests drive the
  generated ConnectRPC clients against the spawned binary.
- **Daemon API** — 6 endpoints under `/api/v1/isolation/*` (status,
  rescan, recommendations, preview, verify, init); 13 integration tests
  cover wire shapes + the init+verify atomic contract.
- **CLI** — `ship isolation status|rescan|verify|recommendations|preview|init`
  with `--json`. Daemon-mediated when shipd is up, falls back in-process.
- **MCP tool** — `isolation_status` for agent self-introspection.
- **Agent profile** — optional `isolation:` block on agent jsonc;
  `ship validate` catches malformed blocks at config-load time.
- **Per-strategy settings** — single typed JSON blob
  `isolation.strategies` (docker/youki/harness_native/srt/lima/fly)
  replaces the sprawl of flat per-key knobs. Power users override only
  what they care about; defaults backfill.
- **UI** — three reusable components for any actor/agent view:
  `<sui-isolation-badge>` (compact chip; auto-fetches status),
  `<sui-isolation-detail>` (full decision card with tier, strategy,
  reason, downgraded-from), `<sui-isolation-verify>` (probe results
  list with per-check pass/fail tone). All three auto-fetch from
  `/api/v1/isolation/*` when no props supplied; pass-through props
  for views that already hold the data. 33 vitest cases. Picker
  remains separate — design-heavier, paused for visual pairing.

## Known gaps

1. **DockerEngine has no host-side ConnectRPC client.** Today it
   drives via `docker exec` (no sidecar — see
   `wrap_for_external_spawn`). Adding the ConnectRPC client path
   is the option for richer in-container control (e.g., per-process
   stop/kill without a fresh exec).
2. **YoukiRootless lifecycle not yet SandboxOwner-attached.** Same
   pattern as DockerEngine. Host driver now speaks ConnectRPC; the
   remaining piece is wrapping the driver in an `Isolator` impl
   + supervisor wiring.
4. **WSL2 / LimaVm / FlyMachines** — strategies declared, capability-
   probed, no backend implementations yet. Each is a separate chunk.
5. **Tier 1 supervisor enforcement matrix** — exercises HarnessNative
   for claude `--sandbox` and SrtWrapper for opencode via the adapter
   table. Full (harness × OS) integration matrix in CI is the
   verification chunk.
6. **UI picker + wizard + settings panel** — `<sui-isolation-picker>`
   is design-heavier (radio cards with availability dots per tier,
   hover-to-preview); awaits visual pairing. The wizard + settings
   page that compose badge + detail + verify + picker is the next
   UI tranche after picker lands.
7. **Credential broker** — proto defines `SecretRequest` /
   `SecretReply` envelope variants; no handler. Wires to shipd's
   keychain bridge.
8. **Threat model doc** — separate doc when Phase G starts.
9. **Image distribution + signing** — `ghcr.io/ship/agent-runner`
   is referenced but not yet published. Build + sign + version-pin
   pipeline is its own work.

## Architecture map (current state)

```
ship-substrate (L0)
  isolation/
    trait_def.rs   — Isolator trait + wrap_for_external_spawn hook
    policy.rs      — SandboxPolicy, NetworkPolicy, ResourceLimits
    handle.rs      — SandboxHandle, ExecHandle, SandboxId
    owner.rs       — SandboxOwner (Drop fires shutdown)
    noop.rs        — baseline Isolator (always-available)
  supervisor/
    ActorHandle.sandbox: Option<Arc<SandboxOwner>>  ← lifetime binding

ship-isolation (L1)
  proto/sidecar.proto   — v1 wire schema
  src/
    sidecar/proto.rs       — generated Rust bindings
    backends/
      docker.rs            — Docker/Podman; wrap_for_external_spawn impl
      youki.rs             — Youki rootless (legacy one-shot path)
      youki_sidecar.rs     — sidecar-driven youki (ConnectRPC client)
      sandbox_runtime.rs   — srt wrapper
    capabilities.rs        — host probe (docker/podman/youki/lima/...)
    harness.rs             — per-(harness × OS) Tier 1 table
    tier.rs                — IsolationTier + IsolationStrategy enums
    recommender.rs         — pure (caps, overrides) -> resolved
    registry.rs            — Strategy -> Arc<dyn Isolator> factory map
    settings.rs            — wrap_launch + WrapContext
    strategies_config.rs   — per-strategy nested settings blob
    net/proxy.rs           — host-side allowlist HTTP proxy

ship-platform (L1)
  src/
    isolation.rs           — decide(), decide_for_agent(),
                             enforcement_enabled(), cached_capabilities,
                             strategies_settings reader/writer
    isolation_ops.rs       — recommendations(), preview(),
                             verify_decision(), init()

ship-supervisor (L1)
  src/executors/process.rs — observe + enforce + wrap + attach
                             SandboxOwner

apps/shipd
  src/routes/isolation_api.rs       — 6 HTTP endpoints
  src/routes/isolation_api_tests.rs — 13 integration tests

apps/ship-cli
  src/isolation_cmd.rs              — daemon-mediated CLI with
                                     in-process fallback
  src/validate.rs                   — isolation block check

apps/mcp
  src/server/mod.rs                 — isolation_status MCP tool

packages/ui-components
  src/sui-isolation-badge/          — reusable agent-domain component
  test/sui-isolation-badge.test.ts  — 15 vitest cases
```

## Test counts (all green)

- 81  ship-isolation lib + 7 proto smoke + 6 docker (live, skip)
- 10 ship-sidecar v1 end-to-end (incl. PTY attach + resize)
- 361 ship-substrate lib (includes 2 SandboxOwner Drop tests)
- 33  ship-supervisor lib
- 1055 ship-platform lib
- 665 shipd lib (includes 13 new isolation endpoint tests)
- 19 ship-cli validate (includes 5 new isolation block tests)
- 15 sui-isolation-badge component tests
