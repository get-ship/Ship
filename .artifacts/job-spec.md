# Job Spec: collections CLEARING — dead service, dual DDL, codegen drift, contract gaps

**Branch:** job/collections-clearing  (fork from launch/unified-HEAD)
**Agent:** rust-lane
**Mode:** autonomous

## Goal
Clear verified dead code, duplication, and unambiguous contract/codegen bugs in the
collections module. Removal + collapse + mechanical fixes only. Findings verified in
`.artifacts/platform-services-audit.md` (collections section) — RE-VERIFY each against the
compiler + caller search before acting. Never guess — STOP + `note(out-of-scope)` on surprise.

## File scope
- `crates/core/ship-platform/crates/ship-collection/**`
- `crates/xtask/src/gen_collection_types.rs`
Do NOT touch: `apps/shipd/src/ship_runtime/init/mod.rs` (the dual-connection item is assigned
to the artifacts job), deck crates, ship-ui beyond what's required.

## CLEAR / COLLAPSE / FIX (verify each first)
1. **Delete `DecisionService`** — `ship-collection/src/decisions.rs` (~330 lines) + its test
   file + `pub mod decisions` in `lib.rs`. Zero runtime callers; its doc comment falsely claims
   `floki MementoTool` integration (MementoTool does not exist anywhere). This removes that
   false-doc finding too. Confirm zero callers across the workspace first.
2. **Collapse dual DDL** — `create_schema`/`update_schema` run DDL twice: once in
   `service.rs:244-247` and once in the projection (`item_projection.rs:61-68`). Remove the
   direct `ensure_collection_table`/`persist_schema` calls from `service.rs`; make
   `item_projection.rs` the sole DDL writer (its module header already claims this invariant).
3. **Codegen column drift (bug)** — `gen_collection_types.rs:33-39` `SYSTEM_COLUMNS` emits
   names (`_id/_created/_updated/_actor/_space`) that do not match the SQLite schema
   (`item_projection.rs:162-170`: `id/created_at/updated_at/space_id`), and `_actor` has no DB
   column at all. Align the codegen to the actual column names and drop `_actor`. Regenerate
   (`cargo run -p xtask -- gen-collection-types`). Add a parity test asserting each
   `SYSTEM_COLUMNS` name exists in `PRAGMA table_info(collection_<fixture>)`.
4. **`List` returns `item_count: 0` (bug)** — `wire.rs:107-109` hardcodes 0; the
   `count_items_for_wire` helper (`wire.rs:308`) already exists and is used by `get`/`update`.
   Wire it into the `list` handler.
5. **`RESERVED_IDS` governance** — add `"view_snapshots"` to `RESERVED_IDS` (`types.rs:71`);
   add a test enumerating all known system collection IDs and asserting each is reserved.

## EXPLICITLY OUT OF SCOPE (flagged separately — do NOT touch)
- `QueryItems.named_query` stub (`wire.rs:153`) — proto/SDK contract decision.
- `Watch(scope=item)` broken snapshot (`patch.rs:52`) — broken feature, separate.
- The dual `DataService` connection in `init/mod.rs` — assigned to the artifacts job.

## Constraints
Verify-then-act; pre-launch (delete, no shims); never push; never widen scope. STOP+note on
any surprise live consumer (e.g. if `DecisionService` turns out to be wired somewhere).

## Acceptance
1. `cargo build -p ship-collection -p shipd` green.
2. `cargo run -p xtask -- gen-collection-types` runs clean; regenerated TS committed.
3. The new parity test + RESERVED_IDS test pass; `cargo build --workspace` / `just check` green.
4. Commit cleanly. DO NOT push.
