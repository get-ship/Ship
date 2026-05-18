# ship-isolation — strategy

The canonical design document for Ship's agent-isolation story. Anything
that contradicts this doc is wrong; anything not covered here is open.

## 1. Purpose

Run agent processes in bypass-permissions mode without putting the
user's machine at risk. A compromised or mistaken agent should have a
bounded blast radius. The user should be able to:

- get a secure default they don't have to understand;
- pick a stronger tier with one click;
- override per-agent or per-run when they need to;
- see honestly what isolation is in effect right now.

## 2. Tiers

Four tiers, ordered by strength. Each tier has one or more concrete
**strategies** that implement it. The user picks the tier; the
**recommender** picks the strategy unless the user pins one.

> Naming: we use `isolation_tier` everywhere to avoid collision with the
> existing `permissions_tier` (standard / autonomous / read-only) on
> agents. Those are *what the agent is allowed to attempt*; isolation
> tier is *where the agent runs*. The two compose orthogonally.

| Tier | Name (id) | What it gives you |
|---|---|---|
| 0 | `none` | Host process. No sandbox. The agent is the user. |
| 1 | `process` | Host process with a per-process filesystem + syscall sandbox (Seatbelt / bubblewrap / harness-native). |
| 2 | `container` | OCI container — separate filesystem, separate network namespace, cgroup limits, host kernel. |
| 3 | `vm` | MicroVM or full VM — separate kernel. Same shape as Tier 2 from the user's view. |
| 4 | `cloud` | Vendor-served sandbox. Sidecar runs in vendor's container/VM; we don't touch the host. |

Tier 0 exists and ships on day one — it's what most users have today
implicitly. The UI must label it honestly: "Not sandboxed".

## 3. Strategies

A strategy is one concrete implementation of a tier. Each strategy
implements `Isolator`. Strategies are *capabilities-gated*: a strategy
that requires Docker won't be picked on a host without Docker.

| Strategy id | Tier | Substrate | Bundle add | Notes |
|---|---|---|---|---|
| `Noop` | 0 | host process | 0 | Baseline; for tests and "opt out". |
| `HarnessSandbox` | 1 | `@anthropic-ai/sandbox-runtime` (srt) | npm dep | Seatbelt on macOS, bwrap+seccomp on Linux. The same surface claude/codex CLIs use natively. |
| `DockerEngine` | 2 | host `docker` or `podman` | 0 (BYO) | Smallest surface — daemon does everything. The preferred Tier 2 when present. |
| `YoukiRootless` | 2 | bundled `youki` + Ubuntu rootfs | 6 MB binary + ~200 MB rootfs lazy-fetched | Gap-fill for users without Docker. Linux + WSL2. |
| `LimaVm` | 3 | bundled `lima` (Apple Vz on arm64 Mac) | ~30 MB + ~500 MB image | macOS arm64 path. Intel Mac falls through to cloud. |
| `Wsl2` | 2 | host WSL2 distro | 0 (BYO WSL) | Windows path when WSL2 present. |
| `FlyMachines` | 4 | Fly.io API | 0 | Cloud Tier 4. Same sidecar; vendor brings the kernel. |

Adding a strategy means: implement `Isolator`, declare the
`StrategyCapabilities` it needs (e.g. `docker_or_podman_present`),
register it. The recommender takes it from there.

## 4. Per-OS default matrix

The recommender's out-of-box pick, given no user pref and a fresh
install. Each row prefers the leftmost available strategy.

| OS / device | Tier 0 | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Default tier |
|---|---|---|---|---|---|---|
| Linux x64/arm64 | Noop | HarnessSandbox | DockerEngine → YoukiRootless | — | FlyMachines | **2** |
| WSL2 | Noop | HarnessSandbox | DockerEngine → YoukiRootless | — | FlyMachines | **2** |
| macOS arm64 | Noop | HarnessSandbox | DockerEngine | LimaVm | FlyMachines | **1** (jump to 3 if user opts up) |
| macOS Intel | Noop | HarnessSandbox | DockerEngine | — (Lima/QEMU is too slow) | FlyMachines | **1** |
| Windows 11 | Noop | — | DockerEngine → Wsl2 | — | FlyMachines | **2** if Docker/WSL2 present, else **4** |
| Windows 10 | Noop | — | Wsl2 if present | — | FlyMachines | **4** if no WSL2, **2** if WSL2 present |
| Detected-in-sandbox (CI, container, codespace) | Noop | — | — | — | — | **0** (already in a sandbox; no double-wrap) |

The default tier is per-OS, not per-strategy. Within a tier, the
recommender picks the first available strategy in preference order.

