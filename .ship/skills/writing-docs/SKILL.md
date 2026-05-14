---
name: writing-docs
stable-id: writing-docs
description: Use when creating or updating reference documentation for a doc skill. Covers plain-markdown format with ship-elements, frontmatter conventions, audience filtering, and multi-page structure.
tags: [skills, documentation, authoring]
authors: [ship]
---

# Writing Docs

Write documentation in `references/docs/` inside a doc-skill directory under `docs/`. Every doc page serves two audiences: humans browsing the docs site and agents retrieving context on demand.

## Two kinds of skills

Two directory trees, two audiences, different rules.

| Tree | Purpose | This skill applies? |
|---|---|---|
| `docs/ship-*/` | **Doc skills** — reference material for humans and agents researching Ship. | Yes — this is where `references/docs/*.md` pages go. |
| `.ship/skills/<id>/` | **Impl skills** — bundled with the Ship runtime. Operational, loaded into agent context when authoring that kind of code. | No — impl skills are single `SKILL.md` files; the rules below don't apply. |

Agents authoring Ship code read impl skills. Humans and agents researching Ship read doc skills. When you say "writing docs," you mean doc skills.

## Where pages live

```
docs/ship-<id>/
  SKILL.md                    <- agent-facing TOC, concise
  references/docs/
    index.md                  <- landing page
    <topic>.md                <- one concern per page
```

`SKILL.md` is the pointer. Keep it under ~50 lines. Reference pages are the library — retrieved when an agent needs depth.

Only files with a `group:` frontmatter field are picked up by the site generator (`apps/docs/scripts/collect-skill-docs.ts`). Files without `group:` are agent-only by convention.

## Format — plain Markdown + ship-elements

Pages are plain `.md` files. No Markdoc. HTML passes through, which means `<ship-card>`, `<ship-alert>`, and any other web component works inline:

```markdown
## Example

<ship-alert tone="info">
Inline web components render via the ship-elements bundle loaded on every page.
</ship-alert>
```

Legacy `.mdoc` pages exist in the repo and still work, but new pages should be `.md`.

## Frontmatter

Every doc page that should render on the site starts with YAML frontmatter:

```yaml
---
group: Architecture
order: 2
title: Kernel
description: Event routing contract, namespace scoping, persist-before-deliver.
audience: internal
---
```

| Field | Required | Purpose |
|---|---|---|
| `group` | yes — omit to mark agent-only | Sidebar group label. Maps to a route segment (`Architecture` → `/reference/architecture/`). |
| `title` | yes | Page title. Starlight renders its own `<h1>` from this — don't add one in the body. |
| `description` | recommended | Meta description. |
| `order` | no | Sort position within the group. Lower = earlier. |
| `audience` | no | `public` (default) \| `internal` \| `agent-only`. Filtering below. |

## Audience tiers

| Value | On public site (`bun run build`) | On internal dev site (`bun run dev`) | Agent-only context |
|---|---|---|---|
| `public` (default) | visible | visible | visible |
| `internal` | hidden | visible | visible |
| `agent-only` | hidden | hidden | visible |

The `AUDIENCE=internal` env var on `predev` controls the dev-site mode. Block-level filtering inside a page uses `{% internal %}...{% endinternal %}` markers.

## Block-level filtering

Two block markers inside a page:

```markdown
{% internal %}
This section only shows on the internal dev site. Stripped from the public build.
{% endinternal %}

{% user-only %}
This section is for users on the public site. Agent-side stripping planned (not yet wired).
{% enduser-only %}
```

## Writing for two audiences

Content works for both a human scanning and an agent retrieving.

- Tables for reference data (fields, attributes, options).
- Code blocks for examples — agents parse these directly.
- Headings as retrieval anchors — agents grep by heading.
- One concern per page. If a page covers three things, it's three pages.
- Index page summarizes the skill and links to siblings.

## Collect and verify

Regenerate the site content:

```bash
cd apps/docs && bun run collect           # user mode
AUDIENCE=internal bash -c 'cd apps/docs && bun run collect'   # internal mode
```

Dev server watches source files and re-collects automatically:

```bash
cd apps/docs && bun run dev               # binds --host, AUDIENCE=internal
```

Open `http://localhost:4321`. Any edit in `docs/ship-*/references/docs/*.md` triggers a re-collect within a couple hundred ms.

## Internal linking

Use site routes, not file paths:

```markdown
See [Kernel](/reference/architecture/kernel/) for the routing contract.
```

Not `../../../ship-kernel/references/docs/kernel.md`. Site routes are stable; file paths break on every group/filename change.

## Bundled vs private

Impl skills in `.ship/skills/` ship with the runtime and must be self-contained — they can't reference private docs. If you need reference content inside an impl skill, inline it there. Duplication with the doc skill is acceptable; bundled content is the constraint.
