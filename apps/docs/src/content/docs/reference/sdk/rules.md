---
title: "Rules"
description: "Hard rules for Ship view authoring. These are enforced by lint — the rest of the SDK assumes them."
sidebar:
  label: "Rules"
  order: 5
---
These are not style preferences. Each one is enforced by `ship lint
views` (or the `lint_view` MCP tool) and each one is what lets Ship
stay themeable, accessible, and fragment-reloadable with no build
step.

## No inline `<script>`

A view template contains no `<script>` tag. If you find yourself
reaching for one, the destination is one of:

- A macro or `<ship-*>` element. Start with
  `GET /api/sdk/query?q=<concept>`.
- A reactive `data-effect="…"` expression for idempotent side effects
  (canvas redraws, syncing a third-party widget). See the impl skill
  for the exact rules.
- A custom web component in `@ship/set`. That's the
  `ship-components` skill's charter.

## No inline `<style>`

Same rule, same destinations. Typography, color, radius, shadow, and
motion all come from design tokens. If an element doesn't expose the
CSS custom property you need, the fix is to extend the element, not
to sidestep it with inline CSS.

Layout-only CSS inside a `<style>` block in a view is a narrow
exception the impl skill documents — but it is **layout**, never
theme. The reference `.ship/views/git/index.html.j2` shows the
pattern: flex/grid scaffolding for the view's shell, nothing visual.

## No raw HTML when a ship-element exists

Don't type a `<button>` when `<ship-button>` exists. Don't write a
`<div class="card">` when `<ship-card>` exists. The full catalog is:

```bash
curl -s http://localhost:3000/api/assets/custom-elements.json | jq .
```

For list rows, tiles, and cards — the most common need — the answer
is always `<ship-list-tile>` with its `leading` / `meta` / `trailing`
slots. Do not invent a new element for a one-off composition.

## No hardcoded colors, radii, or motion

Design tokens are `var(--…)` references. Every surface has one.
Categories: `--space-1..8`, `--radius-sm..xl`, `--text-xs..4xl`,
`--duration-fast/medium/slow`, `--ease-out/in/in-out`, the semantic
palette (`--foreground`, `--background`, `--primary`, `--destructive`,
…), and the per-tone ramps (`--tone-success-{fg,bg,…}`).

Hex values, `rgb(…)` calls, raw `px` spacing, and `ease-in-out`
strings in a view template are all lint errors. The `docs/ship-themes`
skill is the full tour.

## Declare packs explicitly

Heavy island bundles — code editors, terminals, pack-specific
components like `<ship-git-view>` — are gated behind `packs:` in the
view manifest. The linter fires `missing-pack-declaration` when a
template uses a pack-only tag without the declaration, and
`stale-pack-declaration` when a declared pack has no matching tag.

```yaml
# .ship/views/skills/manifest.yaml
packs:
  - editor   # provides <ship-code-editor>, <ship-markdown-editor>
```

Only declare what you use. Don't copy pack lists between views.

## Default to macros over raw Datastar

Raw `data-signals:foo="'bar'"`, `data-on:click="@post(…)"`, and
`data-indicator="_creating"` work, but they're an escape hatch — the
lint engine surfaces them as warnings. Default to `signal()`,
`create()`, `loading()`. The macros get quoting, payload shape, and
a11y plumbing right; hand-written versions drift.

If a macro is missing, file it:

```jinja
{#- SDK gap: macro — need a primitive for <pattern> -#}
```

The gap becomes a lint warning and a ticket for extending the SDK.

## The escape hatch

If none of the above fits your need, you're past the view layer. The
answer is to author a web component in `@ship/set`. That is a
different skill (`ship-components`) with its own rules. A view never
contains the escape-hatch code; it imports the element.

## See also

- `docs/ship-themes/` — token layers, theme catalog, customization.
- `ship-components` impl skill — when and how to extend the library.
- `ship-views` impl skill — the canonical lint-rule list and the
  per-pattern anti-pattern table.
