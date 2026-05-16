---
title: Ship
description: Ship — the multi-agent coordination runtime. Compile declarative config into provider-native artifacts, run pipelines across git worktrees, coordinate agents through an event-sourced kernel.
---

The multi-agent coordination runtime. Declarative config compiles to provider-native artifacts; pipelines run across git worktrees; events coordinate everything.

<div style="display: flex; gap: 0.75rem; margin: 1rem 0 2rem;">
  <a href="/getting-started/"><sui-button tone="primary">Get Started →</sui-button></a>
  <a href="https://getship.dev/registry"><sui-button variant="outline">Registry</sui-button></a>
</div>

## What Ship gives you

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1rem; margin: 1.5rem 0;">

<sui-card padding="md">
<strong slot="header">Smart Skills</strong>
Instruction sets with typed variables. One skill, personalized output per user and project.
</sui-card>

<sui-card padding="md">
<strong slot="header">Agent Profiles</strong>
Compose skills, MCP servers, permissions, and rules into portable configs for every provider.
</sui-card>

<sui-card padding="md">
<strong slot="header">Pipelines</strong>
YAML DAGs of agent steps. Gates, retries, worktree isolation, resumable on crash.
</sui-card>

<sui-card padding="md">
<strong slot="header">Views</strong>
Author live-updating HTML surfaces for your data in `.ship/views/*`. Zero build step.
</sui-card>

<sui-card padding="md">
<strong slot="header">Collections</strong>
Schema-backed structured data. Specs, ADRs, notes, anything — queryable and event-driven.
</sui-card>

<sui-card padding="md">
<strong slot="header">Triggers</strong>
Reactive JSONC that matches events and runs actions. Automation without writing code.
</sui-card>

</div>

## Find your way around

The docs are split by audience — pick the one that matches what you're doing.

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 0.75rem; margin: 1.5rem 0;">

<sui-card padding="md" interactive>
<strong slot="header"><a href="/getting-started/" style="text-decoration: none; color: inherit;">Guide →</a></strong>
Install, init, your first project. How skills, agents, and pipelines fit together.
</sui-card>

<sui-card padding="md" interactive>
<strong slot="header"><a href="/cli/" style="text-decoration: none; color: inherit;">Reference →</a></strong>
CLI commands, MCP tools, schemas, config shape. Look things up.
</sui-card>

<sui-card padding="md" interactive>
<strong slot="header"><a href="/smart-skills/" style="text-decoration: none; color: inherit;">Extensions →</a></strong>
Build on Ship. Seven extension mechanisms: skills, agents, views, pipelines, artifacts, collections, triggers.
</sui-card>

<sui-card padding="md" interactive>
<strong slot="header"><a href="/components/" style="text-decoration: none; color: inherit;">Components →</a></strong>
The SUI components library. Tokens, primitives, themes, cookbook.
</sui-card>

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

<sui-alert tone="info">
<strong slot="title">Migrating from existing provider config?</strong>
Ship converts <code>CLAUDE.md</code>, <code>.cursor/rules</code>, and other provider configs into <code>.ship/</code> format. Run <code>ship convert .</code> in your project root.
</sui-alert>

## SUI, live

The docs site renders with the same component runtime it documents. Every tag below is a real `<sui-*>` element, registered globally by `/sui-components.js` — pick a theme, drive the controls, watch it react.

<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:1.25rem;margin:1.5rem 0;">

<sui-card variant="outline" padding="lg">
<strong slot="header" style="font-weight:700;font-size:15px;">Theme</strong>
<p style="color:var(--muted-foreground);font-size:13px;line-height:1.5;margin:0 0 12px;">The picker writes the same <code>data-theme</code> attribute the rest of the site reacts to.</p>
<sui-theme-picker label-style="text"></sui-theme-picker>
</sui-card>

<sui-card variant="outline" padding="lg">
<strong slot="header" style="font-weight:700;font-size:15px;">Form primitives</strong>
<div style="display:flex;flex-direction:column;gap:8px;margin:8px 0 12px;">
  <sui-input placeholder="agent name"></sui-input>
  <sui-select>
    <option>plan</option>
    <option selected>review</option>
    <option>qa</option>
  </sui-select>
</div>
<sui-toolbar justify="end">
  <sui-button variant="ghost" size="sm">Cancel</sui-button>
  <sui-button tone="primary" variant="solid" size="sm">Dispatch</sui-button>
</sui-toolbar>
</sui-card>

<sui-card variant="outline" padding="lg">
<strong slot="header" style="font-weight:700;font-size:15px;">Pipeline status</strong>
<div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;margin:8px 0 0;">
  <sui-pipeline-step name="collect" step-type="run" state="completed" agent="git log"></sui-pipeline-step>
  <sui-pipeline-step name="review" step-type="agent" state="completed" agent="Claude"></sui-pipeline-step>
  <sui-pipeline-step name="gate" step-type="gate" state="running" agent="acceptance"></sui-pipeline-step>
  <sui-pipeline-step name="merge" step-type="merge" state="pending" agent="fast-forward"></sui-pipeline-step>
</div>
</sui-card>

</div>
