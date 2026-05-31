#!/usr/bin/env bash
# Build the Ship WSL substrate rootfs tarball — the Windows Tier-strategy
# substrate. Import it as a dedicated distro and the `Wsl` Isolator
# (backends/wsl.rs) launches `ship-sidecar` inside it; native-Windows shipd
# drives the actor over WSL2's localhost forwarding.
#
# Output: target/wsl/ship-substrate.tar (a WSL2 rootfs: busybox userland +
# static-musl ship-sidecar + minimal /etc). Verified by the wsl_sidecar_live
# POC recipe (see crates/core/ship-isolation/src/backends/wsl.rs).
#
# Import on Windows (or via interop):
#   wsl.exe --import ship-substrate C:\ship\substrate target\wsl\ship-substrate.tar --version 2
#
# DEV asset builder, not the shipped image — Phase F bakes a pinned/signed
# rootfs. Don't depend on the exact busybox version here for prod.
set -euo pipefail

BUSYBOX_URL="https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox"

cd "$(dirname "$0")/.."
OUT="target/wsl"
mkdir -p "$OUT"

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
mkdir -p "$R"/{bin,usr/bin,dev,proc,sys,run,tmp,etc,root}
cp "$SIDECAR" "$R/usr/bin/ship-sidecar"
strip "$R/usr/bin/ship-sidecar" 2>/dev/null || true
cp "$OUT/busybox" "$R/bin/busybox"
ln -sf busybox "$R/bin/sh"
# /etc/passwd: WSL does getpwuid(0) on launch; without it every command prints
# `getpwuid(0) failed`. Harmless but noisy — give it a root entry.
printf 'root:x:0:0:root:/root:/bin/sh\n' > "$R/etc/passwd"
# /etc/wsl.conf: this is a headless substrate, not an interactive shell —
# disable Windows drive automount + interop so boot is quiet and fast (the
# noisy /mnt + PATH-translation failures the POC surfaced).
printf '[automount]\nenabled=false\n[interop]\nenabled=false\nappendWindowsPath=false\n' > "$R/etc/wsl.conf"
# musl name resolution needs these (EAI_AGAIN otherwise).
printf '127.0.0.1 localhost\n::1 localhost\n' > "$R/etc/hosts"
printf 'hosts: files dns\n' > "$R/etc/nsswitch.conf"
: > "$R/etc/resolv.conf"
# empty fstab so WSL's boot-time `mount -a` is a clean no-op (else it logs a
# harmless "Processing /etc/fstab with mount -a failed").
: > "$R/etc/fstab"

echo "==> pack tarball (WSL --import format)"
rm -f "$OUT/ship-substrate.tar"
tar -C "$R" -cf "$OUT/ship-substrate.tar" .

echo "==> done. $OUT/ship-substrate.tar"
echo "    import:    wsl.exe --import ship-substrate C:\\ship\\substrate \\"
echo "               $(pwd)/$OUT/ship-substrate.tar --version 2"
echo "    then run:  SHIP_WSL_TEST_DISTRO=ship-substrate cargo test -p ship-isolation \\"
echo "               --test wsl_sidecar_live   (on Windows — forwarding is host<->distro)"
