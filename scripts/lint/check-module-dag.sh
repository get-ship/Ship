#!/usr/bin/env bash
# check-module-dag.sh
#
# Enforce the modular-runtime-04 module DAG. The hierarchy (top depends
# on bottom):
#
#     main / lib / daemon
#                 │
#                 ▼
#         routes / mcp
#                 │
#                 ▼
#       services / views
#                 │
#                 ▼
#             transport
#                 │
#                 ▼
#               auth
#
# This lint catches the worst violations:
#   - auth/ imports anything from transport / services / views / routes / mcp
#   - transport/ imports services / views / routes / mcp
#   - services/ + views/ imports routes / mcp
#
# Cycles or upward edges in the DAG would re-create the godstate
# entanglement mr-03 + mr-04 were designed to remove.

set -u

cd "$(dirname "$0")/../.."

if ! command -v rg >/dev/null 2>&1; then
    echo "check-module-dag: ripgrep (rg) not installed; skipping"
    exit 0
fi

violations=0

check_no_upward() {
    local lower="$1"
    shift
    local higher_re="$(IFS='|'; echo "$*")"
    local hits
    hits=$(rg --line-number "use crate::($higher_re)\\b|crate::($higher_re)::" \
        "apps/shipd/src/$lower/" 2>/dev/null \
        | grep -Ev '/_?tests?(\.rs|/)' \
        | grep -Ev '_tests?\.rs:' \
        | grep -Ev '^[^:]*://' \
        || true)
    if [ -n "$hits" ]; then
        echo "check-module-dag: $lower/ imports upward into [$higher_re]:"
        echo "$hits"
        violations=$((violations + 1))
    fi
}

# auth is the bottom of the DAG.
check_no_upward auth transport services views routes mcp
# transport sits above auth; cannot reach into the layers above it.
check_no_upward transport services views routes mcp
# services + views sit above transport; cannot reach into routes / mcp.
check_no_upward services routes mcp
check_no_upward views routes mcp

if [ $violations -gt 0 ]; then
    echo
    echo "Module DAG enforced by modular-runtime-04. Lower layers must not"
    echo "depend on higher ones. Move the shared piece down or invert the"
    echo "dependency through a trait."
    exit 1
fi

echo "check-module-dag: ok"
exit 0
