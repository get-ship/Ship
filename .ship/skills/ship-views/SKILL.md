---
name: ship-views
stable-id: ship-views
description: Author Ship views. Two stacks live here — SUI (.sui templates, AOT-compiled, attach()-on-SSR; new authoring) and j2/Datastar (.html.j2, current production, fix-only). Self-contained skill; deep references are siblings under references/.
tags: [views, sdk, extension-authoring, jinja, datastar, sui, ship-elements]
authors: [ship]
audience: agent
---

# ship-views

Two stacks coexist. Pick deliberately:

| Stack | Status | When to use |
|---|---|---|
| **SUI** (`.sui` + paired `.ts`) | New, primary going forward | All new views. Existing views port over one at a time. |
| **j2 + Datastar** (`.html.j2`) | Current production, fix-only | Fixes / incremental changes to existing `.ship/views/<name>/index.html.j2`. No new j2 views. |

Don't mix the two stacks within a single view. A view is one or the other.

## References

| When you're doing | Read |
|---|---|
| Authoring a new view in SUI | `references/sui-authoring.md` |
| Fixing or extending an existing j2 view | `references/j2-authoring.md` |
| Working on the SUI framework itself (compiler, runtime, daemon SUI handlers) | `references/sui-internals.md` |
| Authoring or composing components | the `ship-components` skill |

## File layout (both stacks)

Project-local extension-point views (daemon resolves on disk):

```
.ship/views/<name>/
  index.{sui|html.j2}        # template (the file extension picks the stack)
  manifest.yaml              # name, type: {sui|html}, rail, events, packs
  partials/                  # reusable fragments
  [param]/index.{sui|html.j2}  # parameterized sub-routes
```

First-party SUI views baked into `shipd` live at `views/<name>/{view.sui, view.ts, view.css, package.json}` (`counter`, `agents`, `git`).

Manifest's `type:` field discriminates: `type: sui` → daemon serves at `/sui/<name>`; `type: html` → `/api/v1/v/<name>`.

## Shell chrome

The shell owns topbar, breadcrumb, status bar. Views contribute via `<ship-view-chrome>` (slot/attribute reference is in `references/j2-authoring.md` — same element across both stacks). Don't reimplement chrome inside the view body.

## Hard rules (both stacks)

- Subscriptions live inside `<data>` (SUI) or in the manifest `data:` block (j2). Never call `ship.*` from imperative code in a template.
- No inline `<script>` or `<style>` inside templates. Use the framework's binding / signal / hook constructs.
- Components compose; views compose components. View-local repetition goes in `partials/`. Promote to a library component only after 3+ uses (and only via the **distribution test** — see `ship-components`).
