#!/usr/bin/env bash
# check-module-docs.sh — fail if any workspace lib.rs >100 LOC has zero //! lines.
#
# Every lib crate with real content must have at minimum a one-line module doc
# (//!) explaining what the crate is and which layer it lives at. This prevents
# silently undocumented growth: a lib.rs that grows past 100 lines without a
# module header is caught here before it lands.
#
# Threshold: >100 lines. Files under 100 lines are tiny re-export or shim crates
# where a header would exceed the content — skip them.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail=0
found=0

while IFS= read -r -d '' f; do
    lines=$(wc -l < "$f")
    if [ "$lines" -le 100 ]; then
        continue
    fi
    doc_lines=$(grep -c "^//!" "$f" 2>/dev/null; true)
    if [ "${doc_lines:-0}" -eq 0 ]; then
        rel="${f#"$REPO_ROOT/"}"
        echo "  ✗ $rel ($lines lines, 0 //! doc lines)" >&2
        found=$((found + 1))
        fail=1
    fi
done < <(find "$REPO_ROOT/crates" "$REPO_ROOT/apps" "$REPO_ROOT/packages" \
    -name "lib.rs" \
    -not -path "*/target/*" \
    -print0 2>/dev/null)

if [ "$fail" -ne 0 ]; then
    echo "check-module-docs: $found lib.rs file(s) >100 LOC with no //! module doc" >&2
    echo "  Add at least one //! line describing what the crate is and its layer." >&2
    exit 1
fi

echo "check-module-docs: ok"
