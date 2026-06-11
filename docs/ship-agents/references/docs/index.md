---
group: Agents
title: Agents
order: 1
---

An **agent** is a profile that tells Ship's first-party actor how to behave in your project — which skills it knows, which tools it can reach for, what it's allowed to do. You author the profile once in `.ship/agents/<name>.jsonc`; the runtime reads it as the actor's contract.

Every agent profile is local to a project. The agent's reach is its actor scope (`cwd`, `space_id`, permissions, transport, MCP tool gate). The kernel supervises the actor alongside every other actor in your project. You don't compile to a third-party harness — the agent runs in Ship.

## What goes into an agent

<sui-card padding="md">
<strong slot="header">Identity</strong>
A name and a description. The agent is addressable by both; the runtime uses the name for routing and the description for surface-level summaries.
</sui-card>

<sui-card padding="md">
<strong slot="header">Skills</strong>
Instruction sets that teach the agent how to do specific things. Pick from your local skills, from installed packages, or both.
</sui-card>

<sui-card padding="md">
<strong slot="header">Connectors</strong>
External tools the agent connects to via MCP — your own services, GitHub, anything MCP-aware. Declared in `.ship/connectors/`, gated by the actor's tool gate.
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

That's the minimum. Identity, skills, permissions. Save the file, run `ship use dev`, and the runtime boots the agent as a supervised actor in your active space.

## Where agents live

- **Project agents** — `.ship/agents/` in your repo. Committed; shared with your team.
- **Library agents** — `~/.ship/agents/` on your machine. Personal; not committed.

## External integrations

When the agent needs to reach for an external model or service, it does so through a connector. Connectors are declared in `.ship/connectors/` and gated by the same MCP tool gate that governs skills. The agent doesn't fork into another harness; the connector is the seam.

The connector surface is small by default — Ship ships with a handful of pre-vetted connectors (the model provider your machine is configured for, plus a few local services). Authoring a new connector is a one-page JSONC file. Reach for the [connectors reference](../../configuration/connectors) when you need one.

## Next

- [Permissions](./permissions/) — the four built-in presets and how to layer your own rules.
- [Configuration](./configuration/) — `ship.jsonc` fields, per-actor overrides, and connector declarations.
