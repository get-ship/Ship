#!/usr/bin/env bash
# check-raw-ship-paths.sh
#
# Flag raw XDG/SHIP path construction outside the canonical accessors
# in `crates/core/ship-substrate/src/home/` and
# `crates/core/ship-substrate/crates/ship-paths/src/`. Per ADR
# `home-layout-and-scope-convention`, `ShipHome` is the only permitted
# origin of paths under any of the four XDG roots or the legacy
# `~/.ship/` fallback.
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
    "crates/core/ship-substrate/src/home/"
    "crates/core/ship-substrate/crates/ship-paths/src/"
)

# Patterns we treat as raw-XDG / raw-~/.ship/ path construction.
# Covers the legacy ~/.ship/ root (no migration — pre-launch) plus the
# four XDG dirs Ship's layout now uses.
PATTERNS=(
    'home_dir\(\)\.join\("\.ship'
    '\.join\("\.ship[/"]'
    '"~/\.ship'
    '"\$XDG_CONFIG_HOME/ship'
    '"\$XDG_DATA_HOME/ship'
    '"\$XDG_STATE_HOME/ship'
    '"\$XDG_CACHE_HOME/ship'
    '"\$XDG_RUNTIME_DIR/ship'
    'join\("\.config/ship'
    'join\("\.local/share/ship'
    'join\("\.local/state/ship'
    'join\("\.cache/ship'
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

# Line number of the last `#[cfg(test)]`-gated `mod tests { ... }` in a file
# (attributes/blank lines between the two are tolerated), or empty if none.
# Repo convention is one trailing test module per file running to EOF, so
# everything from that line on is test code — "tests don't count" as a live
# consumer (see CLAUDE.md). This is a line-based heuristic, not a parser: a
# `#[cfg(test)] mod tests {` that is NOT the trailing module (production code
# follows it) would wrongly suppress violations after it — not the case
# anywhere in this repo today.
trailing_test_mod_boundary() {
    awk '
        /^[[:space:]]*#\[cfg\(test\)\]/ { pending=1; next }
        /^[[:space:]]*#\[/ { next }
        /^[[:space:]]*$/ { next }
        /^[[:space:]]*mod[[:space:]]+tests?[[:space:]]*\{/ {
            if (pending == 1) { boundary = NR }
            pending = 0
            next
        }
        { pending = 0 }
        END { if (boundary) print boundary }
    ' "$1" 2>/dev/null
}

violations=0
for pat in "${PATTERNS[@]}"; do
    matches=$(rg -n --no-heading "${EXCLUDE_GLOBS[@]}" "$pat" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
    if [ -z "$matches" ]; then
        continue
    fi
    while IFS= read -r line; do
        file=${line%%:*}
        rest=${line#*:}
        lineno=${rest%%:*}
        content=${rest#*:}

        allowed=0
        for prefix in "${ALLOW_PREFIXES[@]}"; do
            case "$file" in
                "$prefix"*) allowed=1; break ;;
            esac
        done
        if [ "$allowed" -eq 1 ]; then
            continue
        fi

        # Doc/line comments talking about the pattern (e.g. this script's own
        # name in a module comment) aren't code constructing a path.
        if [[ "$content" =~ ^[[:space:]]*// ]]; then
            continue
        fi

        boundary=$(trailing_test_mod_boundary "$file")
        if [[ -n "$boundary" && "$lineno" -ge "$boundary" ]]; then
            continue
        fi

        if [ "$violations" -eq 0 ]; then
            echo "check-raw-ship-paths: raw XDG/ship path construction outside the sanctioned accessor:" >&2
        fi
        echo "  $line" >&2
        violations=$((violations + 1))
    done <<< "$matches"
done

if [ "$violations" -gt 0 ]; then
    echo "check-raw-ship-paths: $violations violation(s) found" >&2
    exit 1
fi
exit 0
