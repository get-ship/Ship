#!/usr/bin/env bash
# check-migration-naming.sh
#
# Enforce the migration-authoring discipline across all
# sqlx-managed migration directories (platform.db, kv.db, ...).
#
# Rule:
#   - Baseline schema lives in 0001_schema.sql (squashed once, kept
#     editable while pre-launch — wipes are cheaper than chains).
#   - All NEW schema changes after that land as
#     ^[0-9]{12}_[a-z0-9_-]+\.sql$ (UTC timestamp prefix YYYYMMDDHHMM).
#   - sqlx applies them incrementally and tracks them in _sqlx_migrations.
#     We don't write Rust "migrator" helpers that live on forever —
#     SQL DDL only.
#
# See:
#   crates/core/ship-substrate/migrations/README.md

set -u

cd "$(dirname "$0")/../.."

# ship-application was folded into ship-platform; its migrations/ dir is
# gone. platform.db migrations live under ship-substrate.
MIG_DIRS=(
    "crates/core/ship-substrate/migrations"
)

# Allowed pre-launch: the canonical baseline + any legacy numbered
# migrations that already shipped. Renaming a shipped migration would
# break sqlx's `_sqlx_migrations` tracking on installed databases.
# Going forward, use the YYYYMMDDHHMM_kebab.sql pattern.
GRANDFATHERED=(
    "0001_schema.sql"
    "0002_drop_kv_state.sql"
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

violations=0
for MIG_DIR in "${MIG_DIRS[@]}"; do
    if [ ! -d "$MIG_DIR" ]; then
        echo "check-migration-naming: $MIG_DIR does not exist" >&2
        exit 1
    fi

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
done

if [ $violations -gt 0 ]; then
    exit 1
fi

echo "check-migration-naming: ok"
exit 0
