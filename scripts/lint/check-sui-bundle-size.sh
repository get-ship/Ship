#!/usr/bin/env bash
# Bundle-size guard for SUI framework bundles.
#
# Bundles `sui-runtime` and `sui-dom` from source, measures raw + gzip
# sizes, and fails if any budget is breached. Budgets are intentionally
# generous starting points — tighten as the framework stabilizes.
#
# Run from repo root.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Budgets (gzip bytes). Updating: bump only with a written rationale.
declare -A BUDGETS=(
  [sui-runtime.js]=20000
  [sui-dom.js]=4000
)

# Produce the framework bundles via the canonical in-process Rolldown bake
# (#163) — the same engine the daemon runs. Output lands in .deck/framework/.
cargo run -p deck-cli --quiet -- build-assets >/dev/null 2>&1

failures=0
echo "sui bundle sizes:"
for name in "${!BUDGETS[@]}"; do
  path=".deck/framework/$name"
  if [ ! -f "$path" ]; then
    echo "  FAIL: $name not produced at $path" >&2
    failures=$((failures + 1))
    continue
  fi
  raw=$(stat -c%s "$path")
  gz=$(gzip -c "$path" | wc -c)
  budget="${BUDGETS[$name]}"
  status="ok"
  if [ "$gz" -gt "$budget" ]; then
    status="OVER (budget=${budget} gz)"
    failures=$((failures + 1))
  fi
  printf "  %-22s raw=%-9s gz=%-9s budget=%-9s %s\n" \
    "$name" "${raw}B" "${gz}B" "${budget}B" "$status"
done

if [ "$failures" -gt 0 ]; then
  echo "" >&2
  echo "Bundle-size budget breached." >&2
  echo "Either land the change cleanly under budget, or bump the budget" >&2
  echo "in $0 with a written rationale in the same commit." >&2
  exit 1
fi
echo "Within budgets."
