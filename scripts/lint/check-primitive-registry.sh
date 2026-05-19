#!/usr/bin/env bash
# check-primitive-registry.sh
#
# Guard against drift between primitive `<prop>` declarations
# (packages/components/primitives/src/**/*.sui,
#  packages/ui-components/src/**/*.sui)
# and the registry sui-lint's `sui-prop-value` rule loads from
# crates/sui/compiler/primitives.json.
#
# If a primitive adds, removes, or changes an enum-typed prop and the
# author forgets to regenerate, the lint rule silently misses the
# change. This check runs the regen in --check mode and fails when the
# on-disk JSON is stale.
#
# Fix when this fails:
#   cargo run -p sui-lint --example regen_primitives
#   git add crates/sui/compiler/primitives.json
#   git commit

set -euo pipefail

cd "$(dirname "$0")/../.."

cargo run --quiet -p sui-lint --example regen_primitives -- --check
