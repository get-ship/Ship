# ship-isolation — substrate model

**Status: design intent, refined from extended pairing.** Supersedes the
earlier `vm-substrate.md` (which framed "youki+VM" as the primary path —
that double-layered containers inside a VM unnecessarily). Pair with
`runtime-architecture.md` (the sidecar spine) and `strategy.md` (tiers).

## The invariant: the sidecar is the constant; the substrate is the variable

`ship-sidecar` — one static Linux binary speaking `ship.sidecar.v1` —
runs **identically** in every substrate:

- a **container** (podman / docker / Apple Container / youki),
- a **Ship-managed Linux VM** (Lima on macOS),
- a **Ship-managed WSL distro** (Windows),
- or **directly on a Linux host**.

It supervises actors and speaks the same protocol everywhere. **This
consistency across substrates is the architectural priority.** Forcing
*containers specifically* is wrong — a container is one substrate the
sidecar runs in, not *the* substrate. Two things vary, both as parameters
around the constant sidecar:

1. **Byte transport** — unix socket / vsock / forwarded port / TCP+TLS.
   `RuntimeEndpoint::{InProcess, Remote{addr,scheme}}` already abstracts it.
2. **Inner per-actor isolation** — see below.

## Inner isolation is parameterized, not fixed

- **In a container** (Tier 2): the container *is* the boundary. One sandbox
  = one container; no further nesting. (Nesting containers inside a VM is
  silly — it stacks two boundaries when the substrate already gave one.)
- **In a shared VM / distro / host** (Tier 1 & 3): **bubblewrap +
  seccomp + user namespaces** per actor — the OS-level mechanism Ship
  already has (Tier 1 / sandbox-runtime). The substrate is shared across a
  user's projects; bubblewrap separates them.

So the VM/distro path needs **no container runtime at all** — it reuses
Tier 1's OS-level isolation, with the VM acting purely as a Linux-kernel
provider on macOS/Windows.

## Tiers

| Tier | Substrate | Audience | Inner isolation |
|---|---|---|---|
| **T1** | host, no substrate | Linux / mac power users | bubblewrap / Seatbelt / harness-native |
| **T2 — container** | podman · docker · youki | users who **already run** a runtime | the container is the boundary (shared host kernel) |
| **T3 — VM** | **Firecracker** (Linux, KVM) · **Apple Container** (mac, microVM-per-container) · **Lima** (mac, shared VM + bubblewrap) · Ship **WSL** distro (win) | **consumer default** | own guest kernel — hypervisor boundary |

**Tier-3 inner-isolation strength varies (all are T3 vs the host, but differ between co-resident actors):**
- **Firecracker / Apple Container** — **microVM *per sandbox/actor***: each agent gets its own guest kernel. Strongest. Firecracker is the Linux/cloud model; Apple Container is its mac equivalent (Virtualization.framework).
- **Lima / WSL** — **one shared Ship VM**, actors separated *inside* it by bubblewrap (shared VM-kernel between co-resident actors). Strong vs the host, weaker between actors. The fallback for Intel/older macs (Lima) and Windows (WSL, which is inclusive — Hyper-V excludes Home).
- So **Apple Container is not weaker than Lima — it's stronger** (per-actor microVM vs bubblewrap-in-shared-VM); they compete at T3 on mac and we support both, detect-and-prefer Apple Container.

**Recommender (detect-or-bundle):** probe for a container runtime → use it
(T2, no download but the agent image). Else → the **bundled Ship
VM/distro** (T3). For a consumer with nothing installed, **escalate
straight to T3** — a bundled, Ship-managed Linux substrate is genuinely
lower friction than "go install and learn podman." Containers stay
important; they're just not what we make a non-technical user install.

## The Ship-managed substrate: dedicated, never the daily-driver

T3 is a **dedicated, headless, Ship-owned Linux substrate**, provisioned
on first isolation use and invisible to the user:

- **macOS:** a dedicated **Lima VM** (Lima instances *are* VMs — the
  heavier path; gives a real kernel boundary).
- **Windows:** a dedicated **WSL distro** via `wsl --import Ship-Runtime …`
  — a *sibling* to the user's daily-driver distro, never touching it.
  Because **WSL2 shares one utility VM across all distros**, the Ship
  distro is lightweight (separate rootfs, not a second VM). Clean removal:
  `wsl --unregister`.
- **Linux:** no substrate — sidecar + bubblewrap on the host directly.

In all cases: bake the sidecar + minimal tools, drive it headless, expose
the v1 endpoint (vsock / forwarded port), tear down on reap. Same
`Isolator`-shaped contract, a substrate lifecycle wrapped around it.

## Strength dial: bubblewrap (shared-kernel) vs microVM (own-kernel)

