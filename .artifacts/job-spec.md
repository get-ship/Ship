# Theme-B Wave 2 + Settings-S4 + B1 — ORCHESTRATOR (omp + subagents)

Orchestrator: 5 work-items. Fan subagents, commit per item, build+regen green, NEVER push.
Verify-then-act (locate by symbol — lines drift); re-verify each, skip+report if it doesn't hold.

COORDINATION — serialize these (do NOT parallel-edit the same file):
- EVENT REGISTRY: B2, B14, S4 all remove event registrations (kernel.actor.*, mailbox_full,
  space.setting.changed). Their edits to payload_registry / event_types / event_registrations /
  payload modules + xtask fixtures MUST be serialized.
- PROTO + REGEN: B12 and S4 both edit protos (ui.proto, space.proto) and need an SDK/contract/
  fixture regen. Do ALL proto edits first, then ONE regen pass (gen_contract, gen_fixtures, SDK
  proto:generate), committed separately.
- Do S4 BEFORE B1 (B1 reads the unified settings_overlay S4 makes canonical for SPACE).

## B2 — CUT actor snapshot/restore (dead + corrupting)
ship-substrate: remove `KernelRouter::snapshot/suspend/restore` (events/kernel_lifecycle.rs) — KEEP
`handle_actor_crash`. Remove `ActorSnapshot` + `event_stats` (persistence/snapshot.rs). Delete
snapshot_tests.rs + snapshot_edge_tests.rs. Remove the orphaned `kernel.actor.snapshot/suspended/
restored` event types + `KernelActorSnapshot/Suspended/Restored` payloads (payload_registry.rs +
ship-types payload/kernel.rs) + the xtask fixture. (Registered Stable — update the registry.)

## B12 — REMOVE dead UI session writes
proto/ship/ops/v1/ui.proto: remove PatchSession/AppendSession/SignalSession RPCs +
ViewPatchReq/ViewAppendReq/ViewSignalReq messages. apps/shipd/src/rpc/ops/ui_wire.rs: remove the 3
no-op closures. apps/shipd/src/rpc/ops/tests.rs: remove their contract-presence tests (~195-197,
~267-269). (Regen in the proto pass.)

## B14 — CONTAIN mailbox_full event
ship-kernel: remove the `mailbox_full` Stable event registration + const (lib.rs ~:36) — no emitter
(backpressure is fast-follow). Drop it from the event registry.

## S4 — collapse the SPACE settings tier onto the unified overlay
SPACE settings must live ONLY in `settings_overlay` (the unified SettingsService store):
- space.proto: remove `SetSetting` RPC + `SpaceSettingValue` + `SetSpaceSettingReq` +
  `SpaceInfo.settings_json`.
- Delete ship-space/src/config.rs (legacy `space_overlay` writer/reader) + ship-config/src/
  projection.rs (SpaceSettingsProjection) + its `space_settings` migration .sql.
- Repoint the effective-settings reader (ship-space info::compose / apps/shipd/src/rpc/space.rs) to
  read `settings_overlay` via `ship_config::overlay` instead of `space_overlay`.
- Drop the legacy `space.setting.changed` event; the unified `settings.changed` (SettingsService.
  Watch) is THE surface. Repoint SpaceWire.Watch(space) prefix `space.setting.` → `settings.changed`.
- Trim registration/export sites (ship-platform lib.rs, ship-space lib.rs, event_registrations,
  event_types, payload, integrity.rs).
- DATA: add a same-db per-space.db copy migration `INSERT OR REPLACE INTO settings_overlay
  SELECT ... FROM space_overlay` so existing overrides survive; OR skip if you confirm space_overlay
  is empty (pre-launch likely zero rows) — note which you did.
- Tests: port/delete ship-space/tests/behavior.rs (config::set / info-overlay / "exactly one
  space.setting.changed") + ship-config projection in-file tests. Keep the green ship-config/tests/
  behavior.rs (unified path).

## B1 — wire the search opt-out onto the unified overlay (first real consumer)
apps/shipd/src/rpc/search.rs: rewrite `opted_out_spaces` to read the unified overlay per candidate
space — for each candidate's space_store, `ship_config::overlay::get(&*store, "search.global").await
.and_then(|v| v.as_bool()).unwrap_or(true)` (unset = included; only explicit `false` opts out).
Remove the dead SpaceService.info / Value::Null / space_overlay path. The fan-out gate
`run_global(..., |id| !opted_out.contains(id))` already exists + is tested — don't touch it. Keep the
key name `search.global`.

## Acceptance
One commit per item + one regen commit. Proto regen clean (no drift). `cargo check --workspace
--exclude ship-host --features unstable` green; affected tests pass (ship-substrate, ship-kernel,
ship-types, ship-config, ship-space, ship-search, shipd, xtask). The ship-search global-opt-out test
passes with the new reader. NO push.
