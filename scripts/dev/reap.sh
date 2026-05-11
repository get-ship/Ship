#!/usr/bin/env bash
# Kill orphan daemon processes whose workspace dir is gone.
#
# Delegates to `ship daemon reap`. Best-effort: exits 0 when the ship
# binary is not yet built (first `just dev` run before cargo build).
set -euo pipefail

cd "$(dirname "$0")/../.."
. scripts/dev/_lib.sh

bin=$(ship_bin 2>/dev/null || true)
if [ -z "$bin" ]; then
    exit 0
fi

"$bin" daemon reap
