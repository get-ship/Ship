#!/usr/bin/env bash
# Layer-cut lint for the SUI framework boundary.
#
# Per `.artifacts/sui-arc/00-layers.md`, framework packages
# (`sui-runtime`, `sui-dom`, `sui-eval`, `sui-lsp`) and framework
# crates (`sui-compiler`, `sui-signals`, `sui-lint`) must NOT import
# Ship-host or product-layer code. Otherwise the framework becomes
# a tangle of Ship dependencies and the "pure rendering" promise
# is hollow.
#
# Allowed: cross-framework imports (sui-runtime -> sui-dom, etc.),
# stdlib, third-party deps.
# Forbidden: `@ship/sdk-client`, `@ship/sdk`, `@ship/ship-sdk-client`,
# `@ship/ui-components`, `ship-sui-host`, `ship-sui-server`,
# `ship-sui`, `shipd`.
#
# Run from repo root.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

violations=0

check_ts() {
  local dir="$1"
  local hits
  hits=$(grep -rEn \
    "from\s+['\"]@ship/(sdk-client|sdk|ship-sdk-client|ui-components)['\"]|require\(\s*['\"]@ship/(sdk-client|sdk|ship-sdk-client|ui-components)['\"]" \
    "$dir" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    echo "FAIL: $dir imports a Ship-host or product-layer package:" >&2
    echo "$hits" >&2
    violations=$((violations + 1))
  fi
}

check_rust() {
  local dir="$1"
  local hits
  hits=$(grep -rEn \
    "^\s*use\s+(shipd|ship_sui_host|ship_sui_server|ship_sui|ship_application|ship_platform|ship_substrate)::" \
    "$dir" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    echo "FAIL: $dir imports a Ship-host or product-layer crate:" >&2
    echo "$hits" >&2
    violations=$((violations + 1))
  fi
}

# Framework TS packages.
for d in \
  packages/sui/runtime/src \
  packages/sui/dom/src \
  packages/deck/eval/src \
  packages/deck/lsp/src \
; do
  [ -d "$d" ] && check_ts "$d"
done

# Framework Rust crates.
for d in \
  crates/sui/compiler/src \
  crates/sui/signals/src \
  crates/sui/lint/src \
; do
  [ -d "$d" ] && check_rust "$d"
done

if [ $violations -gt 0 ]; then
  echo "" >&2
  echo "SUI layer-cut violations: $violations directory/ies." >&2
  echo "Framework code must not depend on Ship-host or product layers." >&2
  echo "See .artifacts/sui-arc/00-layers.md (local scratch) for the contract." >&2
  exit 1
fi

echo "SUI layer-cut: clean."
