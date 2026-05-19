#!/usr/bin/env bash
# check-raw-env-reads.sh
#
# Flag raw `env::var("SHIP_*")` reads outside the canonical
# `crates/core/ship-substrate/src/env.rs` module. Per ADR
# `home-layout-and-scope-convention`, `env::read(Var::…)` is the single
# accessor.

set -u

cd "$(dirname "$0")/../.."

SEARCH_PATHS=(crates apps)
ALLOW_FILE="crates/core/ship-substrate/src/env.rs"

# Matches `env::var("SHIP_…")` and `env::var("SHIPD_…")`.
PATTERN='env::var\("SHIPD?_'

violations=0
matches=$(rg -n --no-heading \
    --glob '!apps/docs/**' \
    --glob '!docs/**' \
    --glob '!crates/ship/ship-sui/**' \
    --glob '!crates/ship/ship-sui-host/**' \
    --glob '!crates/ship/ship-sui-server/**' \
    "$PATTERN" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
if [ -n "$matches" ]; then
    while IFS= read -r line; do
        file=${line%%:*}
        if [ "$file" = "$ALLOW_FILE" ]; then
            continue
        fi
        if [ $violations -eq 0 ]; then
            echo "check-raw-env-reads: raw env::var(\"SHIP*_…\") outside $ALLOW_FILE"
        fi
        echo "  $line"
        violations=$((violations + 1))
    done <<< "$matches"
fi

if [ $violations -gt 0 ]; then
    echo "check-raw-env-reads: $violations violation(s) found"
    exit 1
fi
exit 0
