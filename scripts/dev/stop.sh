#!/usr/bin/env bash
# Stop the dev daemon serving this worktree's cwd.
set -euo pipefail
cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

ship=$(ship_bin)
"$ship" daemon stop
