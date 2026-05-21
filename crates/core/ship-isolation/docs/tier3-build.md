# Tier 3 — build tracker

The VM-substrate tier. Providers are `Isolator` impls (same contract as
Docker): `prepare` boots a VM/microVM running `ship-sidecar`, returns a
`RuntimeEndpoint::Remote` (vsock); `shutdown` reaps it. The sidecar is
constant — only the substrate boot + transport differ per provider.

**Providers (decided 2026-05):**
- **Firecracker** (Linux, KVM) — microVM per sandbox, own kernel. Strongest;
  also the cloud (Fly) model. *Build+test locally once KVM is accessible.*
- **Apple Container** (mac, Apple Silicon + macOS 15+) — microVM-per-container,
  own kernel. T3, stronger inter-actor than Lima. *Contacts test.*
- **Lima** (mac, Intel/older) — one shared VM + bubblewrap inside. *Intel-mac.*
- **WSL** (win 10/11) — Ship distro hosts the sidecar; inclusive (not Hyper-V).

## Build sequence (Firecracker first — it's the locally-testable leg)

1. **vsock transport** ⬜ — sidecar `--addr vsock:<port>` (bind `VsockListener`);
   connectrpc client dials via `lazy_with_connector` + a vsock connector
   (mirror the `lazy_unix` work). `RuntimeEndpoint::Remote { scheme: vsock }`.
   *Untestable without a VM — lands with the Firecracker Isolator.*
2. **Guest kernel** ⬜ — a firecracker-compatible `vmlinux` (download from the
   firecracker CI bucket, pin the version). Staged under `target/fc/` for dev;
   the shipped path bakes/pins it.
3. **Rootfs + init** ⬜ — minimal ext4 with `ship-sidecar` as init (PID 1),
   launching it on vsock. Build rootlessly (`mke2fs -d <dir>` — no mount/root).
4. **Firecracker `Isolator`** ⬜ — `prepare`: configure the microVM (kernel +
   rootfs + vsock device) via the firecracker API/config, boot, poll the
   sidecar handshake over vsock, return the endpoint. `shutdown`: SIGKILL the
   firecracker process. Model on `youki_sidecar` (the existing sidecar-substrate
   Isolator) + the Docker backend's sidecar mode.
5. **Recommender ladder** ⬜ — add Firecracker/AppleContainer/Lima/WSL detection
   + per-OS preference. **Land each pick WITH its backend** (no recommend-without-
   driver). Also: enforce-path must REFUSE an unwired strategy (loud-fail), not
   silently run unwrapped.
6. **Per-OS Isolators** ⬜ — Apple Container (`container run`), Lima (`limactl`),
   WSL (`wsl --import`), each against the same seam. Validated on hardware.

## Open issues / not done

- **[BLOCKER] KVM access for this dev box** — `/dev/kvm` is `root:kvm 0660`;
  the dev user isn't in `kvm`. Unblock: `sudo setfacl -m u:$USER:rw /dev/kvm`
  (immediate) or `sudo usermod -aG kvm $USER` (relogin). Until then, NO
  Firecracker code can be live-tested → none is committed (no untested stubs).
- **Guest kernel + rootfs not yet staged** — kernel download + rootless rootfs
  bake are the next concrete steps (KVM-independent to *build*, KVM to *boot*).
- ~~**vsock on WSL2**~~ ✅ **viable** — `/dev/vhost-vsock` present, `/dev/vsock`
  is world-rw, AF_VSOCK sockets are creatable. Transport = **vsock** (no
  forwarded-port fallback needed). NB `/dev/vhost-vsock` is `root:kvm 0660`,
  so the host side of vsock is gated by the same kvm-group access as KVM.
- **Rootfs: static sidecar vs glibc assembly** — the rootfs needs `ship-sidecar`
  + its deps. A **static-musl** sidecar makes the rootfs trivial (one binary, no
  libc/loader to assemble) AND erases the glibc bind-mount caveat the container
  cells carry. Recommend building `ship-sidecar` for
  `x86_64-unknown-linux-musl`; verify the dep tree (portable-pty, libc,
  connectrpc) builds on musl first.
