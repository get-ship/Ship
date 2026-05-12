#!/usr/bin/env bash
# Detect raw `&str` byte-index slicing in sui-compiler walker code.
#
# Six panic bugs found by fuzz in one session traced to `&src[i..j]` /
# `&bytes[i..j]` patterns where the byte indices landed inside multi-
# byte UTF-8 chars. The fixes use `is_char_boundary` / `char_indices()`
# / byte-comparison instead. This lint flags new occurrences in the
# walker hotspots so the pattern doesn't reappear.
#
# Allowed (suppress with `// allow(str-slice)` and a reason):
#   - Patterns guarded by is_char_boundary checks above the slice.
#   - Slices where both ends are 0 or .len() (always boundary-safe).
#
# Run from repo root.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

TARGETS=(
  crates/sui-compiler/src/parse
  crates/sui-compiler/src/check
  crates/sui-compiler/src/emit
  crates/sui-compiler/src/render
  crates/sui-compiler/src/lower
)

violations=0

for d in "${TARGETS[@]}"; do
  [ -d "$d" ] || continue
  hits=$(grep -rEn \
    '&(src|text|value|s|input|raw|right)\[[a-zA-Z_][a-zA-Z_0-9]*\s*\.\.\s*[a-zA-Z_][a-zA-Z_0-9.()]*\]' \
    "$d" 2>/dev/null \
    | grep -vE 'allow\(str-slice\)' \
    || true)
  if [ -n "$hits" ]; then
    if [ "$violations" -eq 0 ]; then
      echo "SUI compiler byte-index str-slice candidates:" >&2
    fi
    echo "$hits" >&2
    violations=$((violations + 1))
  fi
done

if [ "$violations" -gt 0 ]; then
  echo "" >&2
  echo "$violations directory/ies contain byte-index str slicing." >&2
  echo "" >&2
  echo "Each match is a candidate for the UTF-8-boundary panic class" >&2
  echo "that surfaced six bugs in one fuzz session. For each: either" >&2
  echo "  1) Confirm the source is &[u8] (not &str) and the slice stays" >&2
  echo "     bytes-only — append \`// allow(str-slice)\` with a reason." >&2
  echo "  2) Confirm both indices are byte AND char boundaries — append" >&2
  echo "     \`// allow(str-slice)\` with a reason." >&2
  echo "  3) Rewrite using is_char_boundary checks or char_indices()." >&2
  # Currently advisory: a baseline of pre-existing matches needs to be
  # audited site-by-site (logged in follow-ups). Once all are annotated
  # or rewritten, flip this to `exit 1` to make CI fail on new matches.
  exit 0
fi

echo "SUI compiler str-slicing: clean."
