#!/usr/bin/env bash
# check-raw-env-reads.sh
#
# Flag raw `env::var("SHIP_…")` reads outside the canonical
# `crates/core/ship-substrate/crates/ship-paths/src/env.rs` module (L0
# surface, re-exported via `ship_substrate_core::env`). Per ADR
# `home-layout-and-scope-convention`, `env::read(Var::…)` is the single
# accessor.
#
# Enforcement: exits non-zero on any matching line outside the allow-list.
# The script is intentionally NOT warn-only.
#
# Note: inline `#` comments inside `$( ... )` break bash's command-
# substitution parsing (the comment terminates the substitution), so all
# documentation lives OUTSIDE the search call.

set -u

cd "$(dirname "$0")/../.."

SEARCH_PATHS=(crates apps)

# Sanctioned accessor locations.
ALLOW_FILES=(
    "crates/core/ship-substrate/crates/ship-paths/src/env.rs"
)

# Matches `env::var("SHIP_…")` and `env::var("SHIPD_…")`.
PATTERN='env::var\("SHIPD?_'

# crates/deck/* moved to the external sui repo post-carve — no glob
# exceptions needed here.
EXCLUDE_GLOBS=(
    --glob '!apps/docs/**'
    --glob '!docs/**'
)

violations=0
matches=$(rg -n --no-heading "${EXCLUDE_GLOBS[@]}" "$PATTERN" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
if [ -n "$matches" ]; then
    while IFS= read -r line; do
        file=${line%%:*}
        allowed=0
        for allow in "${ALLOW_FILES[@]}"; do
            if [ "$file" = "$allow" ]; then
                allowed=1
                break
            fi
        done
        if [ "$allowed" -eq 0 ]; then
            if [ "$violations" -eq 0 ]; then
                echo "check-raw-env-reads: raw env::var(\"SHIP*_…\") outside the sanctioned accessor:" >&2
            fi
            echo "  $line" >&2
            violations=$((violations + 1))
        fi
    done <<< "$matches"
fi

if [ "$violations" -gt 0 ]; then
    echo "check-raw-env-reads: $violations violation(s) found" >&2
    exit 1
fi
exit 0
