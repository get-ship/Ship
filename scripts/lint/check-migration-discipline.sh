#!/usr/bin/env bash
# check-migration-discipline.sh
#
# Enforce migration-authoring discipline for the platform.db migration set
# (crates/core/ship-substrate/migrations/). Complements
# check-migration-naming.sh, which only governs file *names*.
#
# Two rules:
#
#   1. No `CREATE TABLE IF NOT EXISTS` in a post-baseline migration.
#      The squashed baseline (0001_schema.sql) may be idempotent — it is
#      the floor everyone starts from. But a *later* migration that does
#      `CREATE TABLE IF NOT EXISTS foo` silently no-ops when an older,
#      differently-shaped `foo` already exists, then the rest of the
#      migration depends on a shape that was never created. A later
#      migration that needs to (re)create a table must `DROP TABLE IF
#      EXISTS` first, then `CREATE TABLE` unconditionally.
#
#   2. Applied migrations are immutable — a change is a NEW migration.
#      Every .sql file is pinned by sha256 in
#      scripts/lint/migration-checksums.lock. Editing a recorded file
#      changes its hash and fails this check; the fix is to add a new
#      migration, not edit an old one. A new .sql file must be added to
#      the lock in the same change (`--update` regenerates it).
#
# Usage:
#   check-migration-discipline.sh              # check (CI / just check-raw)
#   check-migration-discipline.sh --update     # regenerate the checksum lock
#   check-migration-discipline.sh --self-test  # prove the rules catch bad input
#
# See: crates/core/ship-substrate/migrations/README.md

set -u

cd "$(dirname "$0")/../.."

MIG_DIR="crates/core/ship-substrate/migrations"
LOCK="scripts/lint/migration-checksums.lock"

# Baseline / grandfathered files exempt from rule 1 (idempotency allowed).
#
# `0001_schema.sql` + `0002_drop_kv_state.sql` are the squashed baseline.
# The four `202605*` files below predate this rule's enforcement; they
# already shipped using `CREATE TABLE IF NOT EXISTS`. Editing them now
# would violate rule 2 (immutability), so they stay grandfathered until
# the next baseline squash. New migrations from this date forward must
# use `DROP TABLE IF EXISTS; CREATE TABLE` per the rule.
is_baseline() {
    case "$1" in
        0001_schema.sql | 0002_drop_kv_state.sql) return 0 ;;
        202605181930_analytics.sql) return 0 ;;
        202605190100_analytics_turns.sql) return 0 ;;
        202605191822_agent_sessions.sql) return 0 ;;
        202605250300_provisioned_grants.sql) return 0 ;;
        *) return 1 ;;
    esac
}

# Strip `-- ...` line comments so a rule-1 grep doesn't trip over prose
# that merely *mentions* the forbidden pattern.
strip_sql_comments() {
    sed 's/--.*$//' "$1"
}

