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
- [x] **ship-code → bridge** (executors/lifecycle; path-deps Ship isolation/auth/substrate)
- [ ] Thin agent-def / reduce ship-code’s fat Ship edges over time
- [ ] helmd retire (pending, separate)
- [ ] Rename bridge product → fleet when cheap

## Order (no open wounds)

1. Yoke embed solid in shipd — done
2. harness-config-compiler + ship-code in bridge — done (path-deps Ship infra)
3. agent-def-types / engines extract (cut path deps over time)
4. helmd retire; optional rename bridge → fleet

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
- ship-code feature path-deps Bridge; no new harness product code lands in Ship tree
- Tests that assumed “Bridge only” for native must target yoke embed
