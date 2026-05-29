---
title: Permissions
description: Four presets decide what an agent can do without asking — from read-only to elevated — and you can layer rules on top.
section: Agents
order: 2
kicker: Agents
tags: [agents, permissions, security]
related: [agents]
---

Permissions decide which tools your agent can use without pausing to ask. Ship ships four presets that cover most cases — pick one, layer extra rules if you need to.

## The four presets

- **`ship-readonly`** — read, search, and read-only Ship tools; can't modify files or run destructive commands. For reviewers, gate agents, audits.
- **`ship-standard`** — the default for hands-on work: reads and writes, but the harness asks before risky operations. For interactive sessions paired with a human.
- **`ship-autonomous`** — for agents working alone (typically in a separate worktree): file ops and most shell run without confirmation; pushing, publishing, and destructive resets stay blocked. For dispatched specialists.
- **`ship-elevated`** — like autonomous, but unlocks `git push` and publish; only force-push and hard resets remain blocked. For CI and release automation.

## Picking one

```jsonc
{ "permissions": { "preset": "ship-autonomous" } }
```

That's the whole declaration — Ship handles the underlying allow-lists, deny-lists, and confirmation rules.

## Layering rules

Add to a preset's defaults from the profile — allow what it doesn't, deny extra, or require confirmation:

```jsonc
{
  "permissions": {
    "preset": "ship-autonomous",
    "tools_allow": ["Bash(docker *)"],
    "tools_deny":  ["Bash(rm -rf *)"],
    "tools_ask":   ["Bash(git push --force*)"]
  }
}
```

When a tool matches both allow and deny, **deny wins**.
