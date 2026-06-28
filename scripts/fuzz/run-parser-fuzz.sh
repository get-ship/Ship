#!/usr/bin/env bash
# Time-boxed parser fuzz run for `just fuzz-parser` (default 60s).
#
# Seeds the corpus from every .sui under packages/components so the
# fuzzer mutates from real-world inputs instead of starting from random
# bytes. The corpus dir lives at crates/sui/compiler/fuzz/corpus/parse_fragment
# and is gitignored — this script regenerates it on each run.
set -euo pipefail

DURATION="${1:-60}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
# `crates/sui/compiler/fuzz` lives in the external sui repo post-carve.
# Run this script from the external sui checkout (or update the path
# below to the carve-time absolute).
FUZZ_DIR="/home/madvib/dev/Ship/sui/crates/sui/compiler/fuzz"
CORPUS_DIR="$FUZZ_DIR/corpus/parse_fragment"
# Seeds: every .sui under the external sui repo's packages/components.
SEED_DIR="/home/madvib/dev/Ship/sui/packages/components"

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
# `crates/sui/compiler` lives in the external sui repo.
cd /home/madvib/dev/Ship/sui/crates/sui/compiler
echo "[fuzz] running parse_fragment for ${DURATION}s …"
cargo +nightly fuzz run parse_fragment -- -max_total_time="$DURATION"
echo "[fuzz] seeded $seeded corpus inputs from $SEED_DIR"


