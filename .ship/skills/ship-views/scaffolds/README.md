# View scaffolds

Canonical starter shapes for `ship-views`. Shipwright copies a
scaffold into `.ship/views/<name>/` then substitutes placeholders.

## Placeholders

- `{{TITLE}}` — human-readable view title (Title Case)
- `{{NAME}}` — kebab-case view id (should match the directory name)

## Shapes

| Shape       | Use when                                                                       |
| ----------- | ------------------------------------------------------------------------------ |
| `list`      | A collection of similar items, optionally filterable — default.                |
| `form`      | A single entity created or edited via bound inputs.                             |
| `detail`    | A single entity with multiple sections (meta, wire, permissions).               |
| `dashboard` | A top-level summary surface composed of tiles pulling independent data.         |
| `tabs`      | Multiple related views of the same entity, switched via ship-tabs.              |
| `blank`     | Fallback when none of the above fits; starts with an empty-state placeholder.   |

## Completion criteria

A scaffold is "done" when it:

1. Parses cleanly (no Jinja errors) and renders a non-empty body.
2. Uses `ship.*` ops for data (not raw `fetch`).
3. Declares `rail` metadata + `$schema` reference in its manifest.
4. Passes `lint::lint_view_dir` with zero errors.

If any of these fail after copy + substitution, ask the user a
clarifying question before proceeding — don't commit broken scaffolds.