## 5. Capability detection

A `Capabilities` snapshot is built at shipd start and cached. Probed:

- `docker` — `docker version` succeeds and daemon reachable
- `podman` — `podman version` succeeds (Docker-compatible socket optional)
- `youki` — bundled binary present + cached rootfs version-matches
- `lima` — bundled binary present + a `ship-default` instance reachable
- `tart` — `tart` on PATH (macOS arm64; Lima alternative)
- `wsl2` — Windows + `wsl --status` reports WSL2 default
- `srt` — `bun` + `@anthropic-ai/sandbox-runtime` installed
- `sandbox_exec` — macOS only; `sandbox-exec` on PATH
- `vz_framework` — macOS arm64
- `fly_cli` — `flyctl` on PATH **and** authenticated
- `in_sandbox` — heuristic: cgroup parse, `/.dockerenv`, `WSL_DISTRO_NAME`, `OZ_*` env

Snapshot is exposed as a daemon setting (`isolation.capabilities_cached`)
and re-probed on `ship isolation rescan`. A capability flip emits a
`config.changed` event so the UI updates.

## 6. Recommender

```
input:  Capabilities, OS, user_pref, agent_override, run_override
output: ResolvedIsolation { strategy, tier, reason, downgraded: bool }
```

Algorithm:

1. If `run_override` pins a strategy: use it. If unavailable, **fail
   loud** — don't silently fall back when the human explicitly chose.
2. Else if `agent_override` pins: same rule.
3. Else if `user_pref` pins a tier and at least one strategy in that
   tier is available: pick highest-preference available.
4. Else: take the per-OS default tier; pick highest-preference
   available strategy.
5. If nothing in the chosen tier is available, walk **down** the tiers
   in the order user_pref → default → cloud → none. Each downgrade is
   recorded in `reason` and the result is flagged `downgraded: true`
   so the UI can warn.

A downgrade to Tier 0 must surface in the UI as a banner, not a debug
log. If the user has explicitly enabled `isolation.allow_tier_0 = false`,
the downgrade refuses and the agent run fails with a clear message.

## 7. Override model

Three override levels, narrowest wins:

```
run_override     (one-off, e.g. `ship use --isolation=container`)
  └── agent_override   (in agent profile YAML)
       └── user_pref   (in daemon settings)
            └── per-OS default
```

Each level can pin a **tier**, a **strategy**, or a full
`IsolationProfile` (custom mounts, custom allowlist, resource limits).

## 8. Agent profile schema

```yaml
# in .ship/agents/foo.toml or foo.yml
isolation:
  tier: container          # 0|1|2|3|4 or none|process|container|vm|cloud
  strategy: docker         # optional pin within tier
  network: allowlist       # deny_all|allowlist|open|proxy
  allowlist:               # only when network = allowlist|proxy
    - api.anthropic.com
    - "*.github.com"
  mounts:                  # additional bind mounts beyond workspace
    - host: ~/.gitconfig
      sandbox: /root/.gitconfig
      mode: ro
  resources:
    memory_mb: 4096
    cpu_quota: 200         # 200% = 2 cores
    pids_max: 512
```

All fields optional. Missing fields inherit from the user defaults.

## 9. User-facing settings keys

Stored in `daemon_settings` (the same KV that powers `terminal.program`
etc.). All keys live under the `isolation.` namespace.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `isolation.default_tier` | enum | per-OS (table §4) | Preferred tier when nothing overrides. |
| `isolation.default_strategy` | enum | empty | Optional pin within `default_tier`. |
| `isolation.allow_tier_0` | bool | `true` | If false, recommender refuses to downgrade to Tier 0. |
| `isolation.cache_dir` | path | `~/.ship/cache/isolation` | Where rootfs / images live. |
| `isolation.lazy_fetch` | bool | `true` | Download heavy assets on first use vs at install. |
| `isolation.network_default` | enum | `allowlist` | DenyAll / Allowlist / Open / Proxy when agent doesn't specify. |
| `isolation.allowlist_default` | list[str] | `["api.anthropic.com", "console.anthropic.com", "*.github.com"]` | Default egress hostnames. |
| `isolation.capabilities_cached` | json | computed | Last `Capabilities` snapshot (auto-managed). |
| `isolation.youki_bin` | path | `~/.ship/cache/isolation/youki` | Override the bundled youki binary. |
| `isolation.youki_rootfs` | path | `~/.ship/cache/isolation/ubuntu-rootfs` | Override the bundled rootfs. |
| `isolation.lima_bin` | path | `lima` on PATH | Override Lima binary. |
| `isolation.fly_org` | string | empty | Fly.io org for Tier 4. |
| `isolation.fly_app` | string | `ship-agents` | Fly.io app name. |

