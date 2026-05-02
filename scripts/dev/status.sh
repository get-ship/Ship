#!/usr/bin/env bash
# List every running daemon: workspace, port, pid, sha, alignment.
#
# Walks ~/.ship/machine/daemon (singleton) plus ~/.ship/machine/daemons/*/
# (per-worktree, devx-04). Each row shows whether the recorded pid is alive
# (RUNNING), whether the build SHA matches this checkout's HEAD (ALIGNED),
# or just metadata when no pid is recorded (IDLE).
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

home="${SHIP_HOME:-$HOME/.ship}"
machine="$home/machine"

print_row() {
    local d="$1"
    local label="$2"
    local pid port sha workspace state align this_sha
    pid=$( [ -f "$d/pid" ]       && cat "$d/pid"       || echo "" )
    port=$( [ -f "$d/port" ]     && cat "$d/port"      || echo "" )
    sha=$(  [ -f "$d/sha" ]      && cat "$d/sha"       || echo "" )
    workspace=$( [ -f "$d/workspace" ] && cat "$d/workspace" || echo "" )

    # Workspace recorded but directory gone → orphan daemon, reapable.
    if [ -n "$workspace" ] && [ ! -d "$workspace" ]; then
        state="REAPABLE"
    elif [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        state="RUNNING"
    elif [ -n "$pid" ]; then
        state="STALE"
    elif [ -d "$d/.bin" ]; then
        state="IDLE"
    else
        return 0  # nothing interesting here
    fi

    this_sha=$(build_sha)
    if [ -n "$sha" ] && [ -n "$this_sha" ] && [[ "$sha" == "$this_sha"* ]]; then
        align="aligned"
    elif [ -n "$sha" ]; then
        align="skew→$this_sha"
    else
        align="-"
    fi

    printf '%-8s %-7s %-6s %-12s %-10s %s\n' \
        "$state" "${port:--}" "${pid:--}" "${sha:--}" "$align" "${workspace:-$label}"
}

printf '%-8s %-7s %-6s %-12s %-10s %s\n' \
    "STATE" "PORT" "PID" "SHA" "ALIGN" "WORKSPACE"

# Singleton
[ -d "$machine/daemon" ] && print_row "$machine/daemon" "(global)"

# Per-worktree
if [ -d "$machine/daemons" ]; then
    for d in "$machine/daemons"/*/; do
        [ -d "$d" ] || continue
        print_row "${d%/}" "$(basename "${d%/}")"
    done
fi
