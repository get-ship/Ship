---
title: "Ship Set — `set-*` Prefix for Ported Primitives"
description: "ADR: ported `.html.set` primitives use a `set-` tag prefix during the j2/Lit → ship-set migration so both versions ship side by side. Decision is reviewable once the migration completes."
group: ADRs
order: 51
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, components, naming, migration]
---

# Ship Set — `set-*` Prefix for Ported Primitives

## Context

The framework migration ports two surfaces in parallel:

1. **Views** — j2 templates at `.ship/views/<name>/` move to ship-set at `views/<name>/`. The daemon serves each at a different route (`/<name>` vs `/set/<name>`), so they coexist without naming conflicts.
2. **Primitives** — Lit web components at `packages/ship-set/src/web-components/core/<tag>.ts` (registered as `<ship-button>`, `<ship-chip>`, etc.) move to `.html.set` files under `packages/ship-set-components/src/<tag>/<tag>.html.set` with a `<component name="...">` directive.

The view side has no name collision. The primitive side does: `customElements.define(tag, …)` rejects the second registration for the same tag. If a ported `.html.set` declares `<component name="ship-button">` and the Lit `customElements.define('ship-button', ShipButton)` already ran on the same page, one of them silently breaks.

Both stacks ship traffic concurrently during the migration. j2 views compose Lit primitives; ship-set views compose ported primitives; both serve from the same daemon. Until every j2 view has been ported (estimated multiple weeks of work), the Lit registrations cannot be removed.

## Decision

**Ported primitives use a `set-` tag prefix during the migration.** A Lit `<ship-button>` ports to a `<component name="set-button">` `.html.set` file, registered as `<set-button>`.

Concretely:

- Lit primitive: `<ship-button>` — registered by `apps/shipd/assets/ship-elements.js`, used by every existing j2 view.
- Ported primitive: `<set-button>` — registered by `apps/shipd/assets/ship-set-components.js`, used by every `.html.set` view.
- Both bundles are loaded by the daemon's HTML shell. No registration conflict, no runtime collision.

When a `.html.set` view authors a button, it writes `<set-button>`. When a j2 view authors a button, it still writes `<ship-button>`. The two stacks live in separate visual namespaces.

### What gets prefixed

Every primitive ported from `packages/ship-set/src/web-components/core/`. The first one is `<set-status-dot>` (ports `ship-status-dot`); the rest follow the same pattern.

Custom elements that are NOT part of the Lit primitive library — the framework's own template-hook elements (`<use-kbd>`, `<use-shell>`, `<use-click-outside>`) and view-author-defined components in `views/<name>/` — keep their non-prefixed names. The prefix exists *because* of the Lit collision, not as a general convention.

### What happens when the migration completes

Two paths, decided when the time comes:

1. **Drop the prefix.** Once no Lit `<ship-X>` registration remains, every `<set-X>` is renamed to `<ship-X>`. j2 views are gone by then; no caller breaks.
2. **Keep the prefix.** If `set-` becomes a useful identity marker (e.g., agents grep for it as the "this is the ship-set-framework primitive" signal), it stays.

This ADR commits to the prefix-during-migration. The post-migration choice is a separate decision recorded as a follow-up ADR when the j2 stack is fully retired.

## Alternatives considered

### Cut over per primitive

When a primitive is ported, retire the Lit version immediately. Both j2 and `.html.set` views consume the new one.

**Rejected** because every Lit primitive has dozens of j2 callers; verifying that the ported version is byte-for-byte compatible across the full j2 test surface before cutover would slow the port to a crawl. The migration becomes a serial, audit-heavy process per primitive instead of a parallel-friendly one.

### Replace the Lit primitive in place

Ship the ported `.html.set` under the same `ship-X` tag and simply not register the Lit version. j2 views composed with `<ship-button>` would consume the ported version.

**Rejected** because of the j2 testing problem above, plus a runtime concern: many of the Lit primitives have public methods (`focus()`, `setDiagnostics()`, etc.) and event semantics that the j2 + Datastar layer relies on. Validating bivalence per primitive is a non-trivial engagement.

### A shadow-DOM-only namespace (e.g., always-prefixed elements that share a host with Lit components)

Reject — adds complexity for no win. Custom-element registration is global; prefixing the tag is the simplest disambiguation the platform offers.

## Consequences

### Positive

- **Both stacks ship in parallel.** No coordinated cutover required.
- **Authors see the prefix.** A view referencing `<set-button>` is signaling "this is the new framework"; `<ship-button>` is "this is the old one." Visible distinction maps to the migration phase.
- **Per-primitive porting can be parallelized.** Two ports never collide; each one is independent.

### Negative

- **Author muscle memory has to track which prefix applies.** During migration, `.html.set` authors write `set-`; j2 authors write `ship-`. Each stack is internally consistent so the cost is small, but cross-stack porting (taking a j2 view to ship-set) requires retagging every primitive reference.
- **The post-migration rename is real work** (or a deliberate keep-the-prefix decision). Either way it's a future task.
- **Some component names look awkward.** `<set-status-dot>` reads cleanly; `<set-card>` is fine; `<set-app-icon>` and `<set-status-dot>` show that the prefix is non-semantic. Acceptable trade for migration safety.

### Open questions

- **Does the prefix extend to non-primitive ported elements?** The framework's `<use-*>` elements and the dom-package's `<while-loading>` / `<while-error>` aren't Lit primitives — they have no collision. They keep their existing names. This ADR does not change that.
- **Component packs.** The phased-out `ship-component-packs` skill describes pack distribution, which assumed the `ship-` prefix for vendor components. If/when packs ship `.html.set` components, they'll either keep `ship-` (their own namespace, now distinct from Ship's first-party `set-` family) or pick a vendor prefix. Decision deferred until a real pack ships under the new framework.

## References

- The first ported primitive: `packages/ship-set-components/src/set-status-dot/`.
- Migration map: `docs/ship-set-framework/references/docs/status.md`.
- Lit primitives source (the migration source): `packages/ship-set/src/web-components/core/`.
