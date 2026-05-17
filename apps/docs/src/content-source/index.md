---
title: Ship
description: Ship is the runtime for agent workflows. Agents write code, run pipelines, build live HTML views — Ship gives them the rails.
---

<div class="not-content idx-hero">
  <div class="idx-hero-band" aria-hidden="true"></div>
  <div class="idx-hero-inner">
    <p class="idx-eyebrow">The runtime for agent workflows</p>
    <h1 class="idx-title">Code. Pipelines. Live views.<br/><span class="idx-title-accent">One declarative tree.</span></h1>
    <p class="idx-lede">
      Ship gives agents rails — typed skills, JSONC pipelines on git worktrees, and a
      reactive HTML surface they can author in one shot. One config tree, every provider.
    </p>
    <div class="idx-cta-row">
      <a class="idx-cta idx-cta-primary" href="/tutorial/01-hello-signal/">Try the SUI tutorial →</a>
      <a class="idx-cta idx-cta-secondary" href="/getting-started/">60-second install</a>
      <a class="idx-cta idx-cta-ghost" href="/components/">Browse components →</a>
    </div>
    <div class="idx-pill-row">
      <span class="idx-pill"><span class="idx-pill-dot idx-dot-primary"></span> Claude · Cursor · Gemini · Codex · OpenCode</span>
      <span class="idx-pill"><span class="idx-pill-dot idx-dot-success"></span> Event-sourced kernel</span>
      <span class="idx-pill"><span class="idx-pill-dot idx-dot-info"></span> Resumable pipelines</span>
    </div>
  </div>
</div>

<div class="not-content idx-three">
  <a class="idx-three-card idx-card-primary" href="/agents/">
    <div class="idx-three-icon" aria-hidden="true">⌘</div>
    <div class="idx-three-title">Write code</div>
    <p class="idx-three-body">Agents read + write your tree under typed skills, permissions, and job/session primitives. Compiles to every major provider format.</p>
  </a>
  <a class="idx-three-card idx-card-info" href="/pipelines/">
    <div class="idx-three-icon" aria-hidden="true">⇢</div>
    <div class="idx-three-title">Run pipelines</div>
    <p class="idx-three-body">JSONC DAGs of agent steps with gates, retries, and worktree isolation. Resumable across kernel restarts.</p>
  </a>
  <a class="idx-three-card idx-card-success" href="/tutorial/01-hello-signal/">
    <div class="idx-three-icon" aria-hidden="true">◈</div>
    <div class="idx-three-title">Build live views</div>
    <p class="idx-three-body"><code>.sui</code> templates with optional controllers. Signal-driven, AOT-compiled, hydration-free. The deepest surface — agent-authored end to end.</p>
  </a>
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

## SUI, live

The docs site renders with the same component runtime it documents. Every tag below is a real `<sui-*>` element — pick a theme, drive the controls, watch it react.

<div class="not-content idx-live-grid">

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
  <sui-button variant="outline" size="sm">Cancel</sui-button>
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

## 60-second install

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

## Supported providers

- **Claude Code** — `CLAUDE.md`, `.mcp.json`, `.claude/skills/`
- **Cursor** — `.cursor/rules`, `.cursor/mcp.json`
- **Gemini CLI** — `GEMINI.md`, `.gemini/settings.json`
- **Codex** — `AGENTS.md`, `.codex/config.toml`
- **OpenCode** — `opencode.json`
