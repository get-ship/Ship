---
name: ship-component-packs
stable-id: ship-component-packs
description: "Authoring reusable web-component packs for Ship — for pack vendors and internal teams outgrowing view primitives. NOT for happy-path view authoring; see ship-sdk first. PARTIALLY PHASING OUT — the pack distribution mechanics survive, but the `.ship` SFC format inside packs is dead; new packs ship `.html.set` components per the ship-set-framework skill."
tags: [ship, packs, web-components, components, authoring, distribution, phasing-out]
authors: [ship]
---

> **⚠ Partially phasing out.** Pack distribution (`pack.toml`, lockfile,
> registry, install flow) survives unchanged. The **format of components
> inside packs** is being replaced — `.ship` SFC files are dead; the target
> is `.html.set` per the **`ship-set-framework`** skill. Existing packs of
> `.ship` components remain installable until ported. See
> `docs/ship-set-framework/references/docs/status.md` for the migration map.

# Ship Component Packs

The **escape hatch**. Ship's default authoring surface is view primitives
(`.html.j2` + SDK macros) composed with `<ship-*>` elements from
`@ship/set`. For 90%+ of work you never build a component. Reach for a
pack only when you ship a pro-pack for external distribution, run an
internal app that genuinely outgrew primitives + slots, or need to wrap
a heavy third-party engine that isn't already an island.

If you are authoring a view, start at
[ship-sdk](/reference/sdk/). If you are authoring library components,
the charter lives in `.ship/skills/ship-components/SKILL.md`. This
skill is the **packaging and distribution** layer on top of that
charter.

## Reference pages

- [Component Packs overview](./references/docs/index.md) — the core
  message and when packs are the right tool.
- [When to author](./references/docs/when-to-author.md) — the decision
  rule, with do-not and do tables.
- [Authoring a component](./references/docs/authoring.md) — directory
  layout, Lit template, the distribution test, tokens, events.
- [Manifest & distribution](./references/docs/manifest-and-distribution.md) —
  `packs: [...]` opt-in, bundle scripts, CEM, pro-pack packaging.
- [Testing](./references/docs/testing.md) — what every pack component
  ships on day one.
- [Roadmap](./references/docs/roadmap.md) — agent-only. Honest gaps in
  pack distribution and discovery.
