---
title: Ship
description: Ship — the multi-agent coordination runtime. Compile declarative config into provider-native artifacts, run pipelines across git worktrees, coordinate agents through an event-sourced kernel.
---

The multi-agent coordination runtime. Declarative config compiles to provider-native artifacts; pipelines run across git worktrees; events coordinate everything.

<div style="display: flex; gap: 0.75rem; margin: 1rem 0 2rem;">
  <a href="/reference/getting-started/"><ship-button tone="primary">Get Started →</ship-button></a>
  <a href="https://getship.dev/registry"><ship-button variant="outline">Registry</ship-button></a>
</div>

## What Ship gives you

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1rem; margin: 1.5rem 0;">

<ship-card padding="md">
<strong slot="header">Smart Skills</strong>
Instruction sets with typed variables. One skill, personalized output per user and project.
</ship-card>

<ship-card padding="md">
<strong slot="header">Agent Profiles</strong>
Compose skills, MCP servers, permissions, and rules into portable configs for every provider.
</ship-card>

<ship-card padding="md">
<strong slot="header">Pipelines</strong>
YAML DAGs of agent steps. Gates, retries, worktree isolation, resumable on crash.
</ship-card>

<ship-card padding="md">
<strong slot="header">Views</strong>
Author live-updating HTML surfaces for your data in `.ship/views/*`. Zero build step.
</ship-card>

<ship-card padding="md">
<strong slot="header">Collections</strong>
Schema-backed structured data. Specs, ADRs, notes, anything — queryable and event-driven.
</ship-card>

<ship-card padding="md">
<strong slot="header">Triggers</strong>
Reactive JSONC that matches events and runs actions. Automation without writing code.
</ship-card>

</div>

## Find your way around

The docs are split by audience — pick the one that matches what you're doing.

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 0.75rem; margin: 1.5rem 0;">

<ship-card padding="md" interactive>
<strong slot="header"><a href="/reference/getting-started/" style="text-decoration: none; color: inherit;">Guide →</a></strong>
Install, init, your first project. How skills, agents, and pipelines fit together.
</ship-card>

<ship-card padding="md" interactive>
<strong slot="header"><a href="/reference/cli/" style="text-decoration: none; color: inherit;">Reference →</a></strong>
CLI commands, MCP tools, schemas, config shape. Look things up.
</ship-card>

<ship-card padding="md" interactive>
<strong slot="header"><a href="/reference/smart-skills/" style="text-decoration: none; color: inherit;">Extensions →</a></strong>
Build on Ship. Seven extension mechanisms: skills, agents, views, pipelines, artifacts, collections, triggers.
</ship-card>

<ship-card padding="md" interactive>
<strong slot="header"><a href="/components/" style="text-decoration: none; color: inherit;">Components →</a></strong>
The ship-elements library. Tokens, primitives, themes, cookbook.
</ship-card>

</div>

## Supported providers

- **Claude Code** — `CLAUDE.md`, `.mcp.json`, `.claude/skills/`
- **Cursor** — `.cursor/rules`, `.cursor/mcp.json`
- **Gemini CLI** — `GEMINI.md`, `.gemini/settings.json`
- **Codex** — `AGENTS.md`, `.codex/config.toml`
- **OpenCode** — `opencode.json`

## Quick start

```bash
curl -fsSL https://getship.dev/install | sh
cd your-project
ship init
ship studio --open
```

<ship-alert tone="info">
<strong slot="title">Migrating from existing provider config?</strong>
Ship converts <code>CLAUDE.md</code>, <code>.cursor/rules</code>, and other provider configs into <code>.ship/</code> format. Run <code>ship convert .</code> in your project root.
</ship-alert>
