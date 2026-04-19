---
group: Schema Reference
title: Permissions
section: reference
order: 4
---

# Permission Presets — permissions.jsonc

**Location**: `.ship/permissions.jsonc`
**Schema**: `permissions.schema.json`

Named permission presets define tool access rules. Agents reference presets by name via `permissions.preset`, then overlay additional allow/deny/ask patterns.

## Built-in presets

| Preset | Mode | Use case |
|--------|------|----------|
| `ship-readonly` | `plan` | Reviewers, auditors. Denies all writes. |
| `ship-standard` | `default` | Interactive sessions, human-paired work. |
| `ship-autonomous` | `dontAsk` | Dispatched specialists in worktrees. |
| `ship-elevated` | `dontAsk` | Deploy/release. Unlocks `git push` and `publish`. |

## Preset fields

| Field | Type | Description |
|-------|------|-------------|
| `default_mode` | enum | `plan`, `default`, `acceptEdits`, `dontAsk`, `bypassPermissions` |
| `tools_allow` | ToolPattern[] | Tools allowed without confirmation |
| `tools_deny` | ToolPattern[] | Tools denied. Takes precedence over allow. |
| `tools_ask` | ToolPattern[] | Tools requiring user confirmation. Applied before allow, after deny. |
| `additional_directories` | string[] | Directories the agent can access outside the project root |

## Permission modes

| Mode | Behavior |
|------|----------|
| `plan` | Read-only. No writes allowed. |
| `default` | Ask for writes. |
| `acceptEdits` | Auto-approve file edits, ask for others. |
| `dontAsk` | Fully autonomous. |
| `bypassPermissions` | No restrictions. |

## ToolPattern syntax

| Pattern | Example | Matches |
|---------|---------|---------|
| Bare name | `Read` | The Read tool |
| Parameterized | `Bash(git *)` | Bash calls starting with `git` |
| MCP tool | `mcp__ship__create_job` | Specific MCP tool |
| Glob | `mcp__*__delete*` | Any MCP delete tool |

## Example

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/permissions.schema.json",
  "ship-standard": {
    "default_mode": "default",
    "tools_allow": [
      "Read", "Glob", "Grep",
      "Bash(git status)", "Bash(git diff*)", "Bash(git log*)",
      "mcp__ship__*"
    ],
    "tools_deny": [
      "Bash(git push --force*)",
      "Bash(rm -rf *)"
    ]
  },
  "ship-autonomous": {
    "default_mode": "dontAsk",
    "tools_allow": [
      "Read", "Write", "Edit", "Glob", "Grep", "Bash",
      "mcp__ship__*"
    ],
    "tools_deny": [
      "Bash(git push --force*)"
    ]
  }
}
```
