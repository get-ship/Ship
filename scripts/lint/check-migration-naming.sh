#!/usr/bin/env bash
# check-migration-naming.sh
#
# Enforce the migration-authoring discipline for
# crates/core/runtime/migrations/ (platform.db).
#
# Rule (pre-launch):
#   - The schema is a single 0001_schema.sql. Edit that file when the
#     schema changes; wipe and rebuild platform.db on existing machines.
#   - When pre-launch ends, this check switches to enforcing
#     ^[0-9]{12}_[a-z0-9_-]+\.sql$ (UTC timestamp prefix). For now,
#     anything other than 0001_schema.sql is a regression.
#
# See:
#   crates/core/runtime/migrations/README.md
#   docs/ship-runtime/references/docs/migrations.md

set -u

cd "$(dirname "$0")/../.."

MIG_DIR="crates/core/runtime/migrations"

# Allowed pre-launch: just the canonical single-file schema.
GRANDFATHERED=(
    "0001_schema.sql"
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
