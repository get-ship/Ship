# ship-isolation — VM substrate (intent)

**Status: intent capture, not a build spec.** This records the
understanding reached in design discussion so it isn't lost. The
buildable spec gets refined from here.

Pair with `runtime-architecture.md` (the sidecar-as-runtime spine)
and `strategy.md` (the tier model).

## The headline realization

**youki + a Ship-managed VM is the PRIMARY Tier 2 path, not a
fallback.** It's the one that makes Ship S-tier: a non-technical
user installs Ship and their agents are sandboxed having installed
nothing else — no Docker Desktop, no podman, no runtime setup.

Earlier specs treated youki as "a Linux backend that needs the
youki binary." That undersold it. The actual intent: **Ship owns
its own isolation substrate** so the container story has zero
external dependency on any platform. youki is the bundleable,
rootless, daemonless runtime that makes that possible; the VM is
what gives youki a Linux kernel on macOS / Windows.

docker/podman are *adaptors* for users who already have a runtime —
the secondary, low-friction-for-developers path. The substrate Ship
owns is youki+VM.

## Why a VM is required (the constraint that was missed)

youki is a Linux container runtime — Linux namespaces, cgroups v2,
pivot_root. It cannot run on the macOS or Windows kernel. So:

- **Linux**: youki runs directly. No VM. (The easy case.)
- **macOS / Windows**: youki runs *inside a Linux VM that Ship
  provides*. The VM supplies the Linux kernel; youki supplies the
  container; the user installs nothing.

youki removes the *docker/podman dependency*, not the *Linux-kernel
requirement*. The VM removes the kernel requirement on non-Linux.
Both together = zero-prerequisite containers on every OS.

## The recommender's resolution (detect-or-bundle)

Tier 2 strategy selection becomes:

1. **Probe for a compatible runtime** (docker / podman / OCI-
   compatible). If present → surface it as the strategy. No download
   except the agent image or the sidecar binary. (Technical users.)
2. **Else → bundled youki + Ship-managed Linux VM.** The self-
   contained path. One-time fetch of a VM image / rootfs. (Everyone
   else — the majority.)

The capability probe (`capabilities.rs`) already enumerates host
runtimes; this extends it with VM-provider detection + the
detect-or-bundle decision.

## Per-OS VM providers

| OS | Provider options | Notes |
|---|---|---|
| Linux | none (youki native) | no VM; youki runs directly |
| macOS | Lima, or Apple `Virtualization.framework` directly | Lima wraps Virtualization.framework + gives a CLI; direct is lower-level but no dependency |
| Windows | WSL2, or Hyper-V | WSL2 is the path of least resistance if present |

The provider is an abstraction: **boot a Linux VM, ensure
youki + the sidecar are inside, expose the sidecar's v1 endpoint,
tear down on reap.** Same `Isolator`-shaped contract as every other
runtime, with a VM lifecycle wrapped around it.

## How this plugs into the existing spine (the good news)

The sidecar architecture already solved the hard part — agent
control is substrate-agnostic. The VM substrate reuses it:

- The sidecar runs **inside the VM's container**, same as in a local
  container. The v1 protocol does not change.
- Only the **transport** changes: instead of localhost TCP to a
  published port, it's **vsock** (VM ↔ host) or a forwarded port.
  `RuntimeEndpoint::Remote { addr, scheme }` already accommodates
  this — the dispatcher + SidecarActorRuntime consume an endpoint
  and don't care whether it's a local container or a VM.
- `SandboxOwner` teardown, the reaper, the registry — all unchanged.
  Reap drops the owner → the VM-provider's shutdown tears down the
  VM (or the container inside it).

So the VM substrate is a **new Isolator + VM-provider abstraction**,
not a new control plane. The protocol, dispatcher, runtime trait,
and lifecycle are reused.

## What's built vs. not

**Built (Linux-native youki):**
- `youki.rs` (one-shot) + `youki_sidecar.rs` (long-lived, sidecar-
  driven over v1). These assume a **Linux host with youki present**.

**Not built (the primary path):**
- The **VM-provider abstraction** (Lima / Apple Virtualization /
  WSL2 / Hyper-V).
- VM lifecycle: provision, boot, health, teardown.
- A **Linux rootfs / VM image** carrying youki + the sidecar +
  (optionally) harness CLIs + common MCP servers.
- **vsock / forwarded-port transport** wiring into the sidecar
  client (`RuntimeEndpoint::Remote` with a vsock scheme).
- Recommender extension: VM-provider detection + detect-or-bundle.
- One-time VM-image fetch + caching.

This is the biggest unbuilt piece and it is the **main use case**,
not a frontier. It was deferred during implementation because the
tractable pieces (spine + docker adaptor + youki-on-Linux) landed
first; the VM substrate is the genuinely hard, genuinely important
half.

## Strategic framing

- **youki + Ship VM** = the self-contained, zero-install isolation
  for non-technical users. **The primary path. The S-tier feature.**
- **docker / podman** = detected adaptors for developers who already
  have a runtime. Convenience, not the foundation.
- **rootless everywhere** = a container escape lands as an
  unprivileged user, not root. The security ceiling (acceptable;
  Tier 3 separate-kernel is deferred, cloud Tier 4 is coming).

On macOS/Windows the VM gives a kernel boundary "for free" — the
agent's container shares the *VM's* Linux kernel, not the host
kernel — so the non-Linux experience is arguably stronger isolation
than Linux's pure shared-kernel rootless.

## Open questions for the build spec

- **Lima vs direct Virtualization.framework on macOS** — dependency
  vs. lower-level control / smaller footprint.
- **WSL2 vs Hyper-V on Windows** — WSL2 if present; Hyper-V as the
  bundled fallback?
- **VM image distribution** — built + signed + version-pinned, same
  supply-chain discipline as the agent image. One image with the
  rootfs, or layered?
- **vsock vs forwarded port** for the sidecar transport across the
  VM boundary — vsock is cleaner but provider-dependent.
- **VM sharing granularity** — one VM per project (N containers
  inside), or one VM total (N projects' containers inside)? The pod
  model says project-scoped sandboxes; the VM could host multiple.
- **Cold-start latency** — VM boot + youki container start. Warm
  pool? Keep the VM alive across idle, reap only the inner
  containers?
- **Workspace into the VM** — bind-mount across the VM boundary
  (Lima/WSL support host mounts) vs. the cloud work-unit model. For
  local VMs, host-mount preserves the hot-reload happy path.

## One-line intent

Ship bundles youki + a Linux VM so any user on any OS gets
zero-install rootless container isolation; docker/podman are
detected adaptors for those who already have them; the sidecar
spine is reused unchanged, with only the transport (vsock /
forwarded port) differing across the VM boundary.
