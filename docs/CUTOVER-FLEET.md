# Cutover: harness out of Ship → engines + fleet

**Status:** in progress. Do not degrade native agent runs.

## Target shape

| Piece | Owns | Does not |
|-------|------|----------|
| **yoke** | Headless native rig (loop, gate, durable, code-mode) | Bridge, shipd, UI |
| **fleet** (today `bridge` repo + ship-code) | Multi-harness control plane, compile/drivers, daemon/CLI | Ship spaces/product |
| **shipd** | Host product; **embeds yoke**; maps session events to kernel | Harness drivers (ship-code) |
| **agent-def-types / agent-schema** | Bottom-of-pyramid agent + session types | ship-substrate |

Nothing else depends on the `ship` repo. Shared crates **leave ship** toward engines/fleet.

## Done / current

- [x] Floki crate deleted; shipd path-deps `bridge/crates/yoke`
- [x] Product native path = **in-process yoke embed** (`apps/shipd/src/yoke/embed`) — no bridge process required
- [x] `FlokiRunFn` → `YokeRunFn`; module `floki` → `yoke`
- [x] Optional `YOKE_RPC=1` → Connect to fleet/bridge (non-default)
- [ ] Yoke smoke / multi-provider hardened (engines workstream)
- [ ] `agent-def` fat split → thin types for yoke + fleet
- [x] **harness-config-compiler → bridge** (path-deps Ship `stack-model`; schemas still Ship)
- [ ] **ship-code (executors) → bridge** — Bridge *is* ship-code; do not co-move stack-model / substrate- [ ] ship-agents loses ship-code feature graph on default shipd
- [ ] helmd retire (pending, separate)
- [ ] Rename bridge product → fleet when cheap

## Order (no open wounds)

1. Yoke embed solid in shipd (this doc’s “done” for product path)
2. agent-def-types (unblock clean ship-code move)
3. Move **ship-code** (executors / process assembly) into bridge — Bridge *is* that product. Do **not** co-move stack-model, substrate, isolation, auth.
4. ship-agents: native = yoke port only; 3P via fleet process/RPC if needed later
5. Shared stack/agent types leave Ship only into an engines/substrate home — never “because bridge needed a path”

## Repo gravity (direction)

```
engines/   (rename candidate — not “ship”)
  yoke, agent-schema, durable-execution, ship-js?, eval harness

fleet/     (evolve bridge/)
  fleetd, CLI, protocol, ship-code drivers, harness-config-compiler
  depends on engines

ship/      (product host only)
  shipd embeds yoke; no harness drivers
```

Polyrepo motivation: **merge blast radius** (workers changing the wrong surface), not “agents module is toxic.”

## Env

| Var | Meaning |
|-----|---------|
| (default) | In-process yoke embed |
| `YOKE_RPC=1` | Force Connect client to fleet/bridge socket |
| `BRIDGE_YOKE_SOCKET` / `SHIP_YOKE_SOCKET` | Socket when using RPC path |

## Functionality bar

- shipd default build: native `yoke`/`floki` harness sessions run without `bridge serve`
- ship-code feature still builds (until move completes); no new harness work lands in ship
- Tests that assumed “Bridge only” must target embed
