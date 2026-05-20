# ship-isolation

Agent-process isolation primitives for Ship. Wired through the
supervisor as sandbox-as-runtime (Phase C); Tier 0/1 in-process and
Tier 2 sidecar-driven containers both land here.

See [`docs/architecture.md`](./docs/architecture.md) for the elevator
pitch and component map. See [`docs/open-issues.md`](./docs/open-issues.md)
for locked decisions, the backlog, and what's left.

## What it is

A trait (`Isolator`) and policy DSL for sandboxing the agent process
the orchestrator spawns. Distinct from
`ship_substrate::services::dispatch_ports::IsolationStrategy`, which
prepares the *working directory* (git worktree). The two compose:
worktree → process sandbox.

## Backends

| Backend | Status | What it does |
|---|---|---|
| `Noop` | working | Runs unsandboxed. Baseline. |
| `SandboxRuntime` | live-verified (Linux) | Wraps `@anthropic-ai/sandbox-runtime` (`srt`) via Bun. Seatbelt on macOS, bubblewrap+seccomp on Linux. |
| `Youki` (feature `youki-backend`, Linux) | live-verified | Renders an OCI runtime spec, spawns `youki run` rootless. Bind-mount workspace + ephemeral creds. |
| `SidecarSandbox` (feature `sidecar-mode`, Linux) | live-verified | Long-lived youki container; PID 1 is `ship-sidecar`; host drives exec/stdin/signal/events over HTTP+SSE. Allowlist network policy enforced by host-side HTTP CONNECT proxy. |
| Cloud / Fly | bootstrap doc | `docs/fly-bootstrap.md`. Sidecar binary builds; spawn helper deferred. |

## Try it

Build the agent rootfs (one time, ~200 MB):

```
scripts/build_agent_rootfs.sh /tmp/ship-rootfs/rootfs
```

Download youki:

```
curl -sSL https://github.com/youki-dev/youki/releases/download/v0.6.0/youki-0.6.0-x86_64-musl.tar.gz | tar xzf - -C /tmp
```

Generate a bundle spec template:

```
mkdir -p /tmp/claude-bundle
cd /tmp/claude-bundle && /tmp/youki spec --rootless
```

Build the sidecar binary:

```
cargo build -p ship-sidecar
```

Run Claude Code inside a Ship-managed container, with real OAuth
credentials and an egress allowlist for the Anthropic API:

```
cargo run -p ship-isolation \
  --features 'youki-backend sidecar-mode' \
  --example run_claude_sidecar -- /tmp/work "Say hello"
```

## Test

```
cargo test -p ship-isolation
cargo test -p ship-isolation --features youki-backend
cargo test -p ship-isolation --features 'youki-backend sidecar-mode'
cargo test -p ship-sidecar
```

Live tests under `tests/*_live.rs` auto-skip when their external
dependency (youki, srt+bwrap, rootfs, etc.) isn't present.
