---
title: Roadmap
audience: agent-only
description: Honest gaps in the component-pack story. Agent-only.
---

This page is for agents reasoning about what currently works end-to-end
vs what is scaffolded. It is deliberately not on the public site.

## Gaps

- **No pack-distribution registry.** First-party packs live in
  `packages/ship-set/src/packs/` and are hand-mapped to bundle names in
  `apps/shipd/src/views/packs.rs::pack_bundles`. There is no registry,
  no discovery API, and no install step for third-party packs. Adding
  a pack today means editing that Rust match arm.
- **CEM retrieval for external packs not wired.** Agents consume
  `GET /api/assets/ship-elements.md` (the compact catalog) and
  `custom-elements.json` from the daemon. Those are generated from the
  in-tree `@ship/set` source only. An external pro-pack shipping its
  own CEM has no path into that retrieval surface yet.
- **Pro-pack licensing is product-side TBD.** There is no entitlement
  check, no license server, no per-pack pricing model in code. The
  shape is intentionally open until a real product drives it.
- **No stable contract for third-party pack loading.** Today a pack is
  a bundle name the daemon knows about. When a third-party pack ships,
  we need either a dynamic registration endpoint or a config-file
  convention. Neither exists.
- **Lint coverage for pack authors is partial.** `stale-pack-declaration`
  catches typos in a view's `packs: [...]`. There is no lint that
  enforces the distribution test at author time on a new
  `<ship-*>` element.

## Implications for current agent work

- If an agent is asked to add a new pack end-to-end, edits are needed
  in three places: `packages/ship-set/src/packs/<name>/`, the
  `bundle:<name>` script in `packages/ship-set/package.json`, and the
  `pack_bundles` match arm in `apps/shipd/src/views/packs.rs`.
- If an agent is asked to consume an external pack, today that's not
  supported without landing it in-tree.
- Do not invent commercial licensing APIs or registry endpoints on the
  agent's own initiative. Escalate instead.
