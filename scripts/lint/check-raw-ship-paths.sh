#!/usr/bin/env bash
# check-raw-ship-paths.sh
#
# Flag raw `~/.ship/` path construction outside the canonical
# `crates/core/ship-substrate/src/home/` module. Per ADR
# `home-layout-and-scope-convention`, `ShipHome` is the only permitted
# origin of paths under `~/.ship/`.

set -u

cd "$(dirname "$0")/../.."

SEARCH_PATHS=(crates apps)
ALLOW_PREFIX="crates/core/ship-substrate/src/home/"

# Patterns we treat as raw-`~/.ship/` path construction.
PATTERNS=(
    'home_dir\(\)\.join\("\.ship'
    '\.join\("\.ship[/"]'
    '"~/\.ship'
)

violations=0
for pat in "${PATTERNS[@]}"; do
    matches=$(rg -n --no-heading \
        --glob '!apps/docs/**' \
        --glob '!docs/**' \
        --glob '!**/tests.rs' \
        --glob '!**/tests/**' \
        --glob '!**/*_test.rs' \
        --glob '!**/tests_*.rs' \
        --glob '!crates/sui/ship-sui/**' \
        --glob '!crates/sui/ship-sui-host/**' \
        --glob '!crates/sui/ship-sui-server/**' \
        --glob '!crates/sui/compiler/**' \
        --glob '!crates/sui/signals/**' \
        --glob '!crates/sui/lint/**' \
        "$pat" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
    if [ -z "$matches" ]; then
        continue
    fi
    while IFS= read -r line; do
        file=${line%%:*}
        case "$file" in
            "$ALLOW_PREFIX"*) ;;
            *)
                if [ $violations -eq 0 ]; then
                    echo "check-raw-ship-paths: raw ~/.ship/ path construction outside $ALLOW_PREFIX"
                fi
                echo "  $line"
                violations=$((violations + 1))
                ;;
        esac
    done <<< "$matches"
done

if [ $violations -gt 0 ]; then
    echo "check-raw-ship-paths: $violations violation(s) found"
    exit 1
fi
exit 0
