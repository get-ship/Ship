# macOS substrate verification (Lima + Apple Container)

Both are **drafts pending real-hardware verification** — they compile and the
live cells skip cleanly off-hardware. A Mac has no Linux kernel to run our
Isolators in; Lima and Apple Container each *provide* one. The sidecar + tiers
then run inside exactly as on Linux/WSL.

## Lima (any Mac — Intel or Apple Silicon)

The launch + reap mechanics were proven on a Linux/QEMU POC; the host↔guest
**transport** is the part that needs a real Mac (a nested-virt box's Lima
port-forwarding is unreliable regardless of config — hence the explicit
`portForwards` in the template).

```sh
# 1. The Lima VM is Linux — build a Linux-musl sidecar on the host:
rustup target add x86_64-unknown-linux-musl        # Intel Mac
#   (aarch64-unknown-linux-musl on Apple Silicon)
cargo build -p ship-sidecar --target x86_64-unknown-linux-musl

# 2. Start the substrate instance (declares the portForwards the transport needs):
limactl start --name ship-default contrib/lima/ship-substrate.yaml

# 3. Drive it end to end:
cargo test -p ship-isolation --test lima_sidecar_live -- --nocapture
```

Lima mounts your home dir into the VM, so the host-built sidecar is reachable
at its host path (the cell uses that as `sidecar_path`). If green, Lima is
verified; flip nothing else — it's already wired into the recommender + spawn
path (mac VM tier).

## Apple Container (Apple Silicon, macOS 26+)

`has_impl(AppleContainer)` is intentionally **false** — the backend is a draft,
so it's *not* in the production spawn path yet. The live cell constructs it
directly, so your friend can verify it in isolation.

```sh
# 1. Build an arm64 image with ship-sidecar baked in. This compiles the
#    sidecar INSIDE a Linux build stage, so you don't cross-compile a Linux
#    binary from macOS:
container build -t ship/agent-runner:dev -f contrib/apple/agent-runner.Dockerfile .

# 2. Drive it (Rust must be installed on the host to run the test harness):
SHIP_APPLE_IMAGE=ship/agent-runner:dev \
  cargo test -p ship-isolation --test apple_container_live -- --nocapture
```

**The one bit to verify/adjust:** `backends/apple_container.rs::ip_from_inspect`
parses the container's IP out of `container inspect` JSON. The exact shape is
unverified — run `container inspect <name>` by hand, check which field holds
the address, and adjust the `candidates` paths (there's a unit test pinning the
assumed shapes). The `run`/`inspect`/`stop`/`delete` verbs are Docker-shaped
and should be right. Once green, a follow-up flips `has_impl` true + adds the
spawn arm (recommender already lists it as a mac-arm VM candidate).

## When both are green

The grid is all-✅ across Linux/mac/Windows, and isolation can be parked. The
deliverable layer that remains is packaging (Phase F: pinned/signed Firecracker
rootfs + WSL substrate + agent-runner image + Lima image) and CI for the live
cells on substrate-equipped runners.
