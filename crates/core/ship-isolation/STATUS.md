# ship-isolation — status

**Phase:** experimental spike. No existing crate imports this yet.

## Verified working (live tests, real binaries)

- **`SandboxRuntime` backend** — `tests/srt_live.rs` against installed
  `@anthropic-ai/sandbox-runtime@0.0.51`:
  - Blocks a write to a host path outside the workspace allowlist.
  - Allows a write to a host path inside the workspace allowlist.
  - Settings JSON schema matches `srt`'s zod validator
    (`denyRead`/`allowWrite`/`denyWrite` all required even when empty).
- **`Youki` backend** — `tests/youki_live.rs` against `youki v0.6.0`
  with an Alpine 3.19 rootfs:
  - Materializes a rootless OCI bundle from a `youki spec --rootless`
    template + the policy DSL.
  - Spawns `youki run`; container starts.
  - Bind-mounted workspace passes through (write inside container
    appears on host).
- **Claude Code inside a Ship container with real OAuth** —
  `examples/run_claude.rs` + `tests/run_claude_live.rs`:
  - Ubuntu-24.04-base rootfs with `bun` + `@anthropic-ai/claude-code`
    preinstalled (build via `scripts/build_agent_rootfs.sh`).
  - Workspace + `~/.claude/.credentials.json` + `/etc/resolv.conf`
    bind-mounted; `NetworkPolicy::Open` (shares host net).
  - Claude Code 2.1.143 boots in the container, reads its OAuth
    credentials from the bind mount, calls the Anthropic API, returns
    a response, writes to the workspace, exits clean.
  - Integration test takes ~8s end-to-end.
  - Direct probe (`/tmp/probe-bundle`) confirms container's `/home`
    is the rootfs's `/home` (not the host's): host `~/.bashrc` is
    invisible from inside; only what the policy mounts is visible.
- **`ship-sidecar` binary** — `bin/ship-sidecar/tests/{health,exec}.rs`:
  - Boots on a free port via axum.
  - `/v1/health` returns the `HealthResponse` shape.
  - `/v1/exec` spawns a child with piped stdio, returns exec_id + pid.
  - `/v1/stdin` writes base64 bytes to a running exec.
  - `/v1/signal` dispatches kill(2) with caller-specified signal.
  - `/v1/events` SSE streams Stdout/Stderr/Exited events.
  - `/v1/shutdown` triggers graceful axum shutdown.
- **`SidecarSandbox` wrapper** (feature `sidecar-mode`, Linux only) —
  `src/backends/youki_sidecar.rs`:
  - Spawns a long-lived youki container with `ship-sidecar` as PID 1.
  - Drives exec / stdin / signal / wait via HTTP against the
    sidecar's free TCP port (host network namespace shared for the
    spike).
  - Bind-mounts the host's `ship-sidecar` binary into the container
    at `/usr/local/bin/ship-sidecar` — no rootfs rebuild needed per
    sidecar change. Production should bake the sidecar into the
    rootfs.
  - `run_capture()` convenience: exec + collect stdout/stderr/exit
    via the SSE stream in one call.
- **Allowlist network mediation** — `src/net/proxy.rs`:
  - Small Rust HTTP CONNECT proxy that listens on a free TCP port.
  - `HostMatcher` supports exact + `*.suffix` glob.
  - When `NetworkPolicy::Allowlist { hosts }` is set on a
    `SidecarSandbox`, the wrapper spawns a proxy task, injects
    `HTTPS_PROXY` / `HTTP_PROXY` / `NO_PROXY` env into the sidecar's
    process, child execs inherit them. Denied CONNECTs get 403.
  - Live-verified by `tests/network_live.rs`: in-container curl gets
    blocked by the host proxy and the deny is recorded in the proxy
    log. ~400ms end-to-end.
  - **Scope limitations:** HTTP CONNECT only (sufficient for HTTPS);
    no MITM/body inspection; plain HTTP via absolute-URI form is not
    proxied; SOCKS5 not implemented. Production wants `srt`'s
    proxy or a fuller proxy implementation.

Live tests skip if their external dependency isn't on PATH. They are
NOT marked `#[ignore]` — they auto-run when the tooling is present,
which means a real CI box with the deps installed exercises them.

## Bugs the live tests caught and fixed

1. `srt`'s settings schema requires `denyRead` and `denyWrite` even when
   empty. The Rust backend originally used `skip_serializing_if =
   "Vec::is_empty"` and emitted `{"filesystem": {"allowWrite": ["..."]}}`
   → zod rejected with `denyRead: Required` / `denyWrite: Required`. Fixed
   by always serializing the four fs fields.
