# ship-isolation — compatibility matrix

The unglamorous deliverable that makes isolation *credible*: a grid where
every `harness × OS × tier` cell is **either green-with-a-test or
red-with-a-documented-reason** — never an optimistic blank. This is what
lets us honestly tell a user "isolation works on your setup."

**Status legend (be strict):**
- ✅ **verified** — a passing test exercises the real thing.
- 🟡 **implemented** — code exists, *not* live-verified (no real substrate
  in CI yet).
- 🔴 **blocked** — a known caveat prevents it; reason documented.
- ⬜ **not built.**

Baseline (2026-05): the sidecar v1 protocol is ✅ (10 e2e tests vs the
spawned binary). **First container cell is now ✅** — an actor runs inside
a real rootless **podman** container driven over v1
(`podman_sidecar_live`), on Linux/Ubuntu-24.04. That test caught a real
bug en route (`--detach` passed to `create`, which both docker and podman
reject — the container path had never actually worked). `docker_live` +
`docker_lifecycle` also pass against podman now. The **app-plane
reachability keystone** is also ✅ — `container_reach_live` proves an
in-container actor opens a connection back to the host daemon plane (the
`SHIP_DAEMON_URL` path). Remaining cells are 🟡/⬜; the grind continues
(bubblewrap T1 next, then mac/win substrates).

Scope: harnesses are the three darlings — **claude, codex, opencode**
(gemini/cursor deprioritized, see harness strategy). OS = macOS / Linux /
Windows. Tiers per `substrate.md`.

## Grid 1 — Tier 1 (host OS-level): harness × OS

The dimension that varies most (harness-native flags + "does it even run").

| Harness | macOS | Linux | Windows (native) |
|---|---|---|---|
| **claude** | 🟡 Seatbelt / `--sandbox`. 🔴 **Intel Macs: claude-code doesn't run.** | 🟡 `--sandbox` (sandbox-runtime/bubblewrap) | 🔴 no OS-level sandbox primitive → use WSL (Linux row) |
| **codex** | 🟡 Seatbelt (`--sandbox workspace-write`) | 🟡 Seatbelt n/a → SrtWrapper (bubblewrap) | 🔴 → WSL |
| **opencode** | 🟡 no native mechanism → SrtWrapper (sandbox-exec) | 🟡 SrtWrapper (bubblewrap + seccomp) | 🔴 sandbox-runtime has **no Windows support** → WSL |

**Tier-1 takeaways / caveats:**
- **Windows-native has no Tier-1 story** — `@anthropic-ai/sandbox-runtime`
  is mac+Linux only, and there's no portable Windows sandbox primitive. The
  Windows path *is* WSL (a Linux substrate) → it inherits the Linux column.
- **Intel Macs**: claude-code unavailable (hard 🔴 for that harness there).
- Verified today = the adapter *selection* logic (unit tests). 🟡 = the
  harness actually running sandboxed on that OS is **not** in CI.

## Grid 2 — Tiers 2/3/4 (substrate): OS × tier

Harness is ~uniform here (it runs inside the Linux substrate); the variable
is the substrate mechanism.

| Tier | macOS | Linux | Windows |
|---|---|---|---|
| **T2 — container** | 🟡 podman/docker (own VM) · Apple Container (Apple Silicon) | ✅ **podman** (`podman_sidecar_live` runs an actor; `container_reach_live` proves it reaches the host daemon plane — rootless, ubuntu:24.04) · 🟡 docker/youki | 🟡 podman/docker in WSL2 / Docker Desktop |
| **T3 — Ship VM/distro** | ⬜ Lima VM + bubblewrap | ⬜ (Firecracker microVM for strong; else T1 covers) | ⬜ Ship WSL distro + bubblewrap |
| **T4 — cloud** | ⬜ protocol-ready (`RuntimeEndpoint::Remote`), Firecracker on a cloud host | ⬜ same | ⬜ same |

**Substrate caveats:**
- **bubblewrap *inside* a container** (Tier-1-wrap-in-Tier-2) needs
  `CAP_SYS_ADMIN` / unprivileged userns — often unavailable in a hardened
  container. Don't assume nesting; probe.
- **youki / rootless** needs unprivileged user namespaces enabled.
- **microVM per sandbox** is KVM/Linux only (Firecracker). mac/win can't do
  it cheaply (nested virt immature on Apple Silicon; slow on WSL) → one
  minimal VM + bubblewrap. See `substrate.md`.
- **Windows hypervisor-per-actor** needs Hyper-V (Pro); WSL caps at
  shared-kernel.

## CI plan — what's automatable vs manual

- **Linux is the workhorse.** GitHub Actions Linux runners can install
  **podman** + run **bubblewrap** → live `podman_sidecar_live` (T2) and
  Tier-1 bubblewrap tests are **CI-able**. This is where most 🟡→✅
  conversion happens, and it's the next concrete step.
- **macOS runners**: can verify Tier-1 Seatbelt/sandbox-exec live. Lima/Vz
  VMs in CI are flaky (nested-virt limits) → T2/T3 mac is **manual/periodic
  on real hardware**.
- **Windows runners**: can exercise WSL2; Docker Desktop in CI is heavy.
  T2/T3 win is **best-effort + manual**.
- **Reuse `tests/agents/harness-compat/`** — add OS/tier dimensions to the
  existing suite; do **not** fork a parallel framework (per the
  architecture review).

## How to fill a cell (turn 🟡 → ✅)

1. Provision the substrate (e.g., `dnf install podman` on the Linux runner).
2. Run the harness inside it via the sidecar: prepare → v1 health → spawn
   actor → output over AttachPty → reap.
3. Assert the isolation actually held (egress denied where policy says,
   fs scoped, etc.).
4. Mark ✅ with the test name; on failure, mark 🔴 with the reason.

~~Immediate next: podman → `podman_sidecar_live`.~~ **Done** ✅ (2026-05-21).
~~Then: container→host reachability.~~ **Done** ✅ — `container_reach_live`
proves an in-container actor reaches the host daemon plane via
`--add-host=host.docker.internal:host-gateway` + the
`Docker::container_reachable_url` rewrite (the `SHIP_DAEMON_URL` keystone).
Next: **bubblewrap T1 on Linux** (needs `bubblewrap` + `socat` via apt —
`srt_live` is written and auto-skips until they're installed), then mac
(Seatbelt) + the mac/win substrate provisioners.
