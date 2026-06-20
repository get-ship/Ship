---
title: Your first agent
description: Create an agent in your space — provider, skills, permissions — and watch Ship compile it to your harness config.
section: Getting started
order: 2
kicker: From the dashboard, in a few clicks
tags: [agents, walkthrough]
related: [quick-start]
---

An agent in Ship is a declarative profile — who it is, what it can do, what it's allowed to touch. You compose it in the app; Ship compiles it to the config files your coding agent already reads.

## In the app

From the dashboard, open **Agents** and create a new one. You'll set:

- **Name** — whatever describes the job (e.g. "Backend", "Reviewer").
- **Provider** — Claude, Gemini, Codex, Cursor, or OpenCode.
- **Skills** — pick from the registry or scaffold your own.
- **Permissions** — choose a preset (`ship-readonly`, `ship-standard`, `ship-autonomous`, `ship-elevated`) or fine-tune.

Save, and Ship compiles the agent into the configuration your tool expects — `CLAUDE.md`, `.cursor/rules`, `GEMINI.md`, `AGENTS.md`, and so on. The next time you launch your coding agent against this space, it picks up the new config automatically.

## From the command line

The same things are available as CLI commands if you'd rather stay in the terminal — `ship use`, `ship compile`.

## What's next

- **[What is Ship](/docs/what-is-ship/)** — the mental model the agent fits inside.
- **[Building views](/docs/what-is-sui/)** — give your agent a real surface, not a chat box.