- **VM-sharing granularity** — one VM for all sandboxes vs per-project vs
  per-actor microVM. Firecracker/Apple Container favor per-sandbox microVM;
  Lima/WSL favor one shared VM. Decide per-provider during the build.
- **Cold-start budget** — microVM boot latency; may need a warm pool. Measure
  once booting works.
- **Baked image signing/pinning** — the kernel + rootfs (and the Apple Container
  agent image) must be version-pinned + signed before shipping (Phase F).
- **enforce-path refuse-on-unwired** — today `process.rs`'s `other =>` branch
  warns + runs unwrapped; with `enforce=true` it must refuse (threat-model §6
  loud-fail). Fix before the recommender can pick T3 strategies.

## Dev-box unblock (two sudo one-liners, then I build+test Firecracker)

Paste in your session — both are needed before any Firecracker code is
live-testable here:

```
! sudo setfacl -m u:$USER:rw /dev/kvm /dev/vhost-vsock   # boot + vsock
! sudo apt-get install -y musl-tools                      # static-musl sidecar → trivial rootfs
```

`musl` rust target adds without sudo (`rustup target add
x86_64-unknown-linux-musl`); `mke2fs` is present so the rootfs bakes
rootlessly (`mke2fs -d`). Alternative if you'd rather not install musl-tools:
a glibc rootfs (assemble libc + loader) — more work, avoid if possible.

## Test gates (who validates what)

| Leg | Substrate | Tested by |
|---|---|---|
| Firecracker | Linux/KVM | dev box (once KVM granted) |
| Apple Container | mac 15+/Apple Silicon | newer-mac contacts |
| Lima | Intel/older mac | maintainer's Intel mac |
| WSL | win 10/11 | maintainer's Windows |

## 🎯 Firecracker leg PROVEN end-to-end (spike, on this KVM box)

A Firecracker microVM boots, runs `ship-sidecar` as init, binds vsock, and the
host reaches it over vsock and gets **`HTTP/1.1 200 OK`, body `0.1.0`** from the
sidecar's Connect server. Reproducible. The whole VM-substrate model works:
boot → static-musl sidecar-as-init → rootless rootfs → vsock → host↔guest
Connect. The Firecracker `Isolator` is now a *transcription* of this recipe.

**The proven recipe (what the Isolator must encode):**
1. **static-musl sidecar** — `cargo build -p ship-sidecar --target
   x86_64-unknown-linux-musl` (`CC_x86_64_unknown_linux_musl=musl-gcc`),
   stripped (~11M). Static-pie, no libc in the rootfs.
