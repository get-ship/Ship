#!/usr/bin/env bash
# Time-boxed parser fuzz run for `just fuzz-parser` (default 60s).
#
# Seeds the corpus from every .sui under packages/ui-components/src so the
# fuzzer mutates from real-world inputs instead of starting from random
# bytes. The corpus dir lives at crates/sui/compiler/fuzz/corpus/parse_fragment
# and is gitignored — this script regenerates it on each run.
set -euo pipefail

DURATION="${1:-60}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
FUZZ_DIR="$ROOT/crates/sui/compiler/fuzz"
CORPUS_DIR="$FUZZ_DIR/corpus/parse_fragment"
SEED_DIR="$ROOT/packages/ui-components/src"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "cargo-fuzz not installed. Run: cargo install cargo-fuzz" >&2
    exit 2
fi
if ! rustup toolchain list 2>/dev/null | grep -q '^nightly'; then
    echo "nightly toolchain missing. Run: rustup toolchain install nightly" >&2
    exit 2
fi

mkdir -p "$CORPUS_DIR"
seeded=0
while IFS= read -r -d '' sui; do
    rel="${sui#"$ROOT"/}"
    name="$(printf '%s' "$rel" | tr '/' '_')"
    cp -f "$sui" "$CORPUS_DIR/$name"
    seeded=$((seeded + 1))
done < <(find "$SEED_DIR" -name '*.sui' -type f -print0)
echo "[fuzz] seeded $seeded corpus inputs from $SEED_DIR"

cd "$ROOT/crates/sui/compiler"
echo "[fuzz] running parse_fragment for ${DURATION}s …"
cargo +nightly fuzz run parse_fragment -- -max_total_time="$DURATION"
