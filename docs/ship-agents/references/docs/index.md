---
group: Agents
title: Agents
order: 1
---

An **agent** is a profile that tells an AI assistant how to behave in your project — which skills it knows, which tools it can reach for, what it's allowed to do, and which provider it runs as. You author the profile once in `.ship/agents/<name>.jsonc`; Ship compiles it into the configuration each provider expects.

## What goes into an agent

<sui-card padding="md">
<strong slot="header">Identity</strong>
A name, a description, and the providers this agent should compile to. One agent can target several providers from the same source.
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
    "name": "Dev",
    "providers": ["claude"]
  },
  "skills": {
    "refs": ["tdd", "code-review"]
  },
  "permissions": {
    "preset": "ship-standard"
  }
}
```

That's the minimum. Naming, providers, skills, permissions. Save the file, run `ship use dev`, and the configuration files your CLI agent expects appear automatically.

## Where agents live

- **Project agents** — `.ship/agents/` in your repo. Committed; shared with your team.
- **Library agents** — `~/.ship/agents/` on your machine. Personal; not committed.

## Compiling for several providers at once

Add more providers to `providers` and Ship compiles them all on `ship use`:

```jsonc
{
  "agent": {
    "providers": ["claude", "gemini", "codex"]
  }
}
```

Each provider gets its native format. The compiled files are gitignored build output — your `.ship/` profile is the source of truth.

## Next

- [Permissions](./permissions/) — the four built-in presets and how to layer your own rules.