Env vars (read by the daemon at startup; settings override env):

| Env var | Maps to |
|---|---|
| `SHIP_ISOLATION_TIER` | `isolation.default_tier` |
| `SHIP_ISOLATION_STRATEGY` | `isolation.default_strategy` |
| `SHIP_ISOLATION_CACHE_DIR` | `isolation.cache_dir` |
| `SHIP_YOUKI_BIN` | `isolation.youki_bin` |
| `SHIP_AGENT_ROOTFS` | `isolation.youki_rootfs` |
| `SHIP_LIMA_BIN` | `isolation.lima_bin` |
| `SHIP_FLY_ORG` | `isolation.fly_org` |

The running canonical list lives at
`.artifacts/local-sandbox-spec/settings-and-envs.md`.

## 10. UX contract

The UI must always be able to answer four questions about any agent run:

1. **What tier am I in?** Shown as a chip near the agent name. Plain
   words: "Sandboxed (container)" / "Sandboxed by Claude" / "Cloud
   (Fly)" / "Not sandboxed" — never "Tier 2" alone.
2. **Why this one?** A tooltip explains the reason: "Recommended for
   your system" / "Set in agent config" / "Set in user defaults" /
   "Downgraded from Tier 2 because Docker stopped responding".
3. **What's exposed?** A details panel lists mount points, network
   policy, and credentials in scope.
4. **How do I upgrade?** If on a weaker tier than default, a hint:
   "Install Docker to enable container isolation" or "Sign in to Fly
   for cloud isolation".

A downgrade to Tier 0 is a banner, not a log line. Silent downgrades
are a bug.

## 11. Sidecar parity

The sidecar must support every Ship feature an agent can use. Anything
that crosses the sandbox boundary needs a sidecar endpoint or a host
bind-mount. The audit lives in `docs/sidecar-parity.md` — for each
feature, status is one of `direct-mount` / `sidecar-endpoint` / `gap`.

Hard requirements:

- **Harness CLI** (claude / codex / opencode / gemini / cursor-agent):
  exec via sidecar `/v1/exec` with full env passthrough. Done for
  claude via `run_claude_sidecar` example.
- **MCP servers**: stdio MCP servers run *inside* the sidecar container
  (sidecar spawns them via `/v1/exec`). HTTP MCP servers reachable via
  the proxy allowlist if external.
- **Credentials**: option C broker — sidecar requests creds via
  `/v1/secret/get`; shipd resolves from OS keychain; token never lands
  on container disk. Bind-mount (option B) is the interim.
- **PTY**: agents that need a real TTY (interactive claude) get a
  PTY-backed `ActorHandle` exposed via `/v1/exec` with `pty: true`,
  attachable from the host's `/api/v1/agent_sessions/{slug}/attach`
  WebSocket.
- **Hooks**: hook commands declared on agents must dispatch through
  the sidecar so they run in the sandboxed env, not on the host.
- **File watcher**: workspace changes detected via host-side notify
  on the bind-mounted dir — no in-container watcher needed.
- **Log streaming**: sidecar `/v1/events` SSE already handles stdout
  / stderr / exit. Structured `log_progress` calls round-trip through
  the same channel.

## 12. Phasing

Order is dependency-driven, not calendar-driven.

| Phase | Deliverable | Gates next |
|---|---|---|
| **A** | Tier + Strategy + Capabilities + Recommender + settings + per-OS defaults | Everything |
| **B** | `DockerEngine` adapter (gap-fills the most common case at minimum surface) | UI tier picker has more than one real option |
| **C** | `HarnessSandbox` adapter wraps existing `SandboxRuntime` cleanly | macOS arm64 default works |
| **D** | Sidecar PTY + parity audit + credential broker (option C) | Production-shape integration |
| **E** | UI: tier chip, "why this tier?" tooltip, settings screen | Honest UX |
| **F** | `LimaVm`, `Wsl2`, `FlyMachines` adapters | Full matrix |
| **G** | Threat-model doc + pen-test + cross-OS sidecar CI | Production-ready label |

Phase A is in this branch. The rest follow as separate branches that
merge into this design.

## 13. Out of scope (here)

- Threat model details — separate doc when Phase G starts.
- Auditing / forensics of agent activity inside sandboxes.
- Image signing + reproducible rootfs builds (later, before public release).
- GUI for editing per-agent isolation profile YAML — CLI-first, UI follows.
