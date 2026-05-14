#!/usr/bin/env bash
# Stop the dev daemon serving this worktree's cwd.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

ship=$(ship_bin)
"$ship" daemon stop

# Reap the per-worktree vite tracked by shell2-start.sh. Its call site here
# was dropped in the same merge that lost the _lib.sh vite definitions.
kill_vite
