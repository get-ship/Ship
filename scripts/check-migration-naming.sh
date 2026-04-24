#!/usr/bin/env bash
# check-migration-naming.sh
#
# Enforce the migration-authoring discipline for
# crates/core/runtime/migrations/ (platform.db).
#
# Rule (effective 2026-04-24):
#   - New files MUST match: ^[0-9]{12}_[a-z0-9_-]+\.sql$
#     (UTC timestamp prefix YYYYMMDDHHMM, kebab/underscore description)
#   - The six legacy numeric-prefixed files below are GRANDFATHERED and frozen.
#     They are history. Do not edit them; do not add more like them.
#   - Any other file matching ^[0-9]{4}_ is a regression and fails this check.
#
# See:
#   crates/core/runtime/migrations/README.md
#   docs/ship-runtime/references/docs/migrations.md

set -u

cd "$(dirname "$0")/.."

MIG_DIR="crates/core/runtime/migrations"

# Grandfathered — frozen as of 2026-04-24. The platform-table-drop job
# (0006) was the final allowed edit to this numeric series. All further
# migrations use the timestamp convention.
GRANDFATHERED=(
    "0001_schema.sql"
    "0002_specs.sql"
    "0003_drop_event_log.sql"
    "0004_compiled_asset_cache.sql"
    "0005_git_commit_drafts.sql"
    "0006_drop_platform_user_tables.sql"
)

is_grandfathered() {
    local name="$1"
    for g in "${GRANDFATHERED[@]}"; do
        if [ "$name" = "$g" ]; then
            return 0
        fi
    done
    return 1
}

if [ ! -d "$MIG_DIR" ]; then
    echo "check-migration-naming: $MIG_DIR does not exist" >&2
    exit 1
fi

violations=0

for path in "$MIG_DIR"/*.sql; do
    [ -e "$path" ] || continue
    name=$(basename "$path")

    # New-style timestamp prefix: 12 digits + _kebab + .sql
    if [[ "$name" =~ ^[0-9]{12}_[a-z0-9_-]+\.sql$ ]]; then
        continue
    fi

    # Legacy 4-digit numeric prefix
    if [[ "$name" =~ ^[0-9]{4}_ ]]; then
        if is_grandfathered "$name"; then
            continue
        fi
        if [ $violations -eq 0 ]; then
            echo "check-migration-naming: illegal legacy-numbered migration(s) found"
            echo "  The 0001–0006 series is frozen. Use YYYYMMDDHHMM_kebab.sql."
            echo "  See crates/core/runtime/migrations/README.md"
        fi
        echo "  offender: $path"
        violations=$((violations + 1))
        continue
    fi

    # Anything else that ended up here
    if [ $violations -eq 0 ]; then
        echo "check-migration-naming: non-conforming migration file(s) found"
        echo "  Expected ^[0-9]{12}_[a-z0-9_-]+\\.sql\$"
    fi
    echo "  offender: $path"
    violations=$((violations + 1))
done

if [ $violations -gt 0 ]; then
    exit 1
fi

echo "check-migration-naming: ok"
exit 0
