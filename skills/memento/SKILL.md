---
name: memento
description: Record durable decision records — corrections, operational rules, mistakes, rejected ideas, tooling/dependency choices, product calls. Reinforces on repeat, supersedes on change, surfaces high-signal candidates for promotion. Use to persist what was decided so future agents don't relitigate it.
stable-id: memento
---

> **EXPERIMENTAL — NOT A PRODUCT SURFACE.** This skill is a local prototype.
> Its store is a per-machine sqlite file at `~/.ship/memento.db` (override
> `MEMENTO_DB`) driven by a stdlib Python script. Nothing here syncs, partitions
> by scope, or rides the substrate. Use it to dogfood the decision-record idea
> locally — do not build product features against `memento.py` or its DB shape.
>
> **Product home (ratified decision).** Memento's product form is a
> **Space-scoped `decisions` collection over the substrate**, with
> `record` / `query` / `supersede` / `audit` as **SDK ops** — event-sourced,
> CRDT-merged, synced like every other collection. The local sqlite prototype
> below is explicitly NOT that; it exists only to validate the shape before the
> substrate-backed form is built. When the product form lands, this skill
> retires or becomes a thin client over the `decisions` collection.

## When to record

Record only when one of these seven triggers fires:

- **correction** — a human or gate corrected your approach
- **operational** — a standing rule about how work gets done here
- **mistake** — something that broke or wasted effort; capture so it isn't repeated
- **rejected** — an idea considered and explicitly turned down (and why)
- **tooling** — a tool/workflow choice that constrains future work
- **dependency** — a library/version/service commitment
- **product** — a product or UX decision with alternatives

Fidelity rule: record **what was decided, stated, or done** — never inferred intent, never every passing thought. One record per decision. If it isn't a settled fact, don't write it.

## How

`python3 scripts/memento.py <subcommand>`

- **record** — log a decision; identical kind+title (case-insensitive, active) reinforces instead of duplicating, bumping confirmations and conviction one notch.
  `record --kind operational --title "Use WAL mode" --conviction high --tags db,sqlite`
- **query** — pull terse one-liners, ranked by conviction then confirmations.
  `query --kind tooling --min-conviction medium --tag db --limit 20`
- **get** — full record (with body) as pretty JSON by id.
  `get a1b2c3d4e5f6...`
- **supersede** — replace an existing record with a corrected one; links old↔new, no dedup.
  `supersede <OLD_ID> --kind correction --title "New decision" --body "why it changed"`
- **audit** — operator review of active records ranked by confirmations, labeled STRONG candidate / candidate / weak for promotion.
  `audit --limit 30`
