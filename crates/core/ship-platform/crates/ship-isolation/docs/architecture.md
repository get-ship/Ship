# ship-isolation — architecture

What the crate is and how the pieces fit together. Read this before
diving into any one backend.

## What it isolates

The *agent process* — the harness binary (Claude Code, Codex, etc.)
that the orchestrator spawns. Filesystem reads + writes, network
egress, subprocesses, signals.

What it does **not** isolate: the worktree directory itself — that's
prepared upstream by `ship_substrate::services::dispatch_ports::
IsolationStrategy` (worktree-prep, distinct trait, same word). The two
compose: worktree → process sandbox.

## The trait

```text
pub trait Isolator {
    fn name(&self) -> &'static str;
    async fn available(&self) -> Result<()>;
    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle>;
    async fn exec(&self, sandbox: &SandboxHandle, cmd: LaunchCommand) -> Result<ExecHandle>;
    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<()>;
}
```

`SandboxPolicy` is the unified policy DSL — mounts, network, env,
resource limits. Backends translate it into whatever their substrate
needs (Seatbelt profile, OCI runtime spec, sidecar HTTP request).

## Backends

```
┌──────────────────────┐
│       caller         │
│ (orchestrator / cli) │
└──────────┬───────────┘
           │  Isolator trait
           v
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   Noop ────► spawns unsandboxed (baseline)                   │
│                                                              │
│   SandboxRuntime ──► bun run @anthropic-ai/sandbox-runtime  │
│       │              ↓                                       │
│       │              Seatbelt (macOS) / bubblewrap (Linux)   │
│       │                                                      │
│   Youki ─────► youki run --bundle <…>                        │
│       │        ↓                                             │
│       │        rootless OCI container, cgroups v2,           │
│       │        bind-mount workspace at identical path        │
│       │                                                      │
│   SidecarSandbox ─► youki run, but PID 1 inside is           │
│                    ship-sidecar; host drives via HTTP        │
│                    over a free TCP port; long-lived,         │
│                    multiple execs per sandbox                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

The trait is the same surface for everything; backends differ in how
they enforce the policy and how many features they support.

## SidecarSandbox: the production-shape backend

Container topology:

```
┌── Host (shipd) ─────────────────────────────────────┐
│                                                     │
│   SidecarSandbox                                    │
│     │                                               │
│     ├─► spawns youki run (detached child)           │
│     │       │                                       │
│     │       v                                       │
│     │   ┌── Container ───────────────────────────┐  │
│     │   │ rootfs = ubuntu-base + bun + claude   │  │
│     │   │ PID 1 = /usr/local/bin/ship-sidecar    │  │
│     │   │             │                          │  │
│     │   │             ├─ axum on TCP :PORT       │  │
│     │   │             │  (/v1/exec, /v1/events,  │  │
│     │   │             │   /v1/stdin, /v1/signal, │  │
│     │   │             │   /v1/health, /v1/shutdown│ │
│     │   │             │                          │  │
│     │   │             ├─ /work    (workspace)    │  │
│     │   │             ├─ /root/.claude/.creds    │  │
│     │   │             │  (ephemeral copy)        │  │
│     │   │             └─ HTTPS_PROXY=…           │  │
│     │   │                  ↓                     │  │
│     │   │       agent (claude/codex/…) inherits  │  │
│     │   │       env from sidecar parent          │  │
│     │   └────────────────────────────────────────┘  │
│     │                                               │
│     ├─► spawns ProxyHandle (host-side TCP proxy)    │
│     │       enforces hostname allowlist             │
│     │       logs each decision                      │
│     │                                               │
│     └─► reqwest::Client talks to in-container       │
│         sidecar over 127.0.0.1:PORT                 │
└─────────────────────────────────────────────────────┘
```

Notes:

- The sidecar and proxy both listen on `127.0.0.1` because the
  container's network namespace is shared with the host (no
  `--unshare-net`). Production wants vsock or a unix-socket bind-mount
  so the container's network namespace can be isolated.
- The sidecar binary is bind-mounted from the host's
  `target/debug/ship-sidecar` for dev; production should bake it into
  the rootfs at build time.
- The proxy is HTTP CONNECT only (sufficient for HTTPS clients). No
  body inspection, no SOCKS5, no plain HTTP via absolute-URI form yet.

## Composing helpers

- **`EphemeralFile`** — copies a host file into a per-sandbox scratch
  dir, returns an `FsMount` for it, deletes the scratch on `Drop`.
  The "option B" credentials path. Production wants `option C` (full
  credential broker via sidecar `/v1/secret/get`).
- **`HostMatcher`** — exact + `*.suffix` glob hostname matcher used
  by the proxy.
- **`spawn_allowlist_proxy(matcher)`** — starts the proxy, returns
  `ProxyHandle` with port + log + shutdown channel.

## Rootfs

Built once by `scripts/build_agent_rootfs.sh`:

1. Download Ubuntu-24.04-base tarball (~30 MB).
2. Extract to `<out>/`.
3. `bwrap --uid 0 --gid 0` into it, set `APT::Sandbox::User=root` so
   apt doesn't try to drop privs in the user namespace.
4. `apt-get install ca-certificates curl unzip git openssl`.
5. `curl bun.sh/install | bash` with `BUN_INSTALL=/opt/bun`.
6. `/opt/bun/bin/bun add -g @anthropic-ai/claude-code`.
7. Pre-create `/root/.claude/` and `/work` for bind mounts.

Final size ~200 MB. Not in git; produced locally or fetched as a
tarball.

## Wire types

`src/sidecar/protocol.rs` defines the JSON shapes the sidecar
HTTP API speaks. The wire surface is modeled on
`rivet-dev/sandbox-agent`'s OpenAPI but uses our policy types.

Endpoints implemented today:

| Endpoint | Method | Body | Response |
|---|---|---|---|
| `/v1/health` | GET | — | `HealthResponse` |
| `/v1/session/start` | POST | `SessionStartRequest` | `SessionStartResponse` |
| `/v1/exec` | POST | `ExecRequest` | `ExecResponse` |
| `/v1/stdin` | POST | `StdinRequest` | 204 |
| `/v1/signal` | POST | `SignalRequest` | 204 |
| `/v1/events` | GET | — | `text/event-stream` of `SidecarEvent` |
| `/v1/shutdown` | POST | `ShutdownRequest` | 204 |

Type-defined but unimplemented today:

| Endpoint | Why |
|---|---|
| `/v1/mount` | runtime mount add — wait until first real caller |
| `/v1/policy` | runtime policy update — same |
| `/v1/port-publish` | port forwarding — wait for shared-net retirement |
| `/v1/secret/get` | credential broker (option C) — needs design |

## Test surface

- 23 unit tests covering policy, helpers, sidecar protocol, proxy.
- 5 live integration tests (auto-skip if external deps missing):
  - `srt_live` — sandbox-runtime backend deny + allow
  - `youki_live` — basic container exec
  - `run_claude_live` — Claude Code in container with real OAuth
  - `youki_sidecar_live` — sidecar-driven container exec
  - `network_live` — Allowlist policy enforcement
- 4 sidecar-binary integration tests:
  - `health` — `/v1/health`
  - 3 exec/stdin/signal tests under `tests/exec.rs`

## Out of scope on this branch

- Orchestrator integration (no L1+ crate imports ship-isolation yet).
- PTY support (current stdio is plain pipes; interactive Claude Code
  emits a "no stdin data received in 3s" warning).
- Cross-OS: macOS Lima, Windows WSL2 / Hyper-V.
- Sleep/wake recovery.
- Image distribution.
- Credential broker (option C).
- Plain HTTP / SOCKS5 in the proxy.
