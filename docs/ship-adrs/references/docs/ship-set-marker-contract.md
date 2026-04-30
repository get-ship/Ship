---
title: "Ship Set — SSR Marker Contract (`data-set-*` Attributes)"
description: "ADR: the SSR↔attach handshake uses a `data-set-*` attribute family on rendered elements. No sidecar manifest, no opaque IDs."
group: ADRs
order: 53
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, ssr, hydration, markers, framework]
---

# Ship Set — SSR Marker Contract (`data-set-*` Attributes)

## Context

Ship Set is server-rendered first: `shipd` evaluates a `.html.set` template against initial signal values and emits HTML that is meaningful before any JavaScript runs. A small client attach runtime then walks the rendered DOM, finds reactive bindings, wires event handlers, opens SSE channels for `<data live>`, and runs controller lifecycle.

The seam between server-emitted HTML and the client attach runtime is the **marker contract**: how does the client know that *this* `<button>` carries an `on:click={incr}`, that *this* text node is a `{count()}` interpolation, that *this* `<ul>` is a `{#for}` boundary keyed by `id`?

Three candidate shapes were on the table:

1. **`data-set-*` attribute family.** Each reactive binding emits a `data-set-<kind>="<expr-key>"` attribute on the host element (or a wrapping marker comment when the binding has no host).
2. **Opaque element IDs + sidecar JSON manifest.** Each binding gets a stable opaque ID; the server also emits a sibling JSON manifest mapping IDs → binding metadata; the client fetches the manifest, walks the DOM by ID, and resolves bindings from the manifest.
3. **Inline comment annotations.** `<!-- s:on:click=incr -->` adjacent to the bound element. The client walks comment nodes.

The framework needs to commit before the IR render walker (Phase D) can land — the walker emits markers, and the attach runtime reads them, so they share a contract. Without a decision the walker has no target.

## Decision

**Markers are `data-set-*` attributes on the rendered element.** The server-side render walker emits attributes; the client attach runtime walks `[data-set-*]` selectors to find bindings.

### Attribute family

