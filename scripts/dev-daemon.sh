#!/usr/bin/env bash
# dev-daemon.sh — point the running daemon at THIS worktree's binary + .ship/.
#
# WHEN TO USE
#   You're developing views/themes/CSS in a worktree and need the daemon on
#   :9315 to serve YOUR changes — not the installed `~/.local/bin/ship daemon`
#   binary, which serves older `include_str!`'d bytes.
#
# WHAT IT DOES
#   1. Detects + kills any existing daemon on :9315 (installed or worktree).
#   2. `cargo build -p shipd` from this worktree (skip with --no-build).
#   3. Spawns `target/debug/shipd --port 9315` in the background with
#      SHIP_DIR pointed at THIS worktree's .ship/ — so project resolution
#      doesn't redirect to the main checkout.
#   4. Waits for /api/health to return 200 and prints daemon identity.
#
# WHAT IT DOESN'T DO
#   - Doesn't survive shell exit. The daemon is `nohup`'d and writes to
#     /tmp/shipd-qa.log, but logging out may still kill it on some systems.
#     For long-lived runs use `ship daemon run` in a foreground terminal.
#   - Doesn't update the installed `~/.local/bin/ship` binary.
#   - Doesn't disable any user-level supervisors that may respawn the
#     installed daemon — it warns if one is detected.
#
# REVERTING
#   Run `ship daemon run` (or restart the installed supervisor) — the
#   installed binary takes :9315 again.
#
# FLAGS
#   --no-build       Skip `cargo build -p shipd`.
#   --status         Report whether the configured port is up and which binary serves.
#   --stop           Kill the worktree daemon. No rebuild, no restart.
#   --port <N>       Bind to port N instead of 9315. Default also reads SHIP_DAEMON_PORT.
#                    Pair with vite.config.ts which respects SHIP_DAEMON_PORT for proxy.
#   --keep-installed Leave a separately-running ship daemon (e.g. another worktree's)
#                    on a different port alone instead of warning about it.

set -euo pipefail

# Per-worktree slug for log paths so two worktrees don't clobber each other's logs.
ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
SLUG="$(basename "$ROOT")"
PORT="${SHIP_DAEMON_PORT:-9315}"
LOG="/tmp/shipd-${SLUG}.log"
SHIP_DIR="$ROOT/.ship"
# Resolve cargo target dir via metadata so a shared / redirected target
# (e.g. .cargo/config.toml setting target-dir for cross-worktree cache)
# still points at the right binary.
TARGET_DIR="$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps --offline 2>/dev/null | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)"
BIN="${TARGET_DIR:-$ROOT/target}/debug/shipd"

color() { printf '\033[%sm%s\033[0m' "$1" "$2"; }
green() { color '32' "$1"; }
yellow() { color '33' "$1"; }
red() { color '31' "$1"; }

# Find pid listening on :9315 (Linux /proc-based, no lsof dependency).
pid_on_port() {
    # ss is in iproute2 — present on every modern Linux/WSL2.
    ss -ltnp "sport = :$PORT" 2>/dev/null \
        | awk -F'pid=' 'NR>1 {split($2,a,","); print a[1]}' \
        | head -n1
}

# Resolve a pid to its executable path via /proc/<pid>/exe symlink.
exe_of() {
    local pid="$1"
    [ -n "$pid" ] || return 1
    readlink -f "/proc/$pid/exe" 2>/dev/null || true
}

ship_dir_of() {
    local pid="$1"
    [ -n "$pid" ] || return 1
    # SHIP_DIR is in /proc/<pid>/environ (NUL-separated).
    tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null \
        | awk -F= '$1=="SHIP_DIR"{print $2; exit}'
}

cmd_status() {
    local pid; pid="$(pid_on_port || true)"
    if [ -z "$pid" ]; then
        echo "$(yellow "not running") — nothing listening on :$PORT"
        return 0
    fi
    local exe; exe="$(exe_of "$pid" || echo unknown)"
    local sd; sd="$(ship_dir_of "$pid" || true)"
    echo "$(green "running") pid=$pid"
    echo "  exe:      $exe"
    echo "  SHIP_DIR: ${sd:-<unset>}"
    # Prefer the daemon's own answer for version/commit.
    if curl -fsS "http://127.0.0.1:$PORT/api/health" 2>/dev/null \
        | python3 -c 'import json,sys; d=json.load(sys.stdin); print(f"  version:  {d.get(\"version\",\"?\")}-{d.get(\"commit\",\"?\")}"); print(f"  ship_dir: {d.get(\"ship_dir\",\"?\")}"); print(f"  ok:       {d.get(\"ok\",False)}")' 2>/dev/null; then
        :
    fi
}

