---
group: Agents
title: Permissions
order: 2
---

Permissions decide which tools your agent can use without pausing to ask. Ship ships four presets that cover most cases — pick one, layer extra rules if you need to.

## The four presets

<sui-card padding="md">
<strong slot="header">ship-readonly</strong>
The agent can read, search, and call read-only Ship tools. It can't modify files or run destructive commands. Right for reviewers, gate agents, and audits.
</sui-card>

<sui-card padding="md">
<strong slot="header">ship-standard</strong>
The default for hands-on work. The agent can read and write; the provider asks before risky operations. Right for interactive sessions paired with a human.
</sui-card>

<sui-card padding="md">
<strong slot="header">ship-autonomous</strong>
For agents working alone, typically in a separate worktree. File operations and most shell commands run without confirmation. Pushing, publishing, and destructive resets are still blocked. Right for dispatched specialists.
</sui-card>

<sui-card padding="md">
<strong slot="header">ship-elevated</strong>
Like autonomous but unlocks <code>git push</code> and publish. Only force-push and hard resets remain blocked. Right for CI and release automation.
</sui-card>

## Picking a preset

```jsonc
{
  "permissions": {
    "preset": "ship-autonomous"
  }
}
```

That's the whole declaration. Ship handles the underlying tool allow-lists, deny-lists, and confirmation rules.

## Layering rules

You can add to the preset's defaults from the agent profile. Allow what the preset doesn't, deny extra things, or require confirmation on specific commands:

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

When a tool matches both allow and deny, deny wins.

## Custom presets

If the four built-ins don't fit, write your own in `.ship/permissions.jsonc`:

```jsonc
{
  "my-ci-preset": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Glob", "Grep", "Bash(bun *)"],
    "tools_deny":  ["Write(*)", "Edit(*)"]
  }
}
```

Then reference it from your agent: `"preset": "my-ci-preset"`.

## What's always blocked

A few things are denied for every preset, even `ship-elevated`, and can't be turned off from a profile:

- Direct access to Ship's databases — state must flow through the runtime, not raw SQL.
- Reading or writing common secret files (`.env*`, `credentials*`, `secrets/`).
- Auto-memory features in the underlying provider — Ship is the memory layer, so we keep provider-native memory off.

Writes to `.ship/` itself always require confirmation, even in autonomous mode. The agent can't silently change its own configuration.

## Why scoping matters more than permissions

Presets are a useful default, not a safety boundary. Real safety
comes from scoping the actor: each agent runs in its own space with
its own `cwd`, its own connectors, and its own event log. Two
specialist agents working in parallel on the same project never
see each other's tools, files, or decision history unless they
explicitly share a space. Permission presets keep one agent honest
within its scope; per-actor scoping is what keeps mistakes from
leaking across agents.

A reviewer (in `ship-readonly` mode, or a human) gating the merge
is the last layer, not the first. The runtime's supervisor, the
actor's tool gate, and the per-space event partition are the layers
that hold before any human review happens.