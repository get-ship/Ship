# ship-isolation — status

**Phase:** Phase B near-complete. Tier 1 honest (HarnessNative +
SrtWrapper, per-(harness × OS) adapter table, supervisor enforcement).
Tier 2 has one strategy lifecycle-wired end-to-end (DockerEngine via
SandboxOwner). Protocol v1 frozen in protobuf via ConnectRPC; sidecar
handler rewrite is the next chunk.

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
- **UI** — `<sui-isolation-badge>` reusable component for any actor/
  agent view. 15 vitest cases. Picker / detail / verify components
  follow the same pattern (not yet built).

## Known gaps

1. **Sidecar handlers still use legacy JSON.** `bin/ship-sidecar/`
   speaks the hand-typed protocol. Rewriting handlers to ConnectRPC
   routing against the v1 .proto is the next chunk.
2. **No host-side ConnectRPC client** in backends. Sidecar protocol is
   defined; the Docker/Youki/Fly backends will use it once the sidecar
   handlers are ported. Today DockerEngine drives via `docker exec`
   (no sidecar — see `wrap_for_external_spawn`).
3. **YoukiRootless lifecycle not yet SandboxOwner-attached.** Same
   pattern as DockerEngine but the existing `bin/ship-sidecar/` build
   process needs the protocol port first.
4. **WSL2 / LimaVm / FlyMachines** — strategies declared, capability-
   probed, no backend implementations yet. Each is a separate chunk.
5. **Tier 1 supervisor enforcement matrix** — exercises HarnessNative
   for claude `--sandbox` and SrtWrapper for opencode via the adapter
   table. Full (harness × OS) integration matrix in CI is the
   verification chunk.
6. **UI components beyond badge** — picker, detail, verify each
   ~equivalent in size to the badge. Wiring the wizard + settings
   panel that compose them is the next UI tranche.
7. **PTY in sidecar** — proto defines `AttachPty` server-stream; no
   handler impl. Convergence with main's PTY-WS attach happens here.
8. **Credential broker** — proto defines `SecretRequest` /
   `SecretReply` envelope variants; no handler. Wires to shipd's
   keychain bridge.
9. **Threat model doc** — separate doc when Phase G starts.
10. **Image distribution + signing** — `ghcr.io/ship/agent-runner`
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
    sidecar/protocol.rs    — legacy JSON (deprecated)
    backends/
      docker.rs            — Docker/Podman; wrap_for_external_spawn impl
      youki.rs             — Youki rootless (legacy sidecar path)
      youki_sidecar.rs     — sidecar-driven youki
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
- 361 ship-substrate lib (includes 2 SandboxOwner Drop tests)
- 33  ship-supervisor lib
- 1055 ship-platform lib
- 665 shipd lib (includes 13 new isolation endpoint tests)
- 19 ship-cli validate (includes 5 new isolation block tests)
- 15 sui-isolation-badge component tests