# Run both rules against a migration dir + lock file. Echoes findings,
# returns the violation count via the global `violations`.
run_checks() {
    local mig_dir="$1" lock="$2"
    violations=0

    # ---- Rule 1: no late `CREATE TABLE IF NOT EXISTS` --------------------
    for path in "$mig_dir"/*.sql; do
        [ -e "$path" ] || continue
        local name
        name=$(basename "$path")
        is_baseline "$name" && continue
        if strip_sql_comments "$path" \
            | grep -iqE 'create[[:space:]]+table[[:space:]]+if[[:space:]]+not[[:space:]]+exists'; then
            if [ "$violations" -eq 0 ]; then
                echo "check-migration-discipline: rule 1 — late CREATE TABLE IF NOT EXISTS"
                echo "  A post-baseline migration must DROP TABLE IF EXISTS then"
                echo "  CREATE TABLE unconditionally, never CREATE TABLE IF NOT EXISTS."
            fi
            echo "  offender: $path"
            violations=$((violations + 1))
        fi
    done

    # ---- Rule 2: applied migrations are immutable -----------------------
    # Every locked file must still hash to its recorded value.
    while read -r sum file; do
        case "$sum" in '' | '#'*) continue ;; esac
        local target="$mig_dir/$file"
        if [ ! -f "$target" ]; then
            if [ "$violations" -eq 0 ]; then
                echo "check-migration-discipline: rule 2 — locked migration changed"
            fi
            echo "  missing: $target (recorded in $lock)"
            violations=$((violations + 1))
            continue
        fi
        local actual
        actual=$(sha256sum "$target" | cut -d' ' -f1)
        if [ "$actual" != "$sum" ]; then
            if [ "$violations" -eq 0 ]; then
                echo "check-migration-discipline: rule 2 — locked migration changed"
            fi
            echo "  modified: $target"
            echo "    applied migrations are immutable — add a NEW migration"
            echo "    instead of editing this one (or run --update if this is"
            echo "    a deliberate pre-launch baseline change)."
            violations=$((violations + 1))
        fi
    done < "$lock"

    # Every .sql on disk must be pinned in the lock.
    for path in "$mig_dir"/*.sql; do
        [ -e "$path" ] || continue
        local name
        name=$(basename "$path")
        if ! grep -qE "[[:space:]]$name\$" "$lock"; then
            if [ "$violations" -eq 0 ]; then
                echo "check-migration-discipline: rule 2 — unpinned migration"
            fi
            echo "  unpinned: $path"
            echo "    add it to $lock (run --update) so the set stays accountable."
            violations=$((violations + 1))
        fi
    done

    return 0
}

regenerate_lock() {
    {
        echo "# platform.db migration checksum lock — see check-migration-discipline.sh"
        echo "#"
        echo "# One \`sha256  filename\` row per migration in"
        echo "# crates/core/ship-substrate/migrations/. Applied migrations are immutable:"
        echo "# a schema change is a NEW migration file, never an edit to a recorded one."
        echo "# Editing a listed file changes its hash and the discipline lint fails."
        echo "#"
        echo "# Adding a migration: append it here in the same pass that adds the .sql"
        echo "# (regenerate with \`scripts/lint/check-migration-discipline.sh --update\`)."
        for path in "$MIG_DIR"/*.sql; do
            [ -e "$path" ] || continue
            echo "$(sha256sum "$path" | cut -d' ' -f1)  $(basename "$path")"
        done
    } > "$LOCK"
    echo "check-migration-discipline: regenerated $LOCK"
}

self_test() {
    # Prove each rule catches a deliberately bad fixture.
    local tmp
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' RETURN
    mkdir -p "$tmp/migrations"

    # A clean baseline + a clean post-baseline migration.
    echo "CREATE TABLE IF NOT EXISTS ok (x INTEGER);" > "$tmp/migrations/0001_schema.sql"
    printf 'DROP TABLE IF EXISTS later;\nCREATE TABLE later (x INTEGER);\n' \
        > "$tmp/migrations/202601010000_later.sql"
    {
        echo "$(sha256sum "$tmp/migrations/0001_schema.sql" | cut -d' ' -f1)  0001_schema.sql"
        echo "$(sha256sum "$tmp/migrations/202601010000_later.sql" | cut -d' ' -f1)  202601010000_later.sql"
    } > "$tmp/lock"

    run_checks "$tmp/migrations" "$tmp/lock" >/dev/null
    if [ "$violations" -ne 0 ]; then
        echo "self-test FAIL: clean fixtures flagged ($violations)"
        return 1
    fi

    # Rule 1 fixture: a post-baseline migration with CREATE TABLE IF NOT EXISTS.
    echo "CREATE TABLE IF NOT EXISTS bad (x INTEGER);" \
        > "$tmp/migrations/202602020000_bad.sql"
    echo "$(sha256sum "$tmp/migrations/202602020000_bad.sql" | cut -d' ' -f1)  202602020000_bad.sql" \
        >> "$tmp/lock"
    run_checks "$tmp/migrations" "$tmp/lock" >/dev/null
    if [ "$violations" -eq 0 ]; then
        echo "self-test FAIL: rule 1 did not catch late CREATE TABLE IF NOT EXISTS"
        return 1
    fi
    rm "$tmp/migrations/202602020000_bad.sql"
    head -n 2 "$tmp/lock" > "$tmp/lock.trimmed" && mv "$tmp/lock.trimmed" "$tmp/lock"

    # Rule 2 fixture: edit a locked file so its hash drifts.
    echo "-- sneaky edit" >> "$tmp/migrations/202601010000_later.sql"
    run_checks "$tmp/migrations" "$tmp/lock" >/dev/null
    if [ "$violations" -eq 0 ]; then
        echo "self-test FAIL: rule 2 did not catch an edited locked migration"
        return 1
    fi

    echo "check-migration-discipline: self-test ok (both rules catch bad fixtures)"
    return 0
}

case "${1:-}" in
    --update)
        regenerate_lock
        exit 0
        ;;
    --self-test)
        self_test
        exit $?
        ;;
    "")
        if [ ! -d "$MIG_DIR" ]; then
            echo "check-migration-discipline: $MIG_DIR does not exist" >&2
            exit 1
        fi
        if [ ! -f "$LOCK" ]; then
            echo "check-migration-discipline: $LOCK missing — run --update" >&2
            exit 1
        fi
        run_checks "$MIG_DIR" "$LOCK"
        if [ "$violations" -gt 0 ]; then
            echo "check-migration-discipline: $violations violation(s) found"
            exit 1
        fi
        echo "check-migration-discipline: ok"
        exit 0
        ;;
    *)
        echo "usage: check-migration-discipline.sh [--update | --self-test]" >&2
        exit 2
        ;;
esac
