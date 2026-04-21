---
name: ship-stack
stable-id: ship-stack
description: Use for the 30,000-foot view of Ship — the four layers of the product, how they stack, and where each subsystem lives. Start here before diving into any single area.
tags: [ship, architecture, overview, stack]
authors: [ship]
---

# The Ship Stack

Ship is a stack, not a single product. Four layers, bottom-up: **L1 Substrate**, **L2 Platform SDK**, **L3 First-Party Dogfood**, **L4 Surfaces**. Each has a different audience, cadence, and quality bar.

Reliability posture: **L1 must be boring. L2 must be documented. L3–L4 are where we take risk.**

## Layer 1 — Substrate

The core platform. Event-sourced, service-meshed, Axum-hosted. This layer is invisible to most users and must stay that way.

See [ship-architecture](../ship-architecture/SKILL.md) for crate/app structure, [ship-runtime](../ship-runtime/SKILL.md) for daemon internals, [ship-compiler](../ship-compiler/SKILL.md) for the compile pipeline.

## Layer 2 — Platform SDK

The contract layer. What third parties build against: pipeline spec, SQL service, skill compilation, `@ship/primitives`, the ship SDK, and JSON schemas.

See [ship-skills](../ship-skills/SKILL.md), [ship-schema-reference](../ship-schema-reference/SKILL.md), [ship-workflow](../ship-workflow/SKILL.md).

## Layer 3 — First-Party Dogfood

What we ship in the box. Views, skills, pipelines, and collections authored by us, using the same SDK third parties use. This is how we find bugs in L1/L2 before users do.

See [ship-collections](../ship-collections/SKILL.md), [ship-agents](../ship-agents/SKILL.md), [ship-triggers](../ship-triggers/SKILL.md).

## Layer 4 — Surfaces

User-facing output: the marketing site and the daemon-served shell (Studio). These consume L1–L3 and have the fastest iteration cycle.

See [ship-studio](../ship-studio/SKILL.md), [ship-cloud](../ship-cloud/SKILL.md), [ship-registry](../ship-registry/SKILL.md).

## How the Layers Talk

- **L4 → L1**: Surfaces talk only to the daemon over HTTP. Never directly to the event store.
- **L3 → L2**: Dogfood consumes the SDK. If dogfood reaches into runtime internals, that's a bug — lift the need into L2.
- **L2 → L1**: SDK surfaces are versioned contracts. Breaking changes require an ADR.
- **L1**: Self-contained. No upward dependencies.

## References

- [Stack map and positioning](references/docs/index.md) — layered diagram with subsystem inventory per layer, plus how we sequence reliability work.
