#!/usr/bin/env bash
# Build the Ship agent-runner Tier-2 container image — a base with the
# ship-sidecar baked in, so the Docker/podman Isolator runs a real Ship image
# instead of stock ubuntu + a bind-mounted binary. This is what
# `isolation.strategies.docker.image` points at.
#
# The sidecar is built static-musl so it runs on ANY base regardless of the
# image's libc (the wire is the contract, not the binary format) — this is
# what erases the glibc-ABI / bind-mount caveat.
#
# DEV builder. Phase F signs + version-pins + publishes ghcr.io/ship/agent-runner.
#
# Usage: scripts/build-agent-image.sh [tag]   (default: ship/agent-runner:dev)
#        SHIP_CONTAINER_ENGINE=docker scripts/build-agent-image.sh
set -euo pipefail

cd "$(dirname "$0")/.."
ENGINE="${SHIP_CONTAINER_ENGINE:-podman}"
TAG="${1:-ship/agent-runner:dev}"

echo "==> static-musl ship-sidecar (ship-exec)"
# ship-sidecar lives in ship-exec (j-09); sibling checkout required.
SHIP_EXEC="${SHIP_EXEC:-$(cd ../ship-infra && pwd)}"
rustup target add x86_64-unknown-linux-musl >/dev/null 2>&1 || true
CC_x86_64_unknown_linux_musl=musl-gcc \
  cargo build -p ship-sidecar --target x86_64-unknown-linux-musl --manifest-path "$SHIP_EXEC/Cargo.toml"
SIDECAR="$SHIP_EXEC/target/x86_64-unknown-linux-musl/debug/ship-sidecar"

echo "==> assemble build context"
CTX="$(mktemp -d)"
trap 'rm -rf "$CTX"' EXIT
cp "$SIDECAR" "$CTX/ship-sidecar"
strip "$CTX/ship-sidecar" 2>/dev/null || true
cat > "$CTX/Dockerfile" <<'DOCKERFILE'
# Ship agent-runner (Tier 2). The supervisor drives actors over
# ship.sidecar.v1; the sidecar is PID 1 / the container command. No ENTRYPOINT
# — the Isolator supplies the sidecar invocation (control-plane socket, etc.),
# so the image just needs the binary present.
FROM ubuntu:24.04
COPY ship-sidecar /usr/local/bin/ship-sidecar
RUN chmod +x /usr/local/bin/ship-sidecar
DOCKERFILE

echo "==> build with $ENGINE"
"$ENGINE" build -t "$TAG" "$CTX"

echo "==> done. $TAG"
echo "    point isolation.strategies.docker.image at \"$TAG\" to use it"
