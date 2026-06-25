# Agent-runner image for Apple Container (and any arm64 host) — built so you
# DON'T have to cross-compile a Linux binary from macOS: ship-sidecar is
# compiled INSIDE a Linux build stage, then baked into a slim runtime.
#
# On an Apple-Silicon Mac (from the repo root):
#   container build -t ship/agent-runner:dev -f contrib/apple/agent-runner.Dockerfile .
#
# (Identical for `docker build` / `podman build` — same Dockerfile.)
FROM docker.io/library/rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build -p ship-sidecar

FROM docker.io/library/debian:bookworm-slim
COPY --from=build /src/target/debug/ship-sidecar /usr/local/bin/ship-sidecar
# No ENTRYPOINT — the Isolator supplies the sidecar invocation (--addr …).
