#!/usr/bin/env bash
# Stop the dev daemon and vite server for this worktree.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

ship=$(ship_bin)
"$ship" daemon stop
kill_vite
