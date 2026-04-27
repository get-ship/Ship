#!/usr/bin/env bash
# check-leaf-portability.sh
#
# Enforce modular-runtime-01 spec acceptance: each leaf crate must compile
# for a portable target (wasm32-unknown-unknown today; aarch64-linux-android
# also works once that target is installed in CI).
#
# A regression here means someone added a non-portable dep — `std::fs`,
# `std::process`, `tokio` (with rt-multi-thread), `dirs`, or anything that
# imports the daemon's runtime. The lint catches it at PR time, not at the
# point a cloud worker or browser embed tries to use the crate.

set -u

cd "$(dirname "$0")/../.."

LEAF_CRATES=(ship-pipelines)
TARGET="wasm32-unknown-unknown"

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "check-leaf-portability: $TARGET not installed; skipping"
    echo "  install with: rustup target add $TARGET"
    exit 0
fi

violations=0
for crate in "${LEAF_CRATES[@]}"; do
    if ! cargo check --quiet --target "$TARGET" -p "$crate" >/dev/null 2>&1; then
        echo "check-leaf-portability: $crate fails to compile for $TARGET"
        echo "  re-run: cargo check --target $TARGET -p $crate"
        violations=$((violations + 1))
    fi
done

if [ $violations -gt 0 ]; then
    echo "check-leaf-portability: $violations leaf crate(s) failed portability check"
    exit 1
fi
echo "check-leaf-portability: ok ($TARGET)"
exit 0
