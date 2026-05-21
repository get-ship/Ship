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
spawned binary). **Linux × T2 (podman) is ✅** — an actor runs inside a
real rootless **podman** container driven over v1 (`podman_sidecar_live`,
Ubuntu-24.04). The **app-plane reachability keystone** is ✅ too —
`container_reach_live` proves an in-container actor reaches the host daemon
plane (`SHIP_DAEMON_URL` + `--add-host`). As of the §5/§7.4 hardening, both
cells run over a **bind-mounted unix socket** control plane (no published
TCP port); `docker_lifecycle` covers prepare→reap. Remaining cells are
🟡/⬜.

Two test deletions to note (legacy-path cut): `docker_live` and `srt_live`
are **gone** — they exercised the removed `Isolator::exec` / `SandboxRuntime`
paths. `docker_lifecycle` (prepare/shutdown, no exec) survives. Production
Tier-1 is `wrap_launch` (argv transform), which **never had** a live test —
so Tier-1 live verification is a *new* test against `wrap_launch`, not the
deleted `srt_live`.

Scope: harnesses are the three darlings — **claude, codex, opencode**
(gemini/cursor deprioritized, see harness strategy). OS = macOS / Linux /
Windows. Tiers per `substrate.md`.

## Grid 1 — Tier 1 (host OS-level): harness × OS

The dimension that varies most (harness-native flags + "does it even run").

| Harness | macOS | Linux | Windows (native) |
|---|---|---|---|
| **claude** | 🟡 Seatbelt / `--sandbox`. 🔴 **Intel Macs: claude-code doesn't run.** | 🟡 `--sandbox` (sandbox-runtime/bubblewrap) | 🔴 no OS-level sandbox primitive → use WSL (Linux row) |
| **codex** | 🟡 Seatbelt (`--sandbox workspace-write`) | ✅ SrtWrapper (bubblewrap) — confinement live-verified | 🔴 → WSL |
| **opencode** | 🟡 no native mechanism → SrtWrapper (sandbox-exec) | ✅ SrtWrapper (bubblewrap + seccomp) — confinement live-verified | 🔴 sandbox-runtime has **no Windows support** → WSL |

**Tier-1 takeaways / caveats:**
- **SrtWrapper confinement is live-verified on Linux** —
  `tier1_srt_confinement_live` drives the production path
  (`render_settings` → `wrap_launch(SrtWrapper)` → spawn) and asserts the
  property, not just startup: a write outside the workspace is **blocked**
  (`Read-only file system`), a write inside **succeeds**. This is the
  mechanism codex + opencode route to on Linux, so those cells are ✅. It
  replaces the deleted `srt_live` (which tested the removed `SandboxRuntime`,
  never the production path).
- **`HarnessNative` cells stay 🟡** — claude `--sandbox` / codex Seatbelt
  use the *harness's own* sandbox (we pass the flag, don't wrap), so
  verifying them needs the actual harness installed + an escape assertion.
  That's the next live target.
- **Windows-native has no Tier-1 story** — `@anthropic-ai/sandbox-runtime`
  is mac+Linux only, and there's no portable Windows sandbox primitive. The
  Windows path *is* WSL (a Linux substrate) → it inherits the Linux column.
- **Intel Macs**: claude-code unavailable (hard 🔴 for that harness there).

## Grid 2 — Tiers 2/3/4 (substrate): OS × tier

Harness is ~uniform here (it runs inside the Linux substrate); the variable
is the substrate mechanism.

| Tier | macOS | Linux | Windows |
|---|---|---|---|
| **T2 — container** | 🟡 podman/docker (own VM) · Apple Container (Apple Silicon) | ✅ **podman** — runs (`podman_sidecar_live`) · reaches host plane (`container_reach_live`) · **confines fs** (`podman_fs_confinement_live`) · **blocks egress** (`podman_zero_egress_live`); rootless ubuntu:24.04, unix-socket control plane · 🟡 docker (same backend, untested since `docker_live` cut) · 🟡 youki (gated) | 🟡 podman/docker in WSL2 / Docker Desktop |
| **T3 — VM** | ⬜ Apple Container (microVM-per-container) · ⬜ Lima VM + bubblewrap | ✅ **Firecracker** microVM (`firecracker_live`: boots, sidecar-as-init over vsock, actor exits 0, **+ workspace round-trips** via data-drive work-unit copy) | ⬜ Ship WSL distro + bubblewrap |
| **T4 — cloud** | ⬜ protocol-ready (`RuntimeEndpoint::Remote`), Firecracker on a cloud host | ⬜ same | ⬜ same |

**Tier-2 confinement is now asserted, not assumed.** Beyond "the actor
runs," the Linux/podman cell proves the security properties: T-FS-READ
(can read the workspace mount, cannot read a host file outside it) and
T-NET-EGRESS (`DenyAll` → `--network none` blocks outbound TCP) — and the
egress test doubles as proof that the unix-socket control plane keeps the
sidecar reachable *with no network at all* (the hardened-tier payoff).
Remaining Tier-2 properties without a live test: T-RES (cgroup caps) and
T-CRED (the deferred broker).

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

~~Immediate next: podman → `podman_sidecar_live`.~~ **Done** ✅.
~~Then: container→host reachability.~~ **Done** ✅ (`container_reach_live`).
~~Then: unix-socket control plane.~~ **Done** ✅ (§5/§7.4 hardening).
~~Then: a Tier-1 live test against `wrap_launch`.~~ **Done** ✅ —
`tier1_srt_confinement_live` (SrtWrapper write-escape blocked). Next:
**HarnessNative confinement** (claude `--sandbox` / codex Seatbelt with the
harness installed — assert its escape is blocked too), then **mac Seatbelt**
(real-hardware/CI) + the mac/win **Tier-3** substrate provisioners.
