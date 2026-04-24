---
title: "Skill Variables"
sidebar:
  label: "Skill Variables"
  order: 7
---
**Location**: `.ship/skills/<skill-id>/assets/vars.json`
**Schema**: `vars.schema.json`

Declares typed, scoped configuration variables that make a skill "smart." The compiler resolves variables into SKILL.md at compile time via MiniJinja templates.

## Variable fields

Each top-level key is a variable name. Values are objects:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `type` | enum | `"string"` | `string`, `bool`, `enum`, `array`, `object` |
| `default` | any | — | Default value when no user state is set. Must match `type`. |
| `storage-hint` | enum | `"global"` | Where values are stored: `global`, `local`, `project` |
| `values` | string[] | — | Allowed values for `enum` type. Validated on set and compile. |
| `label` | string | — | Human-readable name shown in Studio and CLI |
| `description` | string | — | Longer explanation shown in Studio tooltips |

## Storage scopes

| Scope | Semantics | Shared? |
|-------|-----------|---------|
| `global` | Machine-wide, follows user across projects | Personal |
| `local` | This project only | Personal |
| `project` | This project, intended for teams | Team-shared |

Merge order: defaults → global → local → project. Last wins.

## Variable types

| Type | Default value example | Notes |
|------|----------------------|-------|
| `string` | `"vitest"` | Set via Studio Skills IDE or `set_skill_var` MCP tool |
| `bool` | `true` | Set via Studio Skills IDE or `set_skill_var` MCP tool |
| `enum` | `"fast"` | Values constrained to `values` list |
| `array` | `["src/"]` | Stored as JSON array string |
| `object` | `{}` | Stored as JSON object string |

## Example

```json
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/vars.schema.json",
  "test_runner": {
    "type": "enum",
    "values": ["vitest", "jest", "pytest", "cargo-test"],
    "default": "vitest",
    "storage-hint": "project",
    "label": "Test Runner",
    "description": "Which test runner to use for TDD cycles."
  },
  "commit_at_green": {
    "type": "bool",
    "default": false,
    "storage-hint": "local",
    "label": "Auto-commit on green",
    "description": "Automatically commit when tests pass."
  },
  "watch_paths": {
    "type": "array",
    "default": ["src/", "tests/"],
    "storage-hint": "project",
    "label": "Watch Paths",
    "description": "Directories to watch for file changes."
  }
}
```
