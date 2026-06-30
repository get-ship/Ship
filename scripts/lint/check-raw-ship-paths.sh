#!/usr/bin/env bash
# check-raw-ship-paths.sh
#
# Flag raw `~/.ship/` path construction outside the canonical accessors
# in `crates/core/ship-substrate-core/src/home/` (L0 surface, re-exported
# from `crates/core/ship-substrate/crates/ship-paths/src/`). Per ADR
# `home-layout-and-scope-convention`, `ShipHome` is the only permitted
# origin of paths under `~/.ship/`.
#
# Enforcement: exits non-zero on ANY matching file/line outside the
# allow-list. The script is intentionally NOT warn-only — `data-safety`
# depends on it (see job datasafety-shiphome).
#
# Note: inline `#` comments inside `$(...)` break bash's command-substitution
# parsing (the comment terminates the substitution), so all comment
# documentation lives OUTSIDE the substitution, not in the search call.

set -u

cd "$(dirname "$0")/../.."

SEARCH_PATHS=(crates apps)

# Sanctioned accessor locations. Both must be reachable for the wildcard
# comparison to work — basename matching against `<file>:` lines from rg.
ALLOW_PREFIXES=(
    "crates/core/ship-substrate-core/src/home/"
    "crates/core/ship-substrate/crates/ship-paths/src/"
)

# Patterns we treat as raw-`~/.ship/` path construction.
PATTERNS=(
    'home_dir\(\)\.join\("\.ship'
    '\.join\("\.ship[/"]'
    '"~/\.ship'
)

# crates/deck/* + crates/sui/* moved to the external sui repo post-carve —
# their glob exclusions no longer apply here (they aren't part of the ship
# repo). If a future crate needs an exception, add its repo-relative path.
EXCLUDE_GLOBS=(
    --glob '!apps/docs/**'
    --glob '!docs/**'
    --glob '!**/tests.rs'
    --glob '!**/*_tests.rs'
    --glob '!**/tests/**'
    --glob '!**/test/**/*.rs'
    --glob '!**/*_test.rs'
    --glob '!**/tests_*.rs'
    --glob '!**/*_fixtures.rs'
    --glob '!**/fixtures/**'
)

violations=0
for pat in "${PATTERNS[@]}"; do
    matches=$(rg -n --no-heading "${EXCLUDE_GLOBS[@]}" "$pat" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
    if [ -z "$matches" ]; then
        continue
    fi
    while IFS= read -r line; do
        file=${line%%:*}
        allowed=0
        for prefix in "${ALLOW_PREFIXES[@]}"; do
            case "$file" in
                "$prefix"*) allowed=1; break ;;
            esac
        done
        if [ "$allowed" -eq 0 ]; then
            if [ "$violations" -eq 0 ]; then
                echo "check-raw-ship-paths: raw ~/.ship/ path construction outside the sanctioned accessor:" >&2
            fi
            echo "  $line" >&2
            violations=$((violations + 1))
        fi
    done <<< "$matches"
done

if [ "$violations" -gt 0 ]; then
    echo "check-raw-ship-paths: $violations violation(s) found" >&2
    exit 1
fi
exit 0
