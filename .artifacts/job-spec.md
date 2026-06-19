# Job Spec: complete op-fixture validators + regen all fixtures

**Branch:** job/fixture-validators  (fork from job/transient-events)
**Agent:** rust-lane
**Mode:** autonomous

## Goal
`gen-fixtures` currently aborts on missing op-response validators, which blocks the
event-fixture regen that the retention reclassification (on this base branch) needs.
Add the missing validators + the missing corpus fixture so `gen-fixtures` runs, then
regen all fixtures. This greens the contract roundtrip and clears the pre-existing
`every_typed_method_has_a_validator_and_fixture` gap.

## What to change
1. Add the missing op-response validators to `crates/xtask/src/gen_fixtures/ops.rs`
   (`type_validators()`): `v!(UiQueryResponse)`, `v!(UiReadResponse)`,
   `v!(TerminalSession)`, `v!(StopTerminalResponse)` (types are in `ship_ops_rpc`,
   already `use`d there). Run `cargo run -p xtask -- gen-fixtures` iteratively — it
   reports one missing type at a time; add each until that class of error is gone.
2. Add the missing `ui.scaffold` corpus entry to `proto/fixtures/fixture-seed.json` —
   a minimal VALID sample of `ui.scaffold`'s response type (inspect the proto /
   the Rust type to construct a value that deserializes).
3. Run `cargo run -p xtask -- gen-fixtures` to regen all fixtures. This both writes
   the new ui/terminal op fixtures AND updates the event fixtures' `retention` to
   match the reclassified registry on this branch.

## Constraints
- ONLY add validators + the corpus sample + regen. Do NOT change op or event logic,
  scopes, or retention. Behavioral tests are the contract; don't weaken them. Never push.
- Do NOT touch `gen-events` (separately broken by the payload/ split — out of scope here).

## Acceptance
1. `cargo run -p xtask -- gen-fixtures` exits 0 (no missing-validator / missing-corpus errors).
2. The contract roundtrip + completeness tests pass: `event_fixtures_roundtrip_and_metadata`
   (retention drift gone) and `every_typed_method_has_a_validator_and_fixture`.
3. `just check` green.
4. Commit the validator/corpus additions AND the regenerated fixtures.

---
> Execute yourself. Commit cleanly when acceptance passes. DO NOT push.