The inner isolation has two strengths, picked by trust:

- **bubblewrap** — namespaces + seccomp + userns. Shared kernel: a kernel
  exploit escapes the sandbox (to the VM's kernel on mac/win, or the host
  on Linux-direct). Light, fast, no hypervisor. Default for *trusted* work.
- **microVM** — a separate guest kernel per sandbox (hypervisor boundary).
  A kernel exploit is contained. For *genuinely untrusted* code.

**microVMs need a hypervisor, and which one is per-OS — this constrains
where per-sandbox microVMs are even affordable:**

| Host | Hypervisor | Per-sandbox microVM? |
|---|---|---|
| **Linux** | KVM | **Yes — Firecracker / Cloud-Hypervisor per sandbox.** Fast, strong. The Fly model. |
| **macOS** | Virtualization.framework | Not per-sandbox cheaply. Firecracker needs KVM; you'd run N native Vz VMs (heavy) or nested virt (immature on Apple Silicon). → one minimal Ship VM + bubblewrap. |
| **Windows** | Hyper-V | Nested virt *works* (WSL2 can nest KVM) but slow/fragile. → one Ship distro + bubblewrap; Hyper-V (Pro) for a real per-VM boundary. |

So: **Linux gets microVM-per-sandbox (Firecracker) for the untrusted
tier; mac/win realistically run one *minimal, fast-booting* Ship VM +
bubblewrap per actor.** The "micro, not fat" lesson (Fly) applies to *that*
Ship VM regardless — minimal rootfs, sidecar as init, fast boot. Beyond
shared-kernel strength on mac/win, the escalation is a **cloud machine
(Tier 4)** running Firecracker, not nested virt locally.

## Fleet networking: WireGuard mesh (forward reference)

When multiple user-controlled daemons + mobile clients exist, the
substrate↔host and node↔node reachability is a **private WireGuard mesh**
(Fly's 6PN / Tailscale's model): each node gets a private address,
NAT-traversed, e2e-encrypted; a thin coordinator (Ship Cloud) does
NAT-traversal signaling but **never sees the data**. WireGuard keys tie to
the device-key / macaroon identity. This is the sovereign relay — not in
scope for single-device isolation, but it's where the transport grows.

## Reachability follows from this (simpler than per-container)

Because per-actor isolation is bubblewrap inside *one* shared substrate
(not N containers), the network boundary is the **substrate↔host edge**
(vsock / forwarded port), mediated once — not per-container bridge
gymnastics. Each substrate provisioner answers "how does the inside reach
the host" (bridge gateway / slirp-pasta / vsock), so the supervisor stays
runtime-neutral. Egress policy and inbound port-forwarding are
tier-parameterized (see `open-issues.md`).

## What's built vs not

**Built:** the sidecar spine (v1, PtY, supervision), Tier 1 OS-level
isolation, the docker/podman backend (container path), youki-on-Linux.

**Not built (the primary consumer path):**
- Substrate provisioners: Lima VM lifecycle (mac), Ship WSL distro
  import/lifecycle (win).
- The baked substrate image/rootfs (sidecar + tools), built/signed/pinned.
- vsock / forwarded-port transport wiring (`RuntimeEndpoint::Remote`).
- Recommender extension: substrate detection + detect-or-bundle + the
  "escalate consumers to T3" policy. **Decided per-OS ladder (2026-05):**
  linux → **Firecracker** microVM (KVM) for strong T3, else native
  podman/youki (T2); mac → **Apple Container** (T3, microVM-per-container)
  if present, else **Lima** (T3 shared VM + bubblewrap, Intel/older); win →
  **WSL** distro (not Hyper-V — it excludes Home). Detect-and-prefer: Apple
  Container only when the `container` binary + macOS 15+/Apple-Silicon present.
- bubblewrap-as-inner-isolation inside the shared substrate (reuse Tier 1).

## Radar (watch, no critical-path change)

- **Apple Container** — promoted from radar to a **decided T3 provider**
  (2026-05): microVM-per-container (Virtualization.framework) = own guest
  kernel per agent, so it's T3 (VM boundary) with container ergonomics —
  *stronger* inter-actor isolation than Lima's shared-VM-bubblewrap, not
  weaker. Preferred mac path when present (Apple Silicon, macOS 15+); Lima
  the Intel/older fallback. Zero sidecar change; lives in the recommender's
  detect-and-prefer ladder.

## One-line intent

The sidecar runs in any Linux substrate; substrates are thin provisioners
whose only job is "run the sidecar and make it reachable"; T2 containers
are detected adapters, T3 Ship VM/distro is the bundled consumer default;
per-actor isolation is the container boundary or bubblewrap; nothing nests.
