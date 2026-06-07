#!/usr/bin/env bash
# Build a Ubuntu-base rootfs with bun + Claude Code preinstalled.
# Output: a directory tree the youki backend can mount as `root.path`.
#
# Usage:
#   build_agent_rootfs.sh <output-dir>
#
# Requirements on the host:
#   - curl
#   - tar
#   - bwrap (bubblewrap) — for rootless package install
#
# Idempotent: if <output-dir> already contains `opt/bun/bin/claude`, exits 0.

set -euo pipefail

OUT="${1:-${SHIP_AGENT_ROOTFS:-/tmp/ship-rootfs/rootfs}}"
UBUNTU_TARBALL_URL="${UBUNTU_TARBALL_URL:-https://cdimage.ubuntu.com/ubuntu-base/releases/24.04/release/ubuntu-base-24.04.3-base-amd64.tar.gz}"

mkdir -p "$OUT"

if [ -x "$OUT/opt/bun/bin/claude" ]; then
  echo "rootfs at $OUT already populated; skipping build"
  exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "==> downloading Ubuntu base"
curl -fsSL -o "$WORK/ubuntu.tar.gz" "$UBUNTU_TARBALL_URL"

echo "==> extracting rootfs to $OUT"
tar xzf "$WORK/ubuntu.tar.gz" -C "$OUT"

echo "==> installing system deps + bun + claude-code (via bwrap)"
bwrap --uid 0 --gid 0 \
  --bind "$OUT" / \
  --proc /proc --dev /dev --tmpfs /tmp \
  --ro-bind /etc/resolv.conf /etc/resolv.conf \
  --ro-bind /etc/hosts /etc/hosts \
  --share-net \
  --chdir / \
  --setenv DEBIAN_FRONTEND noninteractive \
  --setenv PATH /opt/bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  --setenv HOME /root \
  --setenv BUN_INSTALL /opt/bun \
  /bin/bash -euxo pipefail -c '
    echo "APT::Sandbox::User \"root\";" > /etc/apt/apt.conf.d/00sandbox
    apt-get update -qq
    apt-get install -y --no-install-recommends \
      ca-certificates curl unzip git openssl
    rm -rf /var/lib/apt/lists/*

    curl -fsSL https://bun.sh/install | bash
    /opt/bun/bin/bun --version
    /opt/bun/bin/bun add -g @anthropic-ai/claude-code
    /opt/bun/bin/claude --version

    # Pre-create mount points so the isolator can bind-mount into them
    mkdir -p /root/.claude /work
    : > /root/.claude/.credentials.json
  '

echo "==> done. rootfs ready at $OUT"
echo "    /opt/bun/bin/claude → $($OUT/opt/bun/bin/claude --version 2>/dev/null || echo unknown)"
