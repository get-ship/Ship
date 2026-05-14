---
group: Contributing
order: 5
title: Lint Rule Stability Policy
description: Rule IDs are a public contract. This page documents what we promise about renames, additions, removals, and severity changes.
audience: internal
---

# Lint Rule Stability Policy

Lint rule IDs (`style-no-inline-script`, `views-undefined-signal`,
`ast-unknown-route`, ...) appear in three places that outlive any
single Ship release:

- User `// ship-lint-disable: <rule-id>` comments in `.ship/views/`.
- CI configurations that pin specific rules (`ship lint views --rule
  <rule-id>`).
- Third-party tooling that consumes the JSON output of
  `ship lint views --format json`.

After launch, every published rule ID is locked in. The catalog is
`.ship/skills/ship-views/references/lint-rules.md` (or wherever the catalog lives once it's published).

## Naming convention

Every rule ID matches `^[a-z]+-[a-z0-9-]+$`. The first segment is the
*category* prefix and groups related rules:

| Category | Scope                                                    |
|----------|----------------------------------------------------------|
| `style-` | Inline `<script>` and `<style>` discipline.              |
| `views-` | Datastar signal hygiene (regex pass).                    |
| `pkg-`   | View-manifest pack declarations.                         |
| `ast-`   | minijinja-AST checks.                                    |

New categories may be introduced for new rule families
(`runtime-...` for the home-layout grep guards is a likely future
addition). Once shipped, a category prefix is part of the contract
and cannot be renamed either.

## Promises

### Additive

New rules can be added in any release. They land disabled-by-default
(severity `warn`) unless they enforce a hard correctness invariant,
in which case they ship as `error` with a one-release announcement
in `CHANGELOG.md`.

### Renames are breaking

A rule ID rename is a breaking change. We do not perform renames
post-launch. If a rule's intent changes meaningfully, we deprecate
the old ID (see below) and add a new one.

### Removal requires deprecation

Removing a rule requires marking it deprecated for one release. A
deprecated rule still emits its findings but with a leading
`(deprecated)` tag in the human formatter and a `"deprecated": true`
field in JSON output. The next release may remove it.

### Severity changes are signalled

Tightening (`warn` → `error`) or loosening (`error` → `warn`) a
rule's severity is announced in `CHANGELOG.md` under a `Lint
severity changes` heading. CI configs that pin
`--rule <id>` are unaffected; configs that filter by severity should
review the changelog before bumping Ship.

## Author workflow

When you add a rule:

1. Pick the category. Reuse an existing prefix if it fits.
2. Choose a kebab-case descriptor (`<category>-<descriptor>`).
3. Register the constant in `crates/core/lint/src/lib.rs` (or the
   corresponding submodule for AST rules) and append it to
   `ALL_RULES`.
4. Update `.ship/skills/ship-views/references/lint-rules.md` (or wherever the catalog lives once it's published)
   in the same commit — the catalog is the user-facing contract.
5. Add at least one happy-path test and one failure-path test under
   `crates/core/lint/src/tests.rs`.

When you remove a rule:

1. Tag the constant with `#[deprecated]` and update the catalog with
   a deprecation note.
2. Land the actual removal one release later.

## Why this matters

The reverse case — rule IDs that drift each release — silently
breaks user `ship-lint-disable` comments. A comment that no longer
matches a rule ID still parses, but its disable no longer takes
effect, and a previously-suppressed warning becomes an error. Stable
IDs make those failures impossible.
