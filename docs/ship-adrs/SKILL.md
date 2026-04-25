---
name: ship-adrs
stable-id: ship-adrs
description: Architecture Decision Records for Ship. Each ADR captures context, decision, consequences, and alternatives for a choice that constrains future work. Consult this skill before proposing changes that contradict a ratified decision.
tags: [reference, decisions, architecture]
authors: [ship]
---

# Ship ADRs

Durable record of architectural decisions. One markdown file per decision in `references/docs/`. Current home until the `adrs` collection lands (see `collections-as-data-layer`), at which point records migrate and this skill exposes query tools.

## When to write one

A decision qualifies as an ADR when:

- It has real alternatives that would have been viable.
- Future contributors — human or agent — need to understand *why*, not just *what*.
- Reversing it would cost meaningful work.

Minor implementation choices don't qualify. Naming a function, picking a crate for a one-off utility, etc. — skip.

## File conventions

- **Filename:** kebab-title. Stable identity, grep-friendly. No numbers, no dates in the name — git tracks history, frontmatter carries metadata.
- **Frontmatter (YAML):** required fields `status`, `date`. Optional: `supersedes`, `superseded-by`, `tags`. A `deciders` field is available when a specific group ratified the decision — omit it for individual authorship (git authorship is already recorded).
- **Status values:** `proposed`, `accepted`, `superseded`, `deprecated`.
- **Body structure:** `Context` → `Decision` → `Consequences` (Positive / Negative) → `Alternatives Considered`. Optional sections: `Migration`, `How to Measure Success`.

## Example frontmatter

```yaml
---
status: accepted
date: 2026-04-19
supersedes: []
tags: [storage, collections]
---
```

## Lifecycle

- Draft with `status: proposed`. Iterate in review.
- Flip to `accepted` when the decision is ratified.
- Superseding a prior ADR: the new file sets `supersedes: [old-slug]`; the old file sets `superseded-by: new-slug` and `status: superseded`. Keep the old file — decisions don't get deleted, only superseded.

## Future: ADRs as a collection

When the `adrs` collection ships (see `collections-as-data-layer`), these markdown files migrate to collection records. Frontmatter fields map to collection fields; body becomes the record's markdown body. Agents will query via `list_adrs` / `get_adr` rather than reading files. Until then: plain markdown, git-tracked.
