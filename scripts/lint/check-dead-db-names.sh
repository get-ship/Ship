#!/usr/bin/env bash
# check-dead-db-names.sh
#
# Fossil guard (invariant I-6): dead machine stores can never come back.
# `platform.db` (the single global event store), `work.db`, and the
# machine-wide `data.db` are gone from the runtime. The sanctioned set is
# exactly: `runtime.db`, `account.db`, `spaces/<id>/space.db`, and the
# machine `kv.db`. The compile-time twin of this guard is
# `crates/core/ship-substrate-core/src/db/tests_db_paths.rs`.
#
# Enforcement: exits non-zero when any non-test Rust source constructs a
# path with a dead db name — a quoted `"platform.db"` / `"work.db"` /
# `"data.db"` string literal or a `/`-joined path fragment. Prose comments
# that merely mention the dead names (backticked or bare) do not match, and
# prefixed fixture names (`test-data.db`) do not match. Test files and
# fixtures are excluded by the same globs as check-raw-ship-paths.sh.

set -u

cd "$(dirname "$0")/../.."

SEARCH_PATHS=(crates apps)

# A dead db name is a violation only in code-shaped position: immediately
# preceded by a quote (\x22 double, \x27 single) or a path slash.
PATTERN='["\x27/](platform|work|data)\.db'

EXCLUDE_GLOBS=(
    --glob '*.rs'
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
matches=$(rg -n --no-heading "${EXCLUDE_GLOBS[@]}" "$PATTERN" "${SEARCH_PATHS[@]}" 2>/dev/null || true)
if [ -n "$matches" ]; then
    while IFS= read -r line; do
        if [ "$violations" -eq 0 ]; then
            echo "check-dead-db-names: dead db name (platform.db / work.db / data.db) constructed in non-test Rust source:" >&2
        fi
        echo "  $line" >&2
        violations=$((violations + 1))
    done <<< "$matches"
fi

if [ "$violations" -gt 0 ]; then
    echo "check-dead-db-names: $violations violation(s) found — the sanctioned db set is runtime.db / account.db / spaces/<id>/space.db / machine kv.db" >&2
    exit 1
fi
exit 0
