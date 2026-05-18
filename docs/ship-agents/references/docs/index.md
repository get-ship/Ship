---
group: Agents
title: Agents
order: 1
---

An **agent** is a profile that tells an AI assistant how to behave in your project — which skills it knows, which tools it can reach for, what it's allowed to do. You author the profile once in `.ship/agents/<name>.jsonc`; Ship compiles it into the configuration each harness expects.

Agent profiles are harness-agnostic at the baseline. The project's `harnesses` list (in `ship.jsonc`) decides which harnesses get compiled; every agent profile compiles for every harness in that list. To override a single setting (e.g. `model`) for one harness, use `harness_settings.<harness>.<key>` on the profile.

## What goes into an agent

<sui-card padding="md">
<strong slot="header">Identity</strong>
A name and a description. One profile compiles for every harness the project enables.
</sui-card>

<sui-card padding="md">
<strong slot="header">Skills</strong>
Instruction sets that teach the agent how to do specific things. Pick from your local skills, from installed packages, or both.
</sui-card>

<sui-card padding="md">
<strong slot="header">MCP servers</strong>
External tools the agent connects to via Model Context Protocol — your own services, GitHub, anything MCP-aware.
</sui-card>

<sui-card padding="md">
<strong slot="header">Permissions</strong>
A preset that controls what the agent can do without asking — read-only, interactive, autonomous, or elevated. See <a href="./permissions/">Permissions</a>.
</sui-card>

## A small agent

```jsonc
{
  "agent": {
    "id": "dev",
    "name": "Dev"
  },
  "skills": {
    "refs": ["tdd", "code-review"]
  },
  "permissions": {
    "preset": "ship-standard"
  }
}
```

That's the minimum. Identity, skills, permissions. Save the file, run `ship use dev`, and Ship emits one configuration file per harness in the project's `harnesses` list.

## Where agents live

- **Project agents** — `.ship/agents/` in your repo. Committed; shared with your team.
- **Library agents** — `~/.ship/agents/` on your machine. Personal; not committed.

## Compiling for several harnesses at once

Configure the project's `harnesses` list in `ship.jsonc` — Ship compiles every agent profile for every harness in that list on `ship use`:

```jsonc
// ship.jsonc
{
  "harnesses": ["claude", "gemini", "codex"]
}
```

Each harness gets its native format. The compiled files are gitignored build output — your `.ship/` profile is the source of truth.

## Per-harness overrides

When one harness needs a different setting (e.g. a specific `model`), declare it under `harness_settings`:

```jsonc
{
  "agent": { "id": "dev", "name": "Dev" },
  "harness_settings": {
    "gemini": { "model": "gemini-2.5-pro" }
  }
}
```

This applies only to the matching harness file; other harnesses use the default.

## Next

- [Permissions](./permissions/) — the four built-in presets and how to layer your own rules.
