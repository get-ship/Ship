---
group: Ship Stack
order: 1
title: The Ship Stack
description: The four layers of Ship — substrate, SDK, dogfood, surfaces — and how they compose.
---

# Ship Stack — Map and Positioning

The stack exists to separate concerns that move at different speeds. A bug in the marketing site shouldn't block a runtime release; a runtime migration shouldn't ship without the SDK contracts catching up first.

## Subsystem Inventory

### L1 — Substrate

| Subsystem     | Location                          | Docs                                       |
| ------------- | --------------------------------- | ------------------------------------------ |
| Kernel        | `crates/core/ship-kernel/`        | [Kernel](/reference/architecture/kernel/)      |
| Runtime       | `crates/core/runtime/`            | [Runtime](/reference/runtime/)                 |
| Event store   | `runtime/persistence/`            | [Persistence](/reference/runtime/persistence/) |
| Daemon        | `apps/shipd/`                     | [Runtime](/reference/runtime/)                 |
| Compiler      | `crates/core/compiler/`, `packages/compiler/` | [Compiler](/reference/compiler/)   |
| Supervisor    | `apps/shipd/supervisor/`          | [Runtime](/reference/runtime/)                 |
| Mesh          | `runtime/services/mesh.rs`        | [Runtime](/reference/runtime/)                 |
| CLI transport | `apps/ship-cli/`                  | [CLI](/reference/cli/)                         |
| MCP transport | `apps/mcp/`                       | [MCP Tools](/reference/mcp-tools/)             |

### L2 — Platform SDK

| Subsystem       | Location                  | Docs                       |
| --------------- | ------------------------- | -------------------------- |
| Pipeline spec   | `crates/core/ship-jobs/`  | ship-workflow              |
| SQL service     | (TBD — audit gap)         | ship-collections           |
| Skill compile   | `runtime/agents/skill*`   | ship-skills                |
| ship-set        | `packages/ui/`            | ship-studio                |
| ship-sdk        | `packages/sdk/`           | (audit gap)                |
| Primitives      | `packages/primitives/`    | (audit gap)                |
| Schemas         | `schemas/`                | ship-schema-reference      |

### L3 — First-Party Dogfood

| Subsystem   | Location             | Docs                |
| ----------- | -------------------- | ------------------- |
| Views       | `.ship/views/`       | ship-studio         |
| Skills      | `skills/`, `docs/`   | ship-skills         |
| Pipelines   | `.ship/pipelines/`   | ship-workflow       |
| Collections | `.ship/collections/` | ship-collections    |
| Triggers    | `.ship/triggers/`    | ship-triggers       |

### L4 — Surfaces

| Surface           | Location                | Docs           |
| ----------------- | ----------------------- | -------------- |
| Marketing site    | `apps/docs/` or similar | (audit gap)    |
| Daemon shell      | `apps/shell/`           | ship-studio    |
| Studio            | `apps/web/`             | ship-studio    |

Cells marked *(audit gap)* are known missing docs — to be filled in during the doc audit pass.

## Reliability Sequencing

The launch focus is reliability and quality. Work is sequenced bottom-up:

1. **L1 correctness** — event store invariants, kernel routing, supervisor restart policies, compiler determinism. A failure here corrupts state and is not recoverable from L2+.
2. **L2 contracts** — documented, versioned, tested at the boundary. A breaking change here breaks every consumer.
3. **L3 dogfood coverage** — each L1/L2 capability should have at least one first-party consumer in L3. Gaps here mean we find bugs from user reports instead of our own usage.
4. **L4 polish** — surfaces iterate fastest. Reliability here is about not regressing, not about never breaking.

When dogfooding reveals a substrate issue (common with collections/views), the fix goes in L1/L2 — not patched around in L3. Track these findings as ADRs so the substrate change has context.

## What Belongs in `docs/` vs `.ship/`

- **`docs/ship-*`** — stable, public, "what Ship is." Architecture, references, concepts, tutorials. Checked into git, rendered on the docs site.
- **`.ship/` collections + views** — mutable, internal, "what we're shipping." Launch readiness, gap tracking, todos, dogfood findings. Branch-aware, not public.

If a document is answering "how does X work" it belongs in `docs/`. If it's answering "what's the status of X" it belongs in a `.ship/` collection.
