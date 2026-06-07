#!/usr/bin/env bash
# Build the Tier-3 Firecracker dev assets that `firecracker_live` consumes:
# the firecracker binary, a guest kernel, and a rootfs (ext4) running
# `ship-sidecar` as init over vsock. Reproducible + rootless — no sudo to
# *build* (KVM access is only needed to *boot*, i.e. to run the test).
#
# Output: target/fc/{release-*/firecracker-*, vmlinux, busybox, rootfs.ext4}.
# Re-run after changing the sidecar. See crates/core/ship-isolation/docs/
# tier3-build.md for the design + the recipe this encodes.
#
# This is a DEV asset builder, not the shipped image path — Phase F bakes a
# pinned/signed image. Don't depend on the exact versions here for prod.
set -euo pipefail

FC_VERSION="v1.15.1"
KERNEL_URL="https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.10/x86_64/vmlinux-6.1.102"
BUSYBOX_URL="https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox"
ARCH="$(uname -m)"

cd "$(dirname "$0")/.."
OUT="target/fc"
mkdir -p "$OUT"

echo "==> firecracker $FC_VERSION"
FC_BIN="$OUT/release-${FC_VERSION}-${ARCH}/firecracker-${FC_VERSION}-${ARCH}"
if [ ! -f "$FC_BIN" ]; then
  curl -sSL "https://github.com/firecracker-microvm/firecracker/releases/download/${FC_VERSION}/firecracker-${FC_VERSION}-${ARCH}.tgz" \
    | tar -xz -C "$OUT"
fi

echo "==> guest kernel"
[ -f "$OUT/vmlinux" ] || curl -sSL -o "$OUT/vmlinux" "$KERNEL_URL"

echo "==> static busybox"
if [ ! -f "$OUT/busybox" ]; then
  curl -sSL -o "$OUT/busybox" "$BUSYBOX_URL"
  chmod +x "$OUT/busybox"
fi

echo "==> static-musl ship-sidecar"
rustup target add x86_64-unknown-linux-musl >/dev/null 2>&1 || true
CC_x86_64_unknown_linux_musl=musl-gcc \
  cargo build -p ship-sidecar --target x86_64-unknown-linux-musl
SIDECAR="target/x86_64-unknown-linux-musl/debug/ship-sidecar"

echo "==> assemble rootfs"
R="$OUT/rootfs-src"
rm -rf "$R"
mkdir -p "$R"/{bin,usr/bin,dev,proc,sys,run,tmp,etc}
cp "$SIDECAR" "$R/usr/bin/ship-sidecar"
strip "$R/usr/bin/ship-sidecar" 2>/dev/null || true
cp "$OUT/busybox" "$R/bin/busybox"
ln -sf busybox "$R/bin/sh"
# /etc/hosts + resolv.conf: musl startup name-resolution fails EAI_AGAIN
# without them (the sidecar exits → kernel panic). Don't drop these.
printf '127.0.0.1 localhost\n::1 localhost\n127.0.1.1 ship-sandbox\n' > "$R/etc/hosts"
printf 'hosts: files dns\n' > "$R/etc/nsswitch.conf"
: > "$R/etc/resolv.conf"

# init: PID 1. Mounts pseudo-fs, installs the busybox userland, brings lo up
# (a no-network microVM leaves loopback down → 127.0.0.1 unreachable → the
# app-plane bridge can't be dialed), mounts the workspace data drive at the
# ship.workspace= cmdline path, runs a periodic sync (firecracker buffers
# block writes until a virtio flush), then execs the sidecar with the control
# plane (vsock:2468) + the app-plane bridge (127.0.0.1:8088 → vsock host:2469).
cat > "$R/init" <<'INIT'
#!/bin/sh
/bin/busybox mount -t proc proc /proc 2>/dev/null
/bin/busybox mount -t sysfs sys /sys 2>/dev/null
/bin/busybox mount -t devtmpfs dev /dev 2>/dev/null
/bin/busybox --install -s /bin 2>/dev/null
/bin/busybox hostname ship-sandbox 2>/dev/null
/bin/busybox ip link set lo up 2>/dev/null
WS=$(/bin/busybox sed -n 's/.*ship\.workspace=\([^ ]*\).*/\1/p' /proc/cmdline)
if [ -n "$WS" ] && [ -b /dev/vdb ]; then
  /bin/busybox mkdir -p "$WS"
  /bin/busybox mount -t ext4 -o sync /dev/vdb "$WS" && echo "ship-init: workspace at $WS" || echo "ship-init: vdb mount FAILED"
fi
( while /bin/busybox true; do /bin/busybox sync; /bin/busybox sleep 1; done ) &
exec /usr/bin/ship-sidecar --addr vsock:2468 --app-plane 8088:2469
INIT
chmod +x "$R/init"

echo "==> bake ext4 (rootless)"
rm -f "$OUT/rootfs.ext4"
mke2fs -q -t ext4 -d "$R" "$OUT/rootfs.ext4" 256M

echo "==> done. Assets under $OUT/ — run: cargo test -p ship-isolation --test firecracker_live"
echo "    (booting needs /dev/kvm + /dev/vhost-vsock accessible to your user)"
