---
title: "Lint Rule Catalog"
description: "Every rule the view linter ships, with severity, category, description, and an example violation + fix. Rule IDs are a stable contract — see the stability policy."
sidebar:
  label: "Lint Rule Catalog"
  order: 4
---
`ship lint views` and the `lint_view` MCP tool run the rules below.
Each rule has a stable ID of the form `<category>-<descriptor>`. Rule
IDs are a public contract — see
[Lint rule stability policy](../contributing/lint-rule-stability.md)
for the rules around renames, additions, and severity changes.

The catalog is hand-curated to match `lint::ALL_RULES` in
`crates/core/lint/src/lib.rs`. When you add or remove a rule there,
update this page in the same commit.

## Categories

| Prefix    | Scope                                                     |
|-----------|-----------------------------------------------------------|
| `style-`  | Inline `<script>` and `<style>` discipline.               |
| `views-`  | Datastar signal hygiene (regex pass).                     |
| `pkg-`    | View-manifest pack declarations.                          |
| `ast-`    | minijinja-AST checks (macros, imports, filters, routes).  |

## Rules

| ID                            | Severity   | Category | What it checks                                                                                |
|-------------------------------|------------|----------|------------------------------------------------------------------------------------------------|
| `style-no-inline-script`      | error      | style    | Bare `<script>` blocks inside view templates.                                                  |
| `style-inline-budget`         | warn/error | style    | Total inline `<style>` line count across a view (`>50` warn, `>150` error).                    |
| `views-undefined-signal`      | warn       | views    | `$name` references on attributes where `name` is never declared via `signal()` / `data-signals*`. |
| `pkg-missing-pack-declaration`| error      | pkg      | Heavy-island tag (`<ship-code-editor>`, `<ship-terminal>`, ...) used without `packs:` in manifest. |
| `pkg-stale-pack-declaration`  | warn       | pkg      | A `packs: [name]` entry whose tags never appear in the view's templates.                       |
| `ast-unknown-macro`           | error      | ast      | `{% from '_ship.html.j2' import x %}` where `x` is not in `sdk-macros.json`.                   |
| `ast-unused-import`           | warn       | ast      | Imported SDK macro that is never called.                                                       |
| `ast-unknown-filter`          | warn       | ast      | `{{ x | filt }}` where `filt` is not a built-in or shipd-registered filter.                    |
| `ast-undefined-context`       | warn       | ast      | `{{ ident }}` referencing a name absent from the view's declared context schema.               |
| `ast-undeclared-signal`       | warn       | ast      | `$name` reference (AST-grade) without a matching `signal('name', ...)` call.                   |
| `ast-unknown-route`           | warn       | ast      | `@get('/api/...')` / `@post(...)` etc. whose method+path is not registered in `sdk-urls.json`. |
| `ast-parse-error`             | warn       | ast      | The minijinja parser rejected the template (informational, no AST checks ran).                 |
| `ast-unknown-ship-op`         | error      | ast      | `ship.<domain>.<op>(...)` where `<domain>.<op>` is not a registered op at `/api/ship/schema`.  |

## Examples

### `style-no-inline-script`

Violation:

```html
<div>
  <script>console.log('hi');</script>
</div>
```

Fix — move the logic into a `data-effect` expression or extend the
component:

```html
<div data-effect="console.log('hi')"></div>
```

### `style-inline-budget`

Violation — a single view accumulates more than 150 lines of inline
`<style>`. Fix: extract repeated CSS into a custom element or a
shared macro. Layout-only rules can stay inline; theme rules
shouldn't appear at all (see `docs/ship-themes`).

### `views-undefined-signal`

Violation:

```html
<div data-text="$count"></div>
```

Fix — declare the signal before referencing it:

```html
<div {{ signal('count', 0) }} data-text="$count"></div>
```

Or allow the name explicitly:

```jinja
{#- @lint:signals-ok: count -#}
```

### `pkg-missing-pack-declaration`

Violation — `<ship-code-editor>` is used but no `packs:` declared.
Fix:

```yaml
# .ship/views/<name>/manifest.yaml
packs:
  - editor
```

### `pkg-stale-pack-declaration`

Violation — manifest declares `packs: [editor]` but no
`<ship-code-editor>` / `<ship-markdown-editor>` tag appears in any
template. Drop the declaration.

### `ast-unknown-macro`

Violation:

```jinja
{% from '_ship.html.j2' import bogus_macro %}
```

Fix — only import macros that exist in `sdk-macros.json`. Suggested
near-matches appear in the lint message.

### `ast-unused-import`

Violation:

```jinja
{% from '_ship.html.j2' import signal %}
<div>no signals used</div>
```

Fix — drop the unused name from the import list.

### `ast-unknown-filter`

Violation: `{{ title | urlencod }}`. Fix: `{{ title | urlencode }}`
(linter suggests close matches via Levenshtein).

### `ast-undefined-context`

Violation — view context schema (per `sdk-view-contexts.json`)
declares `repo, files`, but the template references `branhc`. Fix —
correct the typo or extend the view-context schema.

### `ast-undeclared-signal`

AST-grade variant of `views-undefined-signal`. Same remediation.

### `ast-unknown-route`

Violation: `data-on:click="@get('/api/nope/nope')"`. Fix — call
`/api/sdk/query` to find a registered route, or add the route to
`sdk-urls.json`.

### `ast-parse-error`

Informational — minijinja could not parse the template; the AST
rules are skipped for this file. Fix the syntax error to re-enable
checks.

### `ast-unknown-ship-op`

Violation: `ship.agents.detial(id="x")`. Fix:
`ship.agents.detail(id="x")` (registry lives at
`/api/ship/schema`).

## See also

- [`ship-sdk` Rules](../sdk/rules.md) — high-level authoring rules,
  the *why* behind each lint.
- [Lint rule stability policy](../contributing/lint-rule-stability.md)
  — what we promise about rule IDs after launch.
