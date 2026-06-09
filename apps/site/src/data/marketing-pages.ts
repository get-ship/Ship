/*
 * Marketing-page SSOT — title, meta description, and an agent-friendly
 * markdown body for every surface in the OS.
 *
 *   • The desktop windows render the rich Astro UI from these slugs.
 *   • `[surface].astro` reads `title`/`desc` here for prerendered HTML.
 *   • `[surface].md.ts` serves the `body` field as plain markdown so
 *     scrapers, LLMs, and the `/llms.txt` indexer have a clean,
 *     chrome-free source of truth.
 *
 * Keep bodies short and structural — the rendered window is the
 * marketing surface; the .md mirror is the *index card* for an agent
 * deciding whether to follow a link. Bullet points + a one-line
 * summary beat paragraphs.
 */

export interface MarketingPage {
  slug: string;
  title: string;
  description: string;
  body: string;
}

const PAGES: MarketingPage[] = [
  {
    slug: 'agents',
    title: 'SHIP — Agents',
    description:
      'A Ship agent is more than a chatbot. One config compiles to Claude Code, Codex, Cursor, OpenCode, and pi.',
    body: `# Agents

A Ship agent is more than a chatbot. Same config, every model.

## Five facets
- **Visual** — agents render a real surface (stats, charts, copy) you point at and annotate. Markup beats paragraphs.
- **Universal** — one agent profile compiles to each harness's native format (Claude Code, Codex, Cursor, OpenCode, pi). No fork.
- **Configurable** — model, skills, permissions, MCP servers, rules, space — every facet is a field you tune in the GUI.
- **Programmable** — skills give tools; pipelines chain agents; triggers fire them on real events.
- **Observable** — every prompt, tool call, decision, and token lands on a live event-sourced bus. Scrub backwards.

## See also
- /orchestration · pipelines + triggers
- /observability · the event bus
- /custom-views · the surface agents render
`,
  },
  {
    slug: 'orchestration',
    title: 'SHIP — Orchestration',
    description:
      'Pipelines chain agents, human review, and custom outputs. Triggers fire them on real events.',
    body: `# Orchestration

Pipelines + triggers. Compose workflows; trigger on anything.

## Pipelines
Three step kinds, composed in order:
- **Agent step** — any Ship agent, same config as the Agents surface.
- **Human review** — pause until you (or a teammate) approve or edit.
- **Custom output** — where the result lands: email, file write, HTTP, MCP tool call, view binding.

Pipelines are code — versioned, shareable, forkable. Restartable from any failed step.

## Triggers
Anything in Ship is an event:
- schedules (cron, timezone-aware)
- MCP servers (gmail, github, linear, …)
- webhooks
- file watcher
- agent.events (completion · gate · output)
- pipeline.events (step · finish)
- view emits (a button in any view)
- you.emit (your own custom topics)

A pipeline can have many triggers; one trigger can fire many pipelines. Same bus.

## See also
- /agents · the actors a pipeline composes
- /observability · how to watch a run replay
`,
  },
  {
    slug: 'data',
    title: 'SHIP — Data',
    description:
      'Two primitives: schemaful collections and content-addressed artifacts. Local-first, reactive, optional cloud sync.',
    body: `# Data

Two primitives. Both live on your machine. Reactive by default.

## Collections
- Schemaful rows, indexed for query
- Skills read from them; views subscribe and update live
- Subscribed by any number of views — updates are pushed, not polled

## Artifacts
- Content-addressed blobs typed by your declarations
- Same hash = same bytes
- Pipelines write outputs (transcripts, files, frames) as artifacts; agents reference them by hash

## Composition
- Skills query collections + declare artifact types
- Pipelines write outputs into both
- Views subscribe and re-render

Optional cloud sync is coming. Local-first means your data, your machine — even when the cloud side ships.

## See also
- /agents · skills + permissions over data
- /orchestration · pipelines that read + write
`,
  },
  {
    slug: 'observability',
    title: 'SHIP — Observability',
    description:
      'Every prompt, tool call, decision, and token — captured the moment it happens. Replay any moment.',
    body: `# Observability

Capture is the foundation. Governance, replay, and per-agent metrics build on it.

## Event capture
Every surface — agents, tools, pipelines, views — publishes to one bus. Captured event types include:
- \`agent.prompt\` · \`agent.decision\`
- \`tool.call\` · \`tool.result\`
- \`token.usage\` (per provider, per cost)
- \`pipeline.step\` · \`pipeline.finish\`
- \`gate.request\` (human approval)
- \`view.update\` (any view that re-renders)

## Live monitor
Per-space cards show active agents and recent throughput. Scrub backwards to replay any moment.

## What's here today
- Live capture
- Replay
- Per-agent metrics

## On the way
- Sharing observations across spaces
- Integrations into your other tools
- Feedback loops that nudge agents to improve

## See also
- /agents · the actors emitting events
- /orchestration · pipelines emit step events
`,
  },
  {
    slug: 'spaces',
    title: 'SHIP — Spaces',
    description:
      'A space holds its own views, agents, and data. Switch spaces and the whole Ship reskins.',
    body: `# Spaces

Isolation + collaboration. Same binary, separate worlds.

## What a space is
- Its own views, agents, and data — isolated and yours to shape
- An accent color and identity
- Optional sync with teammates

## Switching
Switch spaces → the whole Ship reskins around the new context. Personal stays on your machine; shared spaces sync across people.

## Today
- Personal (private, local)
- Acme Team (shared, synced)
- Launch (focused, share-on-demand)

## See also
- /security · per-space isolation strategy
- /agents · agents scoped to a space
`,
  },
  {
    slug: 'custom-views',
    title: 'SHIP — Custom Views',
    description:
      'The surface Ship shows you. Built with SUI — Ship\'s framework for agent-friendly UI. Hot reload in edit mode.',
    body: `# Custom Views

The surface Ship shows you. Authored with SUI, the Ship framework optimised for agents.

## SUI
- Standards-based custom elements
- Reactive bindings over your collections + artifacts
- Hot reload in edit mode
- Annotate any element to pair it with an agent

## Edit mode
Inspect elements, drop annotations, pair them with an agent. Optional Ship Studio for source-level edits.

## Packs
- **SUI primitives** — sui-card, sui-sparkline, sui-bar-chart, sui-button, …
- **Domain packs** — ship-agent-card, ship-pipeline-step, …
- **Vendored** — markdown editor, terminal, code editor

## See also
- /agents · the surface they render
- /docs · SUI tutorial (coming)
`,
  },
  {
    slug: 'security',
    title: 'SHIP — Security',
    description:
      'Without Ship: full filesystem, full shell. With Ship: layered isolation per space and per surface.',
    body: `# Security

Without Ship, an AI tool has the keys to your whole machine. With Ship, isolation is the default.

## Layers (top to bottom)
1. **Spaces** — per-space isolation strategy, chosen per OS
2. **Sandboxed code** — view code in webviews, scripts in isolated runtimes, downloaded apps in single-space sandboxes
3. **SES on SUI** — no fetch from view code, capability-scoped
4. **Permissions + gates** — every privileged action waits for an approval

## See also
- /orchestration · gate steps in a pipeline
- /spaces · per-space isolation
`,
  },
  {
    slug: 'pricing',
    title: 'SHIP — Pricing',
    description:
      'Free on your machine forever. Cloud-side features (sync, hosted spaces, compute) will be priced separately.',
    body: `# Pricing

Local stays free. Cloud-side gets priced once we lock it down.

## Local — today, free
- Local-first storage — your data, your machine
- Bring your own model
- Every app, agent, view + skill
- Unlimited skills

## Pro · cloud-side — coming soon
Plans aren't locked yet. We'd rather wait than guess.
- Sync your install across devices
- Hosted spaces · share with a team
- Cloud compute · optional
- Pay only for what you actually use

Your local Ship keeps working either way.

## See also
- /download · install (lands with public beta)
- /legal · terms
`,
  },
  {
    slug: 'docs',
    title: 'SHIP — Docs',
    description:
      'How Ship works — agents, pipelines, SUI views, collections, MCP. Curated guides; full reference at /reference (soon).',
    body: `# Docs

User-facing guides for Ship. Reference material lives separately.

## Coming soon
- Getting started · install + first agent in 5 minutes
- Agent basics · the five facets in code
- Pipelines · compose, trigger, replay
- SUI tutorial · render a real surface
- MCP integration · plugging in external tools
- Collections + artifacts

Full API reference will live at /reference once the public beta lands.

## See also
- /agents · /orchestration · /custom-views · /data · /security
`,
  },
  {
    slug: 'arcade',
    title: 'SHIP — Arcade',
    description:
      'Three short games built on @ship/arcade. Open primitives, one tiny engine, real custom elements.',
    body: `# Arcade

Three games built on \`@ship/arcade\`. Open primitives, one tiny engine, real custom elements.

## Cabinets
- **Galaga** · shooter — defend the formation
- **Particle Garden** · strategy — drop seeds, hold ground
- **Magnetar** · puzzle — bend the ship's path home through planets and pulsars

## Source
- \`packages/arcade-engine\` — engine + vendored pixi
- \`packages/components/ship/arcade\` — sui-stage, sui-sprite, sui-emitter, sui-input, sui-hud
- \`apps/site/src/games\` — the games themselves
`,
  },
  {
    slug: 'download',
    title: 'SHIP — Download',
    description:
      'Get Ship. Runs on your machine, bring your own model. Free for personal use.',
    body: `# Download

Ship runs on your machine, brings its own runtime, and respects your data.

## Platforms (public beta)
- macOS 10.15+
- Windows 10+
- Linux (x86_64 · arm64)

## Today
Installers land with the public beta. Pick your platform on /download and we'll email you the moment yours is ready.

## What you get
- Local-first storage
- Bring your own model
- Free for personal use
`,
  },
];

export const marketingPages: Readonly<MarketingPage[]> = PAGES;

export const pageBySlug: Readonly<Record<string, MarketingPage>> = Object.freeze(
  Object.fromEntries(PAGES.map((p) => [p.slug, p])),
);
