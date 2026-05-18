# Cloud spike — Fly Machines

**Status: bootstrap only.** This doc captures the scope-of-work for a
local→cloud end-to-end demo against Fly Machines. No `MachinesClient`
Rust code lives in this crate yet; the work is sized below.

## Why Fly

- Per-machine VM isolation already (Firecracker-derived `flyd`).
- Spawn-by-API in single-digit seconds, billed by second.
- Same sidecar binary (`ship-sidecar`) deploys as the container
  entrypoint — no special "cloud variant" needed.
- API surface is small and well-documented.

## Spike scope

End-to-end test: from a local Ship runtime, spawn a Fly Machine
running our sidecar image, run `claude -p "hello"` inside it via the
sidecar protocol, stream the output back, tear down.

### Pieces needed

1. **Sidecar image.** Dockerfile that builds `ship-sidecar`, copies a
   compiled agent harness (Claude Code et al.), `ENTRYPOINT
   ["/usr/local/bin/ship-sidecar", "--addr", "0.0.0.0:2468"]`,
   `EXPOSE 2468`. ~50 lines of Dockerfile.
2. **Fly app.** One Fly app per Ship installation (`ship-sidecar-<userid>`)
   that the spawn API targets. Bootstrap with `flyctl apps create`.
3. **`FlyMachineIsolator` impl.** New backend module
   (`src/backends/fly.rs`, feature `fly-backend`). `available()`
   checks for `FLY_API_TOKEN`; `prepare()` calls
   `POST https://api.machines.dev/v1/apps/<app>/machines` with the
   image ref + region + sidecar token; `exec()` POSTs through the
   sidecar HTTP API at the returned machine's public address;
   `shutdown()` calls `POST .../machines/<id>/destroy`.
4. **Smoke test.** `tests/fly_smoke.rs` — gated on `FLY_API_TOKEN`,
   spins a machine, hits `/v1/health`, kills it. Marked `#[ignore]`
   so it doesn't run in CI without creds.

### What this proves

- The trait survives "the sandbox lives on someone else's hardware."
- The sidecar protocol is transport-agnostic in practice, not just
  in design.
- The same `SandboxPolicy` value renders correctly to a Fly Machine
  config and is enforceable by the sidecar inside.

### What this deliberately does not do

- No cost-aware scheduling, no warm pool, no multi-region routing,
  no spot/on-demand mix. All deferred.
- No `MachinesClient` library; we hand-roll the four API calls we
  need. Wrap in a crate later if we use Fly in production.
- No image registry plumbing — push the sidecar image to Fly's
  registry by hand for the spike. Automate later.

### Pieces deferred to a follow-up

- `flyctl auth token` discovery / Ship-managed Fly app provisioning.
- `MachinesClient` wrapper crate.
- Multi-tenancy story (one Fly app per Ship install vs one shared
  app with per-machine isolation).
- Egress cost vs `Open` network policy.

## Why not E2B / Daytona / Modal first

All viable; Fly chosen because: (a) the Machines API is one of the
smallest and most stable, (b) `flyd` is open-source so we understand
what's underneath, (c) Fly's billing model maps cleanly onto Ship's
subscription thesis (predictable per-second cost, no per-token cloud
margin). Adding E2B etc. as additional backends after the trait
survives Fly is straightforward — the work pattern is the same.