| Attribute | Meaning | Example |
|---|---|---|
| `data-set-on:<event>="<key>"` | Event handler binding | `data-set-on:click="incr"` |
| `data-set-bind:<prop>="<key>"` | Two-way binding | `data-set-bind:value="query"` |
| `data-set-text="<key>"` | Text-interpolation binding (host is the text node's parent; child is a stable comment marker) | `data-set-text="count"` |
| `data-set-attr:<name>="<key>"` | Attribute-interpolation binding | `data-set-attr:href="profileUrl"` |
| `data-set-class:<name>="<key>"` | Conditional class | `data-set-class:active="isActive"` |
| `data-set-style:<name>="<key>"` | Conditional inline style property | `data-set-style:color="primaryColor"` |
| `data-set-attr-dynamic="<key>"` | Attribute whose **name** is reactive (e.g. `<a {hrefAttr}={url}>`) | `data-set-attr-dynamic="b7"` |
| `data-set-if="<key>"` | `{#if}` boundary; element is the rendered branch root | `data-set-if="branch_3"` |
| `data-set-for="<key>"` | `{#for}` boundary; child elements have `data-set-key="<value>"` | `data-set-for="loop_1"` |
| `data-set-match="<key>"` | `{#match}` boundary | `data-set-match="branch_7"` |
| `data-set-await="<key>"` | `{#await}` boundary | `data-set-await="async_2"` |
| `data-set-data="<name>"` | `<data>` subscription host (live or once) | `data-set-data="agents"` |
| `data-set-state="<name>"` | `<state>` declaration site (initial value carried by `data-set-init`) | `data-set-state="count"` |
| `data-set-use="<name>"` | `<use-*>` hook attach point | `data-set-use="kbd"` |

The `<key>` in each attribute is a stable identifier the IR emits; both server and client resolve it through the same emitted ESM module (the IR's compiled JS), so there is no manifest to fetch.

### Initial scope is a JSON island

Initial signal values for `<state>` and `<data>` are emitted into a single inline `<script type="application/json" id="__set_init__">{...}</script>` element appended at the end of the rendered view. Avoids one `data-set-init` attribute per signal (which would balloon the rendered HTML for views with many state keys) while keeping the format inspectable.

### Comment markers for non-element bindings

Text interpolations and control-flow branches that don't have a host element use stable comment markers:

```html
<!--set:text:count--><!--/set-->
<!--set:if:branch_3-->...<!--/set-->
```

These follow the same `<key>` resolution rule as attributes.

## Rationale

`data-set-*` won on three axes:

1. **Inspectable.** Open devtools, see the markers. No round-trip through a manifest, no decoding step. Authors and agents debugging an attach failure see exactly what the runtime sees.
2. **No sidecar to keep in sync.** Opaque IDs require a manifest. The manifest is another asset the daemon serves, another schema to version, another file to invalidate when the IR changes. `data-set-*` is self-describing.
3. **Standard platform behavior.** `data-*` is the documented author-extension point for HTML. Custom elements, ARIA, and frameworks like Datastar already use it. The browser's parser, devtools, accessibility tree, and CSS engine all treat `data-*` as opaque metadata — no semantic conflict.

The cost is a few extra bytes per binding versus opaque IDs (`data-set-on:click="incr"` vs `id="b3"`), and the marker namespace is visible in `outerHTML`. Both are acceptable for a daemon-served local-first app where total page weight is dominated by user content, not framework markers.

## Alternatives considered

### Opaque IDs + sidecar manifest

Rejected. The byte savings (~10–20% of marker overhead) are real but small in absolute terms; the architectural cost is high — every binding now requires the manifest to be present and parsed before reactivity can attach. First paint with reactive intent becomes "render → fetch manifest → parse manifest → attach," adding a serial dependency the simpler shape avoids. The manifest also has to be cache-busted in lockstep with the rendered HTML; one drift between them and the page is silently broken.

### Inline comment annotations

Rejected. Comment markers work for text-only bindings (and we use them for that case below), but applying them universally — including for elements with `on:click` and `bind:value` — means the runtime has to scan comment nodes and walk siblings to find the host element. That's both slower than `querySelectorAll('[data-set-on\\:click]')` and more fragile under DOM mutation by third-party code (browser extensions, etc., which routinely strip comments).

### Datastar's `data-*` verbatim

Rejected for naming. Datastar uses `data-on-click`, `data-text`, `data-show`. We are not Datastar — different reactivity primitive, different control flow, different `<data>` semantics. Borrowing the literal attribute names would invite the assumption that the surfaces are interchangeable. The `data-set-*` prefix establishes our own namespace while keeping the platform-standard `data-*` shape.

## Consequences

### Positive

- **Walker and attach runtime share a stable, declarative contract.** Both sides serialize/deserialize the same attribute family through the IR's emitted ESM. No marshaling layer.
- **Devtools-friendly debugging.** Inspecting an element shows which bindings are live on it.
- **Progressive enhancement preserved.** A page rendered without the attach runtime still parses cleanly — `data-set-*` attributes are inert to browsers, no broken styles, no broken accessibility.
- **CSP-compatible.** No inline scripts beyond the JSON island (which uses `type="application/json"`, not executable). Works under strict CSP.

### Negative

- **Marker bytes in HTML.** Each reactive binding adds 20–60 bytes of attributes. A view with 100 bindings adds 2–6 KB to the rendered output. Acceptable for daemon-served first-party views; would be worth revisiting if Ship Set ever targeted public-web byte budgets.
- **Attribute namespace is reserved.** Authors may not use `data-set-*` for their own purposes. The `set` prefix is the framework's. Documented in the framework skill.
- **Refactoring marker shape later is breaking.** Switching the attribute names or the JSON-island schema once shipped requires coordinated daemon + client redeploy. The contract is part of the public surface from the moment it ships.

### Open questions

- **Marker stability across recompiles.** When the same `.html.set` is recompiled, do the `<key>` identifiers stay stable across builds? They do not need to, since the server and client are always served from the same IR module — but if hot-reload starts streaming partial DOM patches, key stability becomes a constraint. Defer the call until hot-reload exists.
- **Server-side conditional bindings inside dead branches.** When a `{#if}` server-evaluates to false, the unrendered branch's bindings are skipped — but the IR still has them. Does the runtime ever need to attach a binding for a branch that wasn't initially rendered? Today: no, branches mount from scratch when the `if` flips. If we add stream-rendered partial DOM in the future, this question reopens.

## Amendments

### 2026-04-30 — `data-set-style:` and `data-set-attr-dynamic` added

The Phase D-1 walker landed with two attribute kinds not in the original table: `data-set-style:<name>="<key>"` for `style:` directives (the inline-style analog of `class:`) and `data-set-attr-dynamic="<key>"` for attributes whose name is itself reactive (`<a {hrefAttr}={url}>`). Both follow the same family naming convention and the same `<key>` resolution rule as the original table — they are extensions, not divergences. Folded into the canonical attribute family above so the contract stays a single source of truth.

## References

- [SSR and the attach runtime](../../../ship-set-framework/references/docs/ssr-hydration.md) — the doc this ADR concretizes.
- [Implementation status](../../../ship-set-framework/references/docs/status.md) — Phase D-1/D-2/D-3 (IR render walker), Phase E (attach-only client runtime) consume this contract.
- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — the framework spine that constrains the marker namespace.
- Datastar's `data-*` attribute family — referenced for shape, not for names. Their docs are at <https://data-star.dev>.