cmd_stop() {
    local pid; pid="$(pid_on_port || true)"
    if [ -z "$pid" ]; then
        echo "$(yellow "no daemon on :$PORT") — nothing to stop"
        return 0
    fi
    local exe; exe="$(exe_of "$pid" || echo unknown)"
    echo "stopping pid=$pid exe=$exe"
    kill -TERM "$pid" 2>/dev/null || true
    # Grace period.
    for _ in 1 2 3 4 5 6 7 8; do
        sleep 0.25
        kill -0 "$pid" 2>/dev/null || { echo "$(green stopped)"; return 0; }
    done
    kill -9 "$pid" 2>/dev/null || true
    sleep 0.25
    if kill -0 "$pid" 2>/dev/null; then
        echo "$(red "could not kill") pid=$pid"; return 1
    fi
    echo "$(green stopped) (SIGKILL)"
}

cmd_start() {
    local no_build="$1"

    # 1. Kill any existing daemon on :9315 (installed or worktree).
    local pid; pid="$(pid_on_port || true)"
    if [ -n "$pid" ]; then
        local exe; exe="$(exe_of "$pid" || echo unknown)"
        local sd; sd="$(ship_dir_of "$pid" || echo "<unset>")"
        echo "existing daemon: pid=$pid exe=$exe SHIP_DIR=$sd"
        cmd_stop
    fi

    # 2. Warn about likely respawn supervisors. We don't kill them — opt-in.
    if pgrep -af 'ship daemon|shipd ' 2>/dev/null \
        | grep -v "$$" | grep -v "$(basename "$0")" >/dev/null; then
        echo "$(yellow "warning: other ship/shipd processes still running — they may respawn:")"
        pgrep -af 'ship daemon|shipd ' | grep -v "$$" | grep -v "$(basename "$0")" || true
    fi

    # 3. Build (unless skipped).
    if [ "$no_build" = "1" ]; then
        echo "$(yellow "--no-build") — skipping cargo build"
    else
        echo "building shipd from $ROOT ..."
        ( cd "$ROOT" && cargo build -p shipd )
    fi

    if [ ! -x "$BIN" ]; then
        echo "$(red "binary not found"): $BIN"; exit 1
    fi

    # 4. Start in background. nohup so an accidental shell exit doesn't
    # SIGHUP the daemon mid-test. setsid detaches from controlling tty.
    : > "$LOG"
    # Three signals point project resolution at this worktree:
    #   - SHIP_DIR env: honored by runtime::project::get_project_dir()
    #   - --dev-worktree flag: bypasses CWD-walk discovery in main.rs
    #     and routes the daemon into a per-worktree IPC dir + port so
    #     parallel Ship-dev daemons don't collide. Dev-only by name —
    #     users never set this.
    #   - SHIP_RESPECT_WORKTREE=1: opts the runtime out of the
    #     worktree-redirect-to-main behavior in discovery.rs, so this
    #     worktree's local .ship/views/, .ship/agents/, etc. are served
    #     instead of falling back to the main checkout's.
    SHIP_DIR="$SHIP_DIR" \
        SHIP_RESPECT_WORKTREE=1 \
        nohup setsid "$BIN" --port "$PORT" --dev-worktree "$ROOT" >>"$LOG" 2>&1 &
    local launched=$!
    echo "launched pid=$launched · binary=$BIN · SHIP_DIR=$SHIP_DIR"
    echo "log: $LOG"

    # 5. Wait up to ~4s for /api/health.
    local ok=0
    for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do
        sleep 0.25
        if curl -fsS "http://127.0.0.1:$PORT/api/health" >/dev/null 2>&1; then
            ok=1; break
        fi
        # If process died, abort early.
        kill -0 "$launched" 2>/dev/null || break
    done

    if [ "$ok" != "1" ]; then
        echo "$(red "health check failed") — last 40 log lines:"
        tail -n 40 "$LOG" || true
        exit 1
    fi
    echo "$(green "health 200")"
    cmd_status
}

main() {
    local action=start no_build=0
    while [ $# -gt 0 ]; do
        case "$1" in
            --status) action=status ;;
            --stop)   action=stop ;;
            --no-build) no_build=1 ;;
            --port)   shift; PORT="$1" ;;
            --port=*) PORT="${1#--port=}" ;;
            --keep-installed) KEEP_INSTALLED=1 ;;
            -h|--help)
                sed -n '1,45p' "$0"; exit 0 ;;
            *) echo "unknown flag: $1" >&2; exit 2 ;;
        esac
        shift
    done
    case "$action" in
        status) cmd_status ;;
        stop)   cmd_stop ;;
        start)  cmd_start "$no_build" ;;
    esac
}

main "$@"
