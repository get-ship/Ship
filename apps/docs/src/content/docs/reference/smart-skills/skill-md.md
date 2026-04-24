---
title: "SKILL.md Frontmatter"
description: "Canonical YAML frontmatter contract for every Ship skill. Required fields, optional fields, and the JSON Schema that enforces them at load time."
sidebar:
  label: "SKILL.md Frontmatter"
  order: 2
---
Every Ship skill is a directory containing a `SKILL.md`. The first block of that file is YAML frontmatter that declares the skill's identity. The Ship runtime validates this frontmatter against a single canonical JSON Schema every time a skill is loaded. Skills that fail validation are rejected with an error that names the directory and the offending field.

The canonical schema lives at [`schemas/skill.schema.json`](https://github.com/getship-dev/ship/blob/main/schemas/skill.schema.json). The runtime embeds this schema at compile time so validation is deterministic and offline.

## Required fields

| Field | Type | Constraint | Notes |
|-|-|-|-|
| `name` | string | kebab-case, 1-64 chars, must equal the directory name | `docs/ship-triggers/SKILL.md` requires `name: ship-triggers` |
| `stable-id` | string | kebab-case, 1-64 chars | Stable storage key for variable state. Survives directory renames. By convention equals `name`. |
| `description` | string | 1-1024 chars | One sentence. Front-load with "Use when..." for reliable trigger matching. |
| `tags` | array of kebab strings | non-empty | Discovery labels. |
| `authors` | array of strings | non-empty | Free-form. |

## Optional fields

| Field | Type | Notes |
|-|-|-|
| `audience` | enum: `public` \| `internal` \| `agent-only` | Defaults to public. `internal` is excluded from the docs collector; `agent-only` ships only to agents. |
| `version` | string | Semver-ish. Absent means rolling. |
| `supersedes` | kebab string | Stable-id of a replaced skill. |
| `superseded-by` | kebab string | Stable-id of the replacing skill. |
| `license` | string | SPDX identifier, e.g. `MIT`, `Apache-2.0`. |
| `compatibility` | string (max 500 chars) | Comma-separated provider names. Omit for universal compatibility. |
| `allowed-tools` | array of strings | MCP tool names the skill requires. |
| `artifacts` | array of enum (`html`, `pdf`, `markdown`, `image`, `adr`, `note`, `json`, `url`, `svg`) | Artifact types this skill produces. The runtime infers event subscriptions from these. |
| `metadata` | object<string,string> | Reserved keys: `display_name`, `source`. |

## Reserved fields

`packs` is **reserved**. The schema rejects any SKILL.md that includes a `packs` field. The pack distribution model has not been finalized; reserving the key keeps the option open without locking in a half-baked shape. If you have a pack use case, file an ADR.

`author` (singular) is **not allowed**. Use `authors` (plural array).

Unknown fields are rejected. The schema is `additionalProperties: false`.

## Example

```yaml
---
name: ship-triggers
stable-id: ship-triggers
description: Use when configuring Ship triggers — JSONC files that match events and run actions.
tags: [reference, triggers, events, automation]
authors: [ship]
audience: public
artifacts: [adr, markdown]
---

# Ship Triggers

(skill body in markdown)
```

## How validation runs

The runtime loads `SKILL.md` via `crates/core/runtime/src/agents/skill/storage.rs::parse_skill`. Each load:

1. Reads the file and splits the YAML frontmatter from the markdown body.
2. Parses the YAML into a JSON value.
3. Validates the JSON value against the embedded canonical schema.
4. On validation failure, returns an error of the form `Invalid SKILL.md frontmatter in <path>:` followed by a list of `/<field>` paths and constraint failures.
5. On success, deserializes into the loader's typed frontmatter struct and applies cross-field rules (e.g. `name` must match the directory id).

This is the contract that governs every committed `docs/ship-*/SKILL.md` and any user- or agent-authored skill installed via `skills.sh`.
