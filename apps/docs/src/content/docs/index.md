---
title: Ship
description: Ship is the runtime for agent workflows. Agents write code, run pipelines, build live HTML views — Ship gives them the rails.
---

<div style="display: flex; flex-direction: column; gap: 0.5rem; margin: 0 0 1rem;">
  <p style="font-size: 1.125rem; line-height: 1.5; color: var(--foreground); margin: 0; max-width: 42rem;">
    Ship is the runtime for agent workflows.
    Agents write code, run pipelines, build live HTML views — Ship gives them the rails.
  </p>
  <p style="font-size: 0.9375rem; line-height: 1.5; color: var(--muted-foreground); margin: 0; max-width: 42rem;">
    One declarative config tree, every provider. Skills compile to provider-native artifacts.
    Pipelines run across git worktrees. SUI views are the deepest part of the surface — live,
    ergonomic, agent-authored.
  </p>
</div>

<div style="display: flex; gap: 0.5rem; flex-wrap: wrap; margin: 0.75rem 0 2rem;">
  <a href="/tutorial/01-hello-signal/"><sui-button tone="primary" variant="solid">Try the SUI tutorial →</sui-button></a>
  <a href="/getting-started/"><sui-button variant="outline">60-second install</sui-button></a>
  <a href="/components/"><sui-button variant="ghost">Browse components</sui-button></a>
</div>

<div class="screenshot-todo">
<strong>SCREENSHOT</strong> — Ship Studio shell: session view active in the main pane, an agent
dispatched and visibly working in the side rail, dev panel open at the bottom showing the Source
tab with a live <code>.sui</code> file. Approx 1440×900, ship-light theme. Use Playwright against
a live daemon with at least one job mid-flight.
</div>

## How agents work in Ship

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1rem; margin: 1.5rem 0;">

<sui-card padding="md">
<strong slot="header">Write code</strong>
Agents read and write your codebase using Ship's job + session primitives. Permissions, tool gates,
and skill-shaped instructions keep them on rails. Compiles to <code>CLAUDE.md</code>,
<code>.cursor/rules</code>, <code>GEMINI.md</code>, <code>AGENTS.md</code>, <code>opencode.json</code>.
</sui-card>

<sui-card padding="md">
<strong slot="header">Run pipelines</strong>
JSONC DAGs of agent steps. Gates, retries, worktree isolation, resumable on crash. Each step runs
in its own agent profile. Long-running work survives kernel restarts.
</sui-card>

<sui-card padding="md">
<strong slot="header">Build live views</strong>
<code>.sui</code> templates with optional paired controllers. Signal-driven, AOT-compiled, SSR'd,
attached client-side. Zero build step. The deepest surface — and the one agents most often
co-author with humans. <a href="/tutorial/01-hello-signal/">Try it →</a>
</sui-card>

</div>

## What a SUI view looks like

```sui
<!-- views/counter/page.sui -->
<prop name="initial" type="number" default="0" />

<div class="counter">
  <p>count: <strong>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>+1</button>
</div>
```

```ts
// views/counter/controller.ts
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ initial }) => ({ count: signal(initial) }),
})
```

That's the whole surface. The compiler emits a single ESM, the daemon SSRs it, the client
attaches reactivity. Compose 100+ `<sui-*>` primitives.
[Try this in the browser →](/tutorial/01-hello-signal/)

## The supporting cast

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1rem; margin: 1.5rem 0;">

<sui-card padding="md">
<strong slot="header">Skills</strong>
Typed instruction sets with variables. One skill, personalized output per user and project.
</sui-card>

<sui-card padding="md">
<strong slot="header">Agent profiles</strong>
Compose skills, MCP servers, permissions, and rules into portable configs for every provider.
</sui-card>

<sui-card padding="md">
<strong slot="header">Collections</strong>
Schema-backed structured data. Specs, ADRs, notes — queryable and event-driven.
</sui-card>

<sui-card padding="md">
<strong slot="header">Triggers</strong>
Reactive JSONC that matches events and runs actions. Automation without writing code.
</sui-card>

<sui-card padding="md">
<strong slot="header">Components</strong>
Two published packs — <code>@ship/primitives</code> (~80 generic <code>&lt;sui-*&gt;</code> elements) and <code>@ship/charts</code>. Drop into any view.
</sui-card>

<sui-card padding="md">
<strong slot="header">Agent surfaces</strong>
47 MCP tools, a Node + browser LSP, <code>sui-eval</code> regression scoring.
The agent has every surface it needs to author + verify in one turn.
<a href="/agents/agent-surfaces/">Reference →</a>
</sui-card>

</div>

## 60-second install

```bash
curl -fsSL https://getship.dev/install | sh
cd your-project
ship init
ship studio --open
```

<div class="screenshot-todo">
<strong>SCREENSHOT</strong> — <code>ship studio --open</code> result: browser landing on the
Ship Studio home view with the workspace switcher visible top-left, no active session yet. Use
this to give new visitors a sense of "what they see after install". Approx 1440×900.
</div>

<sui-alert tone="info">
<strong slot="title">Migrating from existing provider config?</strong>
Ship converts <code>CLAUDE.md</code>, <code>.cursor/rules</code>, and other provider configs into <code>.ship/</code> format. Run <code>ship convert .</code> in your project root.
</sui-alert>

## Supported providers

- **Claude Code** — `CLAUDE.md`, `.mcp.json`, `.claude/skills/`
- **Cursor** — `.cursor/rules`, `.cursor/mcp.json`
- **Gemini CLI** — `GEMINI.md`, `.gemini/settings.json`
- **Codex** — `AGENTS.md`, `.codex/config.toml`
- **OpenCode** — `opencode.json`

## SUI, live

The docs site renders with the same component runtime it documents. Every tag below is a real `<sui-*>` element — pick a theme, drive the controls, watch it react.

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
