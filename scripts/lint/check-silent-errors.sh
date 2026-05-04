#!/usr/bin/env bash
# Fail CI if silent-error patterns reappear in persistence/dispatch paths.
#
# Background: real-world pipeline failures kept hiding behind `let _ = ...`
# and `if let Ok(x) = ...` in the runtime. Phase 0 of the reliability work
# removed them from the hot paths. This check prevents regression.
#
# Banned patterns (non-exhaustive, see rules below):
#   - `let _ = <anything>.route(...)`       — kernel routing dropped silently
#   - `let _ = <anything>.emit(...)`        — kernel emit dropped silently
#   - `let _ = <anything>.append(...)`      — event persistence dropped silently
#   - `let _ = persist_and_route(...)`      — helper result dropped
#   - `if let Ok(store) = SqliteEventStore::new()` on its own line — the
#     "defensive" pattern that swallows store-open failures
#
# Allowed (grandfathered, legitimate, or unrelated):
#   - `let _ = tx.try_send(...)`            — mailbox overflow, architectural
#   - `let _ = sqlx::query("ROLLBACK")`     — cleanup on already-erroring path
#   - `let _ = kernel.stop_actor(...)`      — fire-and-forget cleanup
#   - `let _ = OnceCell::set(...)`          — initialization race is fine
#   - Anything in a test module (`#[cfg(test)]` context)
#
# Scope: runtime + shipd. The guardrail doesn't police higher-level apps
# (ship-cli, mcp) because they're not the event-log write path.

set -eo pipefail

cd "$(git rev-parse --show-toplevel)"

ROOTS=(crates/core/runtime/src apps/shipd/src)
# Exclude test files — they may legitimately ignore results in fixture setup.
EXCLUDES=(--exclude='*_test*.rs' --exclude='tests_*.rs' --exclude='tests.rs')

violations=0
fail() {
  echo "FAIL: $*" >&2
  violations=$((violations + 1))
}

# ── let _ = *.route(...) ──────────────────────────────────────────────────────
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'let _ = .*\.route\(' "${ROOTS[@]}" 2>/dev/null); then
  fail "silent route drop"
  echo "$matches" >&2
fi

# ── let _ = *.emit(...) ──────────────────────────────────────────────────────
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'let _ = .*\.emit\(' "${ROOTS[@]}" 2>/dev/null); then
  fail "silent emit drop"
  echo "$matches" >&2
fi

# ── let _ = *.append(...) (event store), excluding legitimate non-event appends ─
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'let _ = .*\.append\(' "${ROOTS[@]}" 2>/dev/null | grep -v 'strings' | grep -v 'json'); then
  fail "silent append drop"
  echo "$matches" >&2
fi

# ── let _ = persist_and_route(...) ───────────────────────────────────────────
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'let _ = .*persist_and_route\(' "${ROOTS[@]}" 2>/dev/null); then
  fail "silent persist_and_route drop"
  echo "$matches" >&2
fi

# ── if let Ok(store) = SqliteEventStore::new() { ... } shortcuts ─────────────
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'if let Ok\(store\) = .*SqliteEventStore::new\(\)' "${ROOTS[@]}" 2>/dev/null); then
  fail "if-let-Ok store swallow: use ? with a Context instead"
  echo "$matches" >&2
fi

# ── DROP TABLE _sqlx_migrations in non-test production code ─────────────────
if matches=$(grep -rn "${EXCLUDES[@]}" -E 'DROP TABLE .*_sqlx_migrations' "${ROOTS[@]}" 2>/dev/null); then
  fail "DROP TABLE _sqlx_migrations — silent schema reset is data loss"
  echo "$matches" >&2
fi

if [ "$violations" -gt 0 ]; then
  echo "" >&2
  echo "$violations silent-error pattern violation(s) found." >&2
  echo "If a match is legitimate, document why and either:" >&2
  echo "  - wrap in tracing::warn!/error! with the error, or" >&2
  echo "  - add a specific allowlist comment explaining the intent." >&2
  exit 1
fi

echo "silent-errors check: clean"
