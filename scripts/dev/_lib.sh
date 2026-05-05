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

# Path to the vite.pid file for the current daemon dir.
vite_pid_file() {
    echo "$(daemon_dir)/vite.pid"
}

# Kill the vite process recorded for the current daemon dir and remove the pid file.
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

# Walk all daemon dirs and reap vite processes whose cwd no longer exists.
# This catches vites left running after their worktree was deleted.
reap_orphan_vites() {
    local home machine
    home="${SHIP_HOME:-$HOME/.ship}"
    machine="$home/machine"
    [ -d "$machine/daemons" ] || return 0
    local dir vpf vpid vcwd kd
    for dir in "$machine/daemons"/*/; do
        [ -d "$dir" ] || continue
        vpf="${dir%/}/vite.pid"
        [ -f "$vpf" ] || continue
        vpid=$(cat "$vpf")
        if [ -z "$vpid" ]; then
            rm -f "$vpf"
            continue
        fi
        if ! kill -0 "$vpid" 2>/dev/null; then
            rm -f "$vpf"
            continue
        fi
        vcwd=$(readlink "/proc/$vpid/cwd" 2>/dev/null || true)
        if [ -z "$vcwd" ] || [ ! -d "$vcwd" ]; then
            kill "$vpid" 2>/dev/null || true
            kd=$(( $(date +%s) + 3 ))
            while kill -0 "$vpid" 2>/dev/null && [ "$(date +%s)" -lt "$kd" ]; do
                sleep 0.1
            done
            kill -9 "$vpid" 2>/dev/null || true
            rm -f "$vpf"
            echo "reap_orphan_vites: reaped vite $vpid (cwd gone: ${vcwd:-unknown})"
        fi
    done
}