2. youki refuses to start a container without `PATH` in `process.env`.
   The original patcher overwrote the template's env with the cmd's env;
   when cmd had no env, PATH disappeared. Fixed by merging cmd env into
   template env and defaulting `PATH` if absent.
3. `Youki::exec()` re-patched the spec from the template using a
   default empty policy — wiping the mounts that `prepare()` had baked
   in. Fixed by stashing the policy in `YoukiState` and reusing it on
   exec.

## Known gaps

1. **Network policy on `Youki` is binary** — Open shares host net (with
   `/etc/resolv.conf` bind-mounted), DenyAll creates an empty netns
   (nothing routes out). `Allowlist` / `Proxy` not enforced. The
   `sandbox-runtime` pattern (HTTP+SOCKS5 host proxy + unix socket
   bind-mounted into container netns) is the eventual answer.
2. **Sidecar bring-up inside the container.** `Youki::exec()` and
   `run_to_completion()` run the caller's command directly via
   `youki run`. The sidecar would be PID 1 *instead*, with the
   caller's command driven via the sidecar HTTP API. Not wired.
3. **Sidecar surface beyond `/v1/health`.** All other endpoints exist
   as wire types in `sidecar::protocol`; no handlers yet.
4. **Credentials approach is bind-mount-the-host-file (option A)**
   today. The token never goes to disk inside the container *rootfs*,
   but the host's real `~/.claude/.credentials.json` is the source of
   the mount — a container exploit during the session has read-access
   to the host token. Option B (copy-to-per-sandbox-tmpfs on
   `prepare()`, destroy on `shutdown()`) is ~50 lines from here.
   Option C (full credential bridge: shipd reads OS keychain, sidecar
   proxies requests via `/v1/secret/get`, token never leaves shipd)
   is the production design.
5. **Fly / cloud spike** — bootstrap doc only (`docs/fly-bootstrap.md`).
   No `MachinesClient` Rust code, no live test.
6. **Toolchain install path** — `SandboxRuntime` and `Youki` both take
   explicit paths to their dependencies. Ship's installer would
   download `youki`, run `scripts/build_agent_rootfs.sh`, and pull
   the npm packages.
7. **Linux runtime deps for `SandboxRuntime`** — `bubblewrap` and
   `socat` must be installed by the user.
8. **Rootfs distribution.** The ~200 MB Ubuntu+bun+claude-code rootfs
   is built once locally via `scripts/build_agent_rootfs.sh`; for
   production, ship a prebuilt tarball over HTTP with version
   pinning and integrity check.
9. **macOS path** for `SandboxRuntime` — should work (sandbox-exec is
   built in) but untested on this branch.
10. **Orchestrator wiring** — out of scope. The trait plugs into
    `ship_platform::services::orchestrator` alongside the existing
    `IsolationStrategy` (worktree) and `JobExecutor`. Not wired.

## Naming

`Isolator` was chosen to avoid collision with the existing
`IsolationStrategy` trait in
`crates/core/ship-substrate/src/services/dispatch_ports.rs:248`,
which prepares the worktree directory. Distinct concerns.

## Files

```
crates/core/ship-isolation/
├── Cargo.toml
├── README.md
├── STATUS.md                      — this file
├── bin/ship-sidecar/
│   ├── Cargo.toml
│   ├── src/{main,server}.rs
│   └── tests/health.rs            — live HTTP integration test
├── docs/
│   ├── sidecar-protocol.md
│   └── fly-bootstrap.md
├── src/
│   ├── lib.rs
│   ├── trait_def.rs               — Isolator + LaunchCommand + errors
│   ├── handle.rs                  — SandboxId / Handle / ExecHandle
│   ├── policy.rs                  — SandboxPolicy DSL
│   ├── sidecar/
│   │   ├── mod.rs
│   │   └── protocol.rs            — wire types, round-trip tested
│   └── backends/
│       ├── mod.rs
│       ├── noop.rs
│       ├── sandbox_runtime.rs     — verified live
│       └── youki.rs               — verified live (feature gated, Linux)
└── tests/
    ├── srt_live.rs                — verified live
    └── youki_live.rs              — verified live (feature gated, Linux)
```

## How to run the live tests locally

```bash
# srt backend
bun add -g @anthropic-ai/sandbox-runtime
sudo apt install bubblewrap socat            # Linux only
cargo test -p ship-isolation --test srt_live -- --nocapture

# youki backend (Linux only)
# install youki to /tmp/youki (or set SHIP_YOUKI_BIN)
# generate a spec template + alpine rootfs (or set SHIP_YOUKI_SPEC + SHIP_YOUKI_ROOTFS)
cargo test -p ship-isolation --features youki-backend --test youki_live -- --nocapture

# sidecar
cargo build -p ship-sidecar
cargo test -p ship-sidecar
```
