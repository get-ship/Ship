#!/usr/bin/env bash
# Boundary gate (a): SUI runtime / framework TS may not execute `fetch()`
# against Ship daemon URLs.
#
# The SUI framework (`packages/sui/**`) is the rendering layer; it ships
# the runtime that views run on. Per the boundary rule
# (SUI : Deck : Ship :: React : Next : App), SUI names no daemon URL.
# Hosts that need to fetch from the daemon wire the transport into the
# view (HostContext / data-port) — SUI stays URL-free.
#
# The sister Rust gate `/home/madvib/dev/sui/crates/deck/deck-core/tests/boundary.rs` scans
# *runtime* TS (the leak class that slips past boundary_invariants, which
# only sees the deck bundler output, not the sui runtime source).
#
# Forbidden: an executable transport call — `fetch(`, `new EventSource(`,
# `new WebSocket(`, or a dynamic `import(` — whose first argument is a
# string literal naming a daemon route (`/api`, `/_deck`, `/_sui`) inside
# packages/sui/** source. Doc comments and string examples in `@example`
# blocks are allowed (and currently present in await.ts, data.ts,
# signal.ts as type-/API-shape documentation) — we filter them out via
# the line-prefix suppression below.
#
# Run from repo root.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

violations=0

# Walk .ts files under packages/sui and look for executable fetch() calls
# whose first argument is a Ship-style URL. We exclude:
#   - lines inside `/* ... */` block comments and `// ...` line comments
#     (including the `*` continuation lines of JSDoc blocks);
#   - files under `node_modules/`, `dist/`, `example/`, `test/`, `fixtures/`,
#     and `bindings/` (test/host code, not shipped to the runtime).
#
# Heuristic: any non-comment line containing `fetch(`/api` or
# `fetch('/api` or `fetch("/api` (with `/api` as a literal URL path
# prefix, not a placeholder like `/api-docs`) is a violation. A line is
# treated as a comment if it starts (after optional leading whitespace)
# with `//`, `*`, `/*`, or sits inside a JSDoc/open-block-comment region.
#
# We rely on a simple line-by-line suppression: a line that contains the
# token *and* starts with a comment marker is ignored. This is sufficient
# for the runtime source shapes — the only matches we want to catch are
# top-level executable fetch calls (e.g. `await fetch('/api/...')`).
check_file() {
  local file="$1"
  # Filter out:
  #   - line starts with `//`
  #   - line starts with `*` (continuation of a `/* ... */` block)
  #   - line starts with `/*`
  #   - line is `*/` (block closer) or `/**`
  # In all other cases, look for the forbidden pattern.
  local hits
  # Match any executable transport call whose FIRST argument is a string
  # literal naming a daemon route (`/api`, `/_deck`, `/_sui`):
  #   - fetch(            — HTTP
  #   - new EventSource(  — SSE
  #   - new WebSocket(    — WS
  #   - import(           — dynamic import of a route URL
  # The route literal must open with `/api`, `/_deck`, or `/_sui`.
  hits=$(grep -nE "(fetch|new[[:space:]]+EventSource|new[[:space:]]+WebSocket|import)[[:space:]]*\(['\"\`]/(api|_deck|_sui)" "$file" 2>/dev/null \
    | awk -F: '
        {
          # Reconstruct the line text from the remaining fields.
          line = ""
          for (i = 2; i <= NF; i++) {
            if (i > 2) line = line ":"
            line = line $i
          }
          # Trim leading whitespace.
          sub(/^[ \t]+/, "", line)
          # Skip comment lines.
          if (line ~ /^\/\//) next
          if (line ~ /^\*/) next
          if (line ~ /^\/\*/) next
          print $0
        }' || true)
  if [ -n "$hits" ]; then
    while IFS= read -r line; do
      echo "FAIL: $file: $line" >&2
      violations=$((violations + 1))
    done <<< "$hits"
  fi
}

# packages/sui/ moved to the external sui repo post-carve. Scan there.
while IFS= read -r -d '' f; do
  check_file "$f"
done < <(
  find /home/madvib/dev/sui/packages/sui -type f -name '*.ts' \
    -not -path '*/node_modules/*' \
    -not -path '*/dist/*' \
    -not -path '*/example/*' \
    -not -path '*/test/*' \
    -not -path '*/fixtures/*' \
    -not -path '*/bindings/*' \
    -print0
)

if [ $violations -gt 0 ]; then
  echo "" >&2
  echo "SUI runtime no-ship-api gate: $violations violation(s) found." >&2
  echo "The SUI runtime may not execute fetch() against /api or /api/v1." >&2
  echo "Route data through a HostContext/data-port instead — the host" >&2
  echo "owns the daemon URL, not the framework." >&2
  exit 1
fi

echo "SUI runtime no-ship-api gate: clean (no fetch() of /api or /api/v1)."
