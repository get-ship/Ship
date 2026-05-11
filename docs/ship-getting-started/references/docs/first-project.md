---
group: Getting Started
title: Your First Agent
section: guide
order: 3
---

Open Ship, open your project, and add an agent.

## In the app

From the dashboard, open **Agents** and create a new one. You'll be asked for:

- **A name** — anything that describes what the agent does (e.g. "Backend", "Reviewer").
- **A provider** — Claude, Gemini, Codex, Cursor, or OpenCode.
- **Skills** — pick from the registry or scaffold your own.
- **Permissions** — pick a preset (`ship-readonly`, `ship-standard`, `ship-autonomous`, `ship-elevated`) or fine-tune.

Save. Ship compiles the agent into the configuration files your CLI tool expects (`CLAUDE.md`, `.cursor/rules`, `GEMINI.md`, `AGENTS.md`, etc.) so the next time you launch your CLI agent against this project, it picks up the new config automatically.

## From the command line

The same things are available as CLI commands if you prefer. See the [CLI Reference](/cli/) for `ship init`, `ship use`, `ship compile`, and `ship add`.

## What's next

- [Smart Skills](/smart-skills/) — how skills and typed variables work.
- [Agents](/agents/) — the full shape of an agent profile.
- [Registry](/registry/) — installing and publishing skill packages.
