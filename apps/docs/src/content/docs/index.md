---
title: Introduction
description: Ship — get the most out of your AI agents with skills, compilation, and a visual studio.
template: splash
hero:
  tagline: Get the most out of your AI agents. Ship extends the Agent Skills specification with typed variables, multi-provider compilation, and a visual studio.
  actions:
    - text: Get Started
      link: /reference/getting-started/
      icon: right-arrow
    - text: Browse the Registry
      link: https://getship.dev/registry
      variant: minimal
---

## How it works

1. **Declare** — `ship init` scaffolds your `.ship/` config. Add skills and agents from the registry or write your own.
2. **Activate** — Set an active agent. Ship compiles everything into provider-native config.
3. **Run** — Each provider gets its own output format. Same source, every provider.

## What Ship gives you

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1rem; margin: 1.5rem 0;">

<ship-card padding="md">
<strong slot="header">Smart Skills</strong>
Instruction sets with typed variables that adapt to each user and project. Same skill, personalized output for everyone.
</ship-card>

<ship-card padding="md">
<strong slot="header">Agent Profiles</strong>
Compose skills, MCP servers, permissions, and rules into portable agent configs.
</ship-card>

<ship-card padding="md">
<strong slot="header">Ship Studio</strong>
A visual IDE connected to your local projects. Edit skills, set variables, browse the registry.
</ship-card>

<ship-card padding="md">
<strong slot="header">Registry</strong>
Publish and install at getship.dev. Content-hashed, versioned, verified.
</ship-card>

</div>

## Supported providers

- **Claude Code** — CLAUDE.md, .mcp.json, .claude/skills/
- **Cursor** — .cursor/rules, .cursor/mcp.json
- **Gemini CLI** — GEMINI.md, .gemini/settings.json
- **Codex** — AGENTS.md, .codex/config.toml
- **OpenCode** — opencode.json

## Quick start

```bash
curl -fsSL https://getship.dev/install | sh
cd your-project
ship init
ship studio --open
```

<ship-alert tone="info">
<strong slot="title">Migrating from existing provider config?</strong>
Ship converts CLAUDE.md, .cursor/rules, and other provider configs into `.ship/` format. Run `ship convert .` in your project root.
</ship-alert>
