---
title: "Skill Frontmatter"
sidebar:
  label: "Skill Frontmatter"
  order: 6
---
**Location**: `.ship/skills/<skill-id>/SKILL.md`
**Schema**: `skill.schema.json`

SKILL.md uses YAML frontmatter for metadata. The body is markdown — either plain or a MiniJinja template when `assets/vars.json` exists.

## Frontmatter fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Skill identifier. Lowercase alphanumeric with hyphens. Pattern: `^[a-z0-9][a-z0-9-]*[a-z0-9]$` Max: 64 chars. |
| `description` | string | yes | When to use this skill. Start with "Use when..." for reliable trigger matching. Max: 1024 chars. |
| `stable-id` | string | no | Stable storage key for variable state. Survives directory renames. Defaults to directory name. |
| `tags` | string[] | no | Category labels for discovery on getship.dev |
| `authors` | string[] | no | Skill authors |
| `license` | string | no | SPDX identifier, e.g. `MIT` |
| `compatibility` | string | no | Comma-separated provider names. Omit for universal. |
| `allowed-tools` | string[] | no | MCP tool names the skill requires |
| `artifacts` | enum[] | no | Artifact types produced: `html`, `pdf`, `markdown`, `image`, `adr`, `note`, `json`, `url`, `svg` |
| `metadata` | object | no | Arbitrary key-value string metadata |

## Skill directory structure

```
skills/<skill-id>/
  SKILL.md              # Skill content with frontmatter (required)
  assets/
    vars.json           # Typed variables (makes it a "smart skill")
  references/
    docs/               # Reference material for progressive disclosure
  scripts/              # Executable scripts
```

## Example

```yaml
---
name: code-review
stable-id: code-review
description: Use when reviewing code changes for quality, correctness, and style.
tags: [review, quality]
authors: [ship]
license: MIT
allowed-tools: [mcp__ship__log_progress]
artifacts: [markdown]
---

# Code Review

Review the diff for...
```
