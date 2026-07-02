#!/usr/bin/env bash
# Settings single-owner guard.
#
# The unified `SettingsService` (crates/core/ship-platform/crates/ship-config)
# is the ONLY settings path. This fails if a parallel settings store or reader
# is re-introduced:
#   1. the dead jsonc file store (settings.jsonc / local.jsonc)
#   2. the dead LibrarySettings / LocalSettings structs
#   3. any caller of the `daemon_settings` Runtime backing outside ship-config
#
# "How do I store a setting?" has exactly one answer: the SettingsService,
# pick a scope. This guard keeps it that way.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

CONFIG='crates/core/ship-platform/crates/ship-config/'
fail=0
emit() { echo "SETTINGS-GUARD FAIL: $1" >&2; fail=1; }
notest() { grep -viE '/tests?/|_test\.rs|/tests_'; }

hits=$(rg -n --type rust 'settings\.jsonc|local\.jsonc' | notest | grep -v 'pipelines/local\.jsonc' || true)
[ -n "$hits" ] && { emit "settings.jsonc/local.jsonc still referenced — the file store is dead"; printf '%s\n' "$hits" >&2; }

hits=$(rg -n --type rust 'LibrarySettings|LocalSettings' | notest || true)
[ -n "$hits" ] && { emit "LibrarySettings/LocalSettings still referenced — the jsonc structs are dead"; printf '%s\n' "$hits" >&2; }

# ship-isolation is experimental ship-code with a KNOWN, documented infraction
# (it reads/writes isolation.* config straight to daemon_settings, bypassing the
# SettingsService). Flagged in ship-isolation/src/decision.rs; exempted here
# until the ship-code isolation work routes it through the service.
hits=$(rg -l --type rust 'daemon_settings::' | grep -vF "$CONFIG" | grep -vF 'ship-isolation' | notest || true)
[ -n "$hits" ] && { emit "daemon_settings:: called outside ship-config — go through SettingsService"; printf '%s\n' "$hits" >&2; }

[ "$fail" -eq 0 ] && echo "settings single-owner: OK"
exit "$fail"
