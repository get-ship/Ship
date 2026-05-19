# Job Spec: fix clippy warning in async_block.rs

**Branch:** `job/async-block-clippy`
**Agent:** rust-lane
**Mode:** autonomous

## Goal
`just lint` (clippy -D) flags `crates/sui/compiler/src/parse/template/blocks/async_block.rs:122` — "calls to `push` immediately after creation." Pre-existing from the kill-legacy {#async} work; surfaced by the pipelines-history job running full-workspace lint. Fix it.

## What to change
- `crates/sui/compiler/src/parse/template/blocks/async_block.rs:~122` — replace the `let mut v = Vec::new(); v.push(...)` pattern with `vec![...]` (or whatever the idiomatic form clippy wants).
- Run `cargo clippy -p sui-compiler -- -D warnings` to confirm clean.

## Acceptance criteria
1. `just lint` passes clean (no async_block.rs warning).
2. `cargo test -p sui-compiler` still green (228+ tests).
3. The {#async} golden fixtures still pass.

## Out of scope
Anything beyond the single clippy fix.

> Tiny. One commit. complete_workspace + mesh_broadcast when done.
