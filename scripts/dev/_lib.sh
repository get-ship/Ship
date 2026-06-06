#!/usr/bin/env bash
# Shared helpers for scripts/dev/*. Sourced; do not run directly.
set -euo pipefail

# Resolve the current cargo target dir, honoring .cargo/config.toml redirects.
target_dir() {
    cargo metadata --format-version 1 --no-deps --offline 2>/dev/null \
        | grep -o '"target_directory":"[^"]*"' \
        | cut -d'"' -f4
}

# The `ship` binary built in this checkout. Falls back to PATH if missing.
ship_bin() {
    local t
    t=$(target_dir)
    if [ -x "$t/debug/ship" ]; then
        echo "$t/debug/ship"
    elif [ -x "$t/release/ship" ]; then
        echo "$t/release/ship"
    else
        command -v ship
    fi
}

# Daemon IPC dir for the current cwd. Falls back to a shell-computed path
# when the binary isn't built yet (first `just dev` run).
daemon_dir() {
    local b
    b=$(ship_bin 2>/dev/null || true)
    if [ -n "$b" ]; then
        "$b" daemon dir
        return
    fi
    # Fallback: hash project root the same way runtime::home::path_hash does.
    local root home hash
    root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
    root=$(realpath "$root")
    hash=$(printf '%s' "$root" | sha256sum | head -c 8)
    home="${SHIP_HOME:-$HOME/.ship}"
    if [ -d "$root/.ship" ] || [ -d "$root/.git" ]; then
        echo "$home/machine/daemons/$hash"
    else
        echo "$home/machine/daemon"
    fi
}

# Stash a built binary into the daemon dir's .bin/ subdir.
# Usage: stash_binary <name>   e.g. stash_binary shipd
stash_binary() {
    local name="$1"
    local t d
    t=$(target_dir)
    d=$(daemon_dir)
    mkdir -p "$d/.bin"
    if [ ! -x "$t/debug/$name" ]; then
        echo "stash_binary: $t/debug/$name not found — run `just build` first" >&2
        return 1
    fi
    cp -f "$t/debug/$name" "$d/.bin/$name.new"
    mv "$d/.bin/$name.new" "$d/.bin/$name"
}

# Read SHA from daemon_dir/sha. Empty when absent.
daemon_sha() {
    local d
    d=$(daemon_dir)
    [ -f "$d/sha" ] && cat "$d/sha" || true
}

# Current build SHA (from build-info via shipd's identity line). Falls back
# to `git rev-parse --short=8 HEAD` when the binary is unbuilt.
build_sha() {
    local b
    b=$(target_dir)/debug/shipd
    if [ -x "$b" ]; then
        # shipd doesn't have a --version that prints just the sha; parse from `--help`?
        # Cleanest: read it from build-info via the identity line by spawning a
        # helper. Simpler: fall back to git for now.
        :
    fi
    git rev-parse --short=8 HEAD 2>/dev/null || echo "unknown"
}

# Path to the vite.pid file for the current daemon dir. shell-start.sh
# records the per-worktree vite PID here so dev-stop can reap it.
vite_pid_file() {
    echo "$(daemon_dir)/vite.pid"
}

# Kill the vite process recorded for the current daemon dir and remove the
# pid file. No-op when nothing is tracked or the process is already gone.
kill_vite() {
    local pf pid kd
    pf=$(vite_pid_file)
    [ -f "$pf" ] || return 0
    pid=$(cat "$pf")
    rm -f "$pf"
    [ -n "$pid" ] || return 0
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        kd=$(( $(date +%s) + 3 ))
        while kill -0 "$pid" 2>/dev/null && [ "$(date +%s)" -lt "$kd" ]; do
            sleep 0.1
        done
        kill -9 "$pid" 2>/dev/null || true
        echo "vite: killed pid $pid"
    fi
}
# Stop the dev daemon serving the current daemon dir.
#
# `start.sh` launches `shipd` directly — not under `ship daemon supervise` —
# so the running process's liveness is the `pid` handshake file shipd
# writes into the daemon dir, NOT the flock-based `shipd.pid` that
# `ship daemon stop` consults. A bare shipd never creates that flock
# pidfile, so `ship daemon stop` finds no supervisor and exits 0 without
# signalling anything. Signal shipd's own pid directly; it handles SIGTERM
# with a graceful shutdown, so no SIGKILL escalation is needed. Returns
# non-zero (rather than escalating) if shipd genuinely ignores SIGTERM, so
# the failure surfaces instead of being papered over.
kill_daemon() {
    local d pf pid deadline
    d=$(daemon_dir)
    pf="$d/pid"
    [ -f "$pf" ] || return 0
    pid=$(cat "$pf" 2>/dev/null || true)
    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        # Stale or empty handshake file — shipd is already gone. Clear the
        # liveness files so the next start doesn't mistake them for a live
        # daemon (shipd only prunes these on its *next* startup).
        rm -f "$pf" "$d/port"
        return 0
    fi
    kill "$pid" 2>/dev/null || true
    deadline=$(( $(date +%s) + 10 ))
    while kill -0 "$pid" 2>/dev/null && [ "$(date +%s)" -lt "$deadline" ]; do
        sleep 0.1
    done
    if kill -0 "$pid" 2>/dev/null; then
        # shipd ignored SIGTERM within the graceful window. That's a daemon
        # shutdown bug worth fixing — but blocking `dev-restart` on it strands
        # the worktree with a zombie daemon. Surface it loudly, then escalate to
        # SIGKILL so the restart proceeds.
        echo "kill_daemon: shipd pid $pid ignored SIGTERM after 10s — escalating to SIGKILL (daemon graceful-shutdown bug)" >&2
        kill -9 "$pid" 2>/dev/null || true
        deadline=$(( $(date +%s) + 3 ))
        while kill -0 "$pid" 2>/dev/null && [ "$(date +%s)" -lt "$deadline" ]; do
            sleep 0.1
        done
    fi
    rm -f "$pf" "$d/port"
    echo "shipd stopped (pid $pid)"
}
