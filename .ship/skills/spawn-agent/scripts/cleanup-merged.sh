#!/usr/bin/env bash
# Post-merge cleanup for a dispatched agent's slug.
#
# Run AFTER `git merge --no-ff job/<slug>` lands cleanly on the integration
# branch. Tears down everything dispatch.sh and the agent created:
#   1. The worktree at ~/dev/Ship/worktrees/<slug> (force, in case the
#      agent left untracked files like package-lock.json from an
#      accidental npm invocation)
#   2. The local branch job/<slug>
#   3. The tmux session named <slug> (where dispatch.sh launched the agent)
#   4. Any vite dev server whose cwd is the worktree path
#   5. Any shipd daemon whose ~/.ship/machine/daemons/<id>/workspace points
#      at the worktree, plus the daemon dir itself
#   6. Any orphaned `claude --model …` process whose cwd is the (now
#      deleted) worktree
#
# Usage:
#   bash .ship/skills/spawn-agent/scripts/cleanup-merged.sh <slug> [<slug>...]
#
# Idempotent: missing tmux session / daemon / vite is not an error. Safe
# to run before merge for diagnostic ("are there any resources?") and
# again after to actually tear them down.
set -u

# Resolve worktree base via the shared helper sourced from this script's dir.
# Helpers: scripts/_worktree-base.sh — defines resolve_worktree_base()
_SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_worktree-base.sh
source "${_SELF_DIR}/_worktree-base.sh"
WORKTREE_BASE="$(resolve_worktree_base)"
WORKTREE_BASE="${WORKTREE_BASE/#\~/$HOME}"
DAEMONS_DIR="$HOME/.ship/machine/daemons"

if [[ $# -lt 1 ]]; then
  echo "usage: cleanup-merged.sh <slug> [<slug>...]" >&2
  exit 2
fi

cleanup_one() {
  local slug="$1"
  local wt="${WORKTREE_BASE}/${slug}"
  echo "── cleanup: ${slug} ──"

  # 1. Worktree (--force tolerates untracked files left by the agent).
  # `git worktree list` shows registered worktrees; presence of the path
  # in that list is the authoritative signal — a bare directory at the
  # path may exist as detritus after a previous incomplete remove.
  local repo_root
  repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
  if git -C "$repo_root" worktree list --porcelain 2>/dev/null | grep -q "^worktree ${wt}$"; then
    git -C "$repo_root" worktree remove --force "$wt" 2>&1 \
      && echo "  worktree removed"
  elif [[ -d "$wt" ]]; then
    rm -rf "$wt" && echo "  worktree dir (orphan) rm -rf'd"
  else
    echo "  worktree absent"
  fi

  # 2. Branch (only if it exists; skip silently otherwise).
  if git rev-parse --verify --quiet "job/${slug}" >/dev/null; then
    git branch -D "job/${slug}" 2>&1 | sed 's/^/  branch: /'
  fi

  # 3. Tmux session.
  if tmux has-session -t "$slug" 2>/dev/null; then
    tmux kill-session -t "$slug" 2>&1 && echo "  tmux killed"
  fi

  # 4. Vite dev servers anchored to this worktree's path.
  for pid in $(pgrep -f "${wt}/.*vite/bin/vite" 2>/dev/null); do
    kill "$pid" 2>/dev/null && echo "  vite ${pid} killed"
  done

  # 5. Daemon dir + process for this workspace.
  if [[ -d "$DAEMONS_DIR" ]]; then
    for d in "$DAEMONS_DIR"/*/; do
      [[ -e "$d/workspace" ]] || continue
      local ws
      ws="$(cat "$d/workspace" 2>/dev/null)"
      if [[ "$ws" == "${WORKTREE_BASE}/${slug}" ]]; then
        local pid
        pid="$(cat "$d/pid" 2>/dev/null)"
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
          kill "$pid" && echo "  daemon ${pid} killed"
        fi
        rm -rf "$d" && echo "  daemon dir $(basename "$d") removed"
      fi
    done
  fi

  # 6. Orphan claude/agent processes whose cwd was the worktree (now
  # deleted — /proc/<pid>/cwd shows "(deleted)" suffix or the bare path).
  for pid in $(pgrep -f 'claude .*--model'); do
    local cwd
    cwd="$(readlink "/proc/$pid/cwd" 2>/dev/null)"
    case "$cwd" in
      "${wt}"|"${wt} (deleted)")
        kill "$pid" 2>/dev/null && echo "  agent ${pid} killed"
        ;;
    esac
  done

  echo
}

for slug in "$@"; do
  cleanup_one "$slug"
done
