#!/bin/sh
# scripts/_sui-root.sh — portable resolution of the Sail monorepo tip
# (sibling `sui/`, or orch worktree at `worktrees/orch`).
#
# Deck builds (hub/spaces, component packs) need two paths from the monorepo:
#   - <root>/Cargo.toml                          (the deck-cli workspace)
#   - <root>/crates/deck/deck-core/assets        (the framework assets bundle)
#
# Layout: umbrella with sui/ + ship/ as siblings is the common case;
# during cutover we also accept `worktrees/orch`.
#
#   1. If `SUI_ROOT` is set and points at a directory containing `Cargo.toml`,
#      use it as-is. (Also honor `ORCH` / `SAIL_ROOT` as aliases.)
#   2. Otherwise, walk up from the caller's directory looking for the closest
#      ancestor that looks like the ship product root (apps/ + packages/ +
#      package.json named "ship"), then look for:
#        - `worktrees/orch` next to the Ship umbrella parent
#        - `sui/` as a SIBLING of that root
#        - same-dir `sui/`
#   3. Also accept being called from inside sui/ or orch directly.
#   4. Bail with a clear error if none finds the monorepo.
#
# Exports:
#   SUI_ROOT                  — absolute path to the Sail monorepo tip
#   DECK_FRAMEWORK_ASSETS_DIR — absolute path to the framework assets bundle
#
# Usage from a package.json script:
#   . "$PWD/../../scripts/_sui-root.sh"

set -u

_SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

start_dir=${1:-$PWD}

if [ -z "${SUI_ROOT:-}" ]; then
  if [ -n "${ORCH:-}" ]; then
    SUI_ROOT=$ORCH
  elif [ -n "${SAIL_ROOT:-}" ]; then
    SUI_ROOT=$SAIL_ROOT
  fi
fi

if [ -n "${SUI_ROOT:-}" ] && [ -f "$SUI_ROOT/Cargo.toml" ]; then
  : # fall through with the user's value
elif [ -n "${SUI_ROOT:-}" ]; then
  echo "_sui-root.sh: SUI_ROOT is set but $SUI_ROOT/Cargo.toml does not exist" >&2
  echo "  (unset SUI_ROOT to fall back to walk-up discovery)" >&2
  return 1 2>/dev/null || exit 1
else
  SUI_ROOT=
  d=$(CDPATH= cd -- "$start_dir" && pwd)
  while [ -n "$d" ]; do
    base=$(basename -- "$d")
    if { [ "$base" = "sui" ] || [ "$base" = "orch" ]; } && [ -f "$d/Cargo.toml" ]; then
      SUI_ROOT=$d
      break
    fi
    # Product root signature: apps/ + packages/ + package.json name "ship"
    # (also accept legacy "ship-apps" during cutover).
    if [ -d "$d/apps" ] && [ -d "$d/packages" ] && [ -f "$d/package.json" ] \
       && grep -qE '"name"[[:space:]]*:[[:space:]]*"(ship|ship-apps)"' "$d/package.json" 2>/dev/null; then
      if [ -f "$d/../worktrees/orch/Cargo.toml" ]; then
        SUI_ROOT=$(CDPATH= cd -- "$d/../worktrees/orch" && pwd)
        break
      fi
      if [ -f "$d/../sail/Cargo.toml" ]; then
        SUI_ROOT=$(CDPATH= cd -- "$d/../sail" && pwd)
        break
      fi
      if [ -f "$d/sui/Cargo.toml" ]; then
        SUI_ROOT=$d/sui
        break
      fi
      if [ -f "$d/orch/Cargo.toml" ]; then
        SUI_ROOT=$d/orch
        break
      fi
    fi
    parent=$(dirname -- "$d")
    [ "$parent" = "$d" ] && break
    d=$parent
  done
  if [ -z "$SUI_ROOT" ]; then
    echo "_sui-root.sh: could not locate the Sail monorepo tip (sui/ or worktrees/orch)" >&2
    echo "  started at: $start_dir" >&2
    echo "  expected: worktrees/orch or sui/ next to the ship root" >&2
    echo "  or set SUI_ROOT=/abs/path/to/orch (or ORCH=...) explicitly." >&2
    return 1 2>/dev/null || exit 1
  fi
fi

export SUI_ROOT
export DECK_FRAMEWORK_ASSETS_DIR="$SUI_ROOT/crates/deck/deck-core/assets"
export SHIP_APPS_SCRIPTS="$_SCRIPT_DIR"