2. **rootfs** (rootless `mke2fs -d <dir> rootfs.ext4 256M`): `/usr/bin/ship-sidecar`,
   `/bin/busybox` (+ `sh` symlink), an `/init` that mounts proc/sys/devtmpfs,
   sets a hostname, then `exec /usr/bin/ship-sidecar --addr vsock:2468`, AND
   **`/etc/hosts` + `/etc/resolv.conf`** (REQUIRED — without them musl startup
   name-resolution fails `EAI_AGAIN: Try again` and the sidecar exits → kernel
   panic; this cost real debugging time, don't drop it).
3. **firecracker config**: kernel (`vmlinux`), rootfs as `/dev/vda` root,
   `vsock { guest_cid: 3, uds_path }`, boot args `console=ttyS0 reboot=k
   panic=1 pci=off root=/dev/vda rw init=/init`, 1 vcpu / 256 MiB.
4. **host client transport** (the new connector): connect to the firecracker
   vsock **uds**, send `CONNECT <port>\n`, read `OK <n>\n`, *then* speak Connect
   over that stream. So the host-side connectrpc client uses
   `Http2Connection::lazy_with_connector` with a connector that opens the uds +
   does the CONNECT handshake (NOT raw AF_VSOCK — firecracker bridges via the uds).

## Status

- ✅ Providers decided; firecracker `v1.15.1` + kernel `vmlinux-6.1.102` staged.
- ✅ vsock viable; sidecar `--addr vsock:` committed (`534550135`).
- ✅ **Firecracker microVM + sidecar + vsock + host↔guest Connect 200 OK —
  proven end-to-end on KVM.**
- ✅ **Firecracker `Isolator` landed + live-tested** (`backends/firecracker.rs`,
  `firecracker_live`): `prepare` copies the rootfs per-VM, writes the fc config,
  boots firecracker, polls the sidecar over the vsock uds; `shutdown` SIGKILLs +
  cleans up. The host uds-CONNECT connector is wired into `SidecarActorRuntime`
  (`fcvsock:<port>:<uds>` endpoint). An actor runs inside the microVM, exits 0.
  TCP/unix/podman transports unregressed.
- ⬜ Then: rootfs/image baking as real tooling (vs the dev spike), pinned +
  signed; recommender ladder (with backend); per-OS legs (Apple Container/Lima/
  WSL on your hardware).

## Open issues

- ~~**Host uds-CONNECT connector**~~ ✅ wired (`SidecarActorRuntime`, `fcvsock:`).
- **Workspace into the microVM** — the actor runs but has **no workspace**, and
  **Firecracker has NO virtio-fs** (confirmed: device model is drives/vsock/net
  only, by design) — so no live host-fs mount. The options:
  1. **data-drive snapshot** — `mke2fs -d <workspace>` → attach as a 2nd drive →
     guest mounts it; extract back at shutdown via `debugfs rdump` (rootless).
     Self-contained, testable here; snapshot (not live) semantics.
  2. **git/work-unit copy** — agent clones in + pushes out over a net device
     (needs firecracker networking; cloud-aligned).
  3. **architectural split (recommended):** **live-FS is the Lima/WSL/Apple-
     Container tier** (shared dir / virtio-fs on mac/win); **Firecracker is the
     strong *ephemeral* tier** — work-unit copy (option 1 for local, option 2
     for cloud). The microVM too isolated for a live mount is exactly the one
     you pick when you want that isolation. Reconciles substrate.md's
     "live FS is Tier-≤3 / Tier-4 is work-unit copy": Firecracker behaves like
     the cloud tier on the fs axis.
  ✅ **DONE (option 3, data-drive work-unit copy).** `firecracker_live`'s
  `microvm_workspace_round_trips`: a host dir is seeded into the microVM
  (`mke2fs -d` → `/dev/vdb`), the guest `/init` mounts it at the
  `ship.workspace=` cmdline path (+ `busybox --install -s` for a real
  userland), the agent reads a seeded file + writes a new one, and the new
  file copies back out via `debugfs rdump` at shutdown — **no agent `sync`
  required**. Flush mechanism: Firecracker buffers guest block writes until a
  virtio flush, so the guest runs a 1s periodic `sync` and the Isolator waits
  one interval before the hard stop.
  *Refinements:* deletions don't propagate via rdump; the periodic-sync +
  flush-wait is a v1 — a cooperative **ACPI graceful shutdown** (firecracker
  API `SendCtrlAltDel` + an init that syncs/unmounts on signal) is the clean
  follow-up; dynamic data-image sizing (fixed 512M today).
- **rootfs/image as tooling** — the rootfs is baked ad hoc by the dev spike;
  productionize as a pinned, signed image (Phase F) + a baking step the
  Isolator/CI runs (currently `FirecrackerConfig.rootfs` is a supplied path).
- **per-sandbox vs shared microVM + cold-start** — today one microVM per
  `prepare` (per-VM rootfs copy). Cheap boot, but a warm pool may be wanted;
  measure.
- **enforce-refuse-on-unwired** still required before the recommender picks T3.
- **Recommender** doesn't pick Firecracker yet (land with the ladder, gated by
  `available()` + the enforce-refuse fix).
