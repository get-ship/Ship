# Archived skills

These skills are kept for reference and maintenance work on legacy code paths
that still ship in the project. They are **not loaded into agent context by
default** and **not part of the active authoring contract**.

Anything you write today should follow the canonical skills in
`.ship/skills/`, not these.

| Archived skill        | Superseded by      | Why archived                                                                                                                                              |
| --------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ship-components-lit` | `ship-components`  | The active component library is `<sui-*>` web components compiled from `.sui` templates (`packages/ui-components/`). Lit `<ship-*>` components are maintenance-only. |
| `ship-views-j2`       | `ship-views`       | The active view stack is `.sui` templates with paired `.controller.ts` (`views/<name>/page.sui`). Jinja `.html.j2` views (`.ship/views/`) are maintenance-only. |

## When to read an archived skill

- You are explicitly editing a Lit `<ship-*>` component under
  `packages/ship-set/src/web-components/`.
- You are explicitly editing a `.html.j2` view under `.ship/views/` or
  a daemon-rendered partial under `apps/shipd/templates/`.
- You are migrating one of the above to its successor and need to read
  the legacy contract first.

For any new component, primitive, or view: ignore this directory.
