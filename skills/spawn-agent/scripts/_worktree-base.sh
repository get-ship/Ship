#!/usr/bin/env bash
# Shared resolver for the worktree base directory.
#
# Precedence: SHIP_WORKTREE_DIR env > `ship config get worktrees.dir`.
# The `ship` lookup trims trailing whitespace, takes the LAST line, and ignores
# the literal `(unset)` so an unset config falls through to a loud error rather
# than a guessed HOME layout.
#
# Callers should still apply tilde expansion (`${VAR/#\~/$HOME}`) to the
# returned value, since `ship config` returns the path verbatim.
#
# Sourced by dispatch-portable.sh.
# Defines resolve_worktree_base() and leaves WORKTREE_BASE unset —
# callers bind the result themselves so they control the tilde step
# and the `set -u` interaction.
resolve_worktree_base() {
  if [[ -n "${SHIP_WORKTREE_DIR:-}" ]]; then echo "$SHIP_WORKTREE_DIR"; return; fi
  local cfg=""
  if command -v ship &>/dev/null; then
    cfg="$(ship config get worktrees.dir 2>/dev/null | tail -n1 | tr -d '[:space:]')"
  fi
  if [[ -n "$cfg" && "$cfg" != "(unset)" ]]; then echo "$cfg"; return; fi
  echo "SHIP_WORKTREE_DIR/worktrees.dir is required; no default worktree base is guessed" >&2
  return 1
}
