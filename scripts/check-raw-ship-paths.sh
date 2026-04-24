#!/usr/bin/env bash
# check-raw-ship-paths.sh
#
# Flag raw `~/.ship/` path construction outside the canonical
# `crates/core/runtime/src/home/` module. Per ADR
# `home-layout-and-scope-convention`, `ShipHome` is the only permitted
# origin of paths under `~/.ship/`.
#
# TODO(phase-2): flip `exit 0` at the bottom to `exit $violations` once
# every existing call site has been migrated. Until then this is warn-only.

set -u

cd "$(dirname "$0")/.."

SEARCH_PATHS=(crates apps)
ALLOW_PREFIX="crates/core/runtime/src/home/"

# Patterns we treat as raw-`~/.ship/` path construction.
PATTERNS=(
    'home_dir\(\)\.join\("\.ship'
    '\.join\("\.ship"\)'
    '"~/\.ship'
)

violations=0
for pat in "${PATTERNS[@]}"; do
    # rg returns 1 when there are no matches; we don't want that to kill us
    # under `set -e` (which we didn't set, but be explicit anyway).
    matches=$(rg -n --no-heading "$pat" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
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
    echo "check-raw-ship-paths: $violations violation(s) found (warn-only; phase-2 will fail the build)"
fi

# TODO(phase-2): change to `exit $violations` (> 0 means fail).
exit 0
