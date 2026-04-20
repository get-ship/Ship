#!/usr/bin/env bash
# Tests for scripts/install.sh — shellcheck + offline idempotency smoke.
#
# The smoke tests never hit the network: they stub the `download` step by
# setting SHIP_VERSION to a value matching a fake local archive placed in
# the mock server dir, or use --dry-run to exercise the planning code.
#
# Usage:
#   bash scripts/install.test.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_SH="$SCRIPT_DIR/install.sh"
PASS=0
FAIL=0

pass() { echo "  PASS: $1"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $1"; echo "    $2"; FAIL=$((FAIL + 1)); }

# ── 1. shellcheck clean ─────────────────────────────────────────────────────

test_shellcheck() {
  if ! command -v shellcheck > /dev/null 2>&1; then
    echo "  SKIP: shellcheck not installed"
    return 0
  fi
  if shellcheck "$INSTALL_SH" > /tmp/install.shellcheck.log 2>&1; then
    pass "shellcheck clean"
  else
    fail "shellcheck clean" "$(cat /tmp/install.shellcheck.log)"
  fi
}

# ── 2. dry-run succeeds + prints the expected bundle URL ────────────────────

test_dry_run() {
  local out
  out=$(SHIP_VERSION=0.0.0-test bash "$INSTALL_SH" --dry-run 2>&1 || true)
  if echo "$out" | grep -q "ship-0.0.0-test-"; then
    pass "dry-run prints versioned bundle filename"
  else
    fail "dry-run prints versioned bundle filename" "$out"
  fi
  if echo "$out" | grep -q "(dry run"; then
    pass "dry-run prints dry-run marker"
  else
    fail "dry-run prints dry-run marker" "$out"
  fi
}

# ── 3. PATH register is idempotent ──────────────────────────────────────────

test_path_idempotent() {
  local tmp_home bashrc
  tmp_home=$(mktemp -d)
  bashrc="$tmp_home/.bashrc"
  : > "$bashrc"

  # Import the path helpers without running main().
  # Strip trailing `main "$@"` invocation so we can source selectively.
  local sourceable
  sourceable=$(mktemp)
  sed '/^main "\$@"$/d' "$INSTALL_SH" > "$sourceable"

  (
    HOME="$tmp_home" \
    SHIP_HOME="$tmp_home/.ship" \
    SHIP_INSTALL_DIR="$tmp_home/.ship/bin" \
    INSTALL_DIR="$tmp_home/.ship/bin" \
    bash -c "
      source '$sourceable'
      path_append_posix '$bashrc'
      path_append_posix '$bashrc'
      path_append_posix '$bashrc'
    "
  ) > /dev/null 2>&1

  local count
  count=$(grep -c '# ship-installer:path' "$bashrc")
  if [[ "$count" == "1" ]]; then
    pass "register_path is idempotent (sentinel appears exactly once)"
  else
    fail "register_path is idempotent" "sentinel found $count times in $bashrc"
  fi

  rm -rf "$tmp_home" "$sourceable"
}

# ── 4. required helpers exist in the script ─────────────────────────────────

test_helpers_present() {
  for fn in register_service register_path \
            open_setup_if_first_run detect_platform detect_kernel; do
    if grep -q "^${fn}()" "$INSTALL_SH"; then
      pass "function present: $fn"
    else
      fail "function present: $fn" "not found"
    fi
  done
}

# Service registration is delegated to `ship daemon install-service` —
# install.sh must not inline a second copy of systemd/launchd logic.
test_service_delegation() {
  if grep -q 'ship daemon install-service' "$INSTALL_SH"; then
    pass "install.sh delegates service registration to ship daemon install-service"
  else
    fail "install.sh delegates service registration" "delegation call not found"
  fi
  for legacy in '\[Unit\]' '\[Service\]' 'launchctl load'; do
    if grep -q "$legacy" "$INSTALL_SH"; then
      fail "no parallel service logic in install.sh" "found inlined: $legacy"
    else
      pass "no parallel service logic in install.sh: $legacy"
    fi
  done
}

# ── 5. install.sh references the new bundle layout ──────────────────────────

test_bundle_layout() {
  if grep -q 'ship-${version_raw}-${platform}' "$INSTALL_SH"; then
    pass "install.sh targets ship-<ver>-<target> bundle"
  else
    fail "install.sh targets ship-<ver>-<target> bundle" "bundle naming not found"
  fi
  for name in ship shipd; do
    if grep -q "install_file .* \"\${INSTALL_DIR}/$name\"" "$INSTALL_SH"; then
      pass "installs binary: $name"
    else
      fail "installs binary: $name" "install_file call not found"
    fi
  done
}

# ── runner ──────────────────────────────────────────────────────────────────

echo "== install.sh tests =="
test_shellcheck
test_dry_run
test_path_idempotent
test_helpers_present
test_service_delegation
test_bundle_layout
echo
echo "Results: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
