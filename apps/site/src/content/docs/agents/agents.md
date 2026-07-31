---
title: Agents
description: An agent is a declarative profile — skills, tools, permissions — that compiles to whatever harness you already use.
section: Agents
order: 1
kicker: Agents
tags: [agents]
related: [permissions, harnesses, skills]
---

An **agent** is a profile that tells an AI assistant how to behave in your space — which skills it knows, which tools it can reach for, what it's allowed to do. You author it once in `.ship/agents/<name>.jsonc`; Ship compiles it into the configuration each harness expects.

## What goes into an agent

- **Identity** — a name and description.
- **Skills** — instruction sets that teach the agent how to do specific things (from your own skills, installed packages, or both).
- **MCP servers** — external tools the agent reaches over Model Context Protocol — your services, GitHub, anything MCP-aware.
- **Permissions** — a preset controlling what it can do without asking. See [Permissions](/docs/permissions/).

## A small agent

```jsonc
{
  "agent": { "id": "dev", "name": "Dev" },
  "skills": { "refs": ["tdd", "code-review"] },
  "permissions": { "preset": "ship-standard" }
}
```

That's the minimum — identity, skills, permissions. Save it, run `ship use dev`, and Ship emits one config file per harness.

## Where agents live

- **Space agents** — `.ship/agents/` in your space; committed and shared with your team.
- **Library agents** — `~/.ship/agents/` on your machine; personal, not committed.

## One profile, many harnesses

The project's `harnesses` list (in `ship.jsonc`) decides which harnesses get compiled — every profile compiles for every harness in the list:

```jsonc
// ship.jsonc
{ "harnesses": ["claude", "gemini", "codex"] }
```

Each harness gets its native format; the compiled files are build output, and your `.ship/` profile is the source of truth. When one harness needs a different setting, override it under `harness_settings.<harness>`. More in [Harnesses](/docs/harnesses/).
