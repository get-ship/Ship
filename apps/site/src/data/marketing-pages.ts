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
      "Ship's first-party agent runs in your active space. It reads your project, scaffolds a real view, queries your collections, writes artifacts.",
    body: `# Agents

Ship's first-party agent is the actor. It runs in your active space,
with its own scope (\`cwd\`, \`space_id\`, permissions, transport,
MCP tool gate), supervised by the runtime alongside every other
actor in your project.

## Five facets
- **Visual** — agents render a real surface (stats, charts, copy) you point at and annotate. Markup beats paragraphs.
- **Universal** — same agent profile powers the chat pane, the editor's inline agent, headless runs, and the CLI. One profile, every surface.
- **Configurable** — model, skills, permissions, connectors, rules, space — every facet is a field you tune in the GUI.
- **Programmable** — skills give tools; pipelines chain agents; triggers fire them on real events.
- **Observable** — every prompt, tool call, decision, and token lands on a live event-sourced bus. Scrub backwards.

## See also
- /orchestration · pipelines + triggers
- /observability · the event bus
- /custom-views · the surface agents render
- /security · the four layers
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

A space is a context with its own identity, agents, and data. Switch
spaces to switch what your agent sees.

## What a space is
- Its own views, agents, and data — yours to shape
- An accent color and identity
- A window of its own

## Switching
Switch spaces → the whole Ship reskins around the new context. The
active space is a per-connection concern — multiple spaces can be
open in parallel.

## Today
- Personal (private, local)
- Acme Team (shared, synced)
- Launch (focused, share-on-demand)

## See also
- /agents · agents scoped to a space
- /security · what the actor's tool gate enforces
`,
  },
  {
    slug: 'custom-views',
    title: 'SHIP — Custom Views',
    description:
      "The surface Ship shows you. Built with SUI — Ship's framework for agent-friendly UI. Hot reload in edit mode.",
    body: `# Custom Views

The surface Ship shows you. Authored with SUI, the Ship framework
optimised for agents.

## SUI
- Standards-based custom elements
- Reactive bindings over your collections + artifacts
- Hot reload in edit mode
- Annotate any element to pair it with an agent

## Edit mode
Inspect elements, drop annotations, pair them with an agent. The
code editor (ship-code-editor), terminal (ship-terminal), and
markdown editor (ship-markdown-editor) are vendored primitives you
can drop into any view.

## Packs
- **SUI primitives** — sui-card, sui-button, sui-sparkline, sui-bar-chart, sui-dialog, sui-tabs, …
- **Domain packs** — ship-agent-card, ship-pipeline-stepper, ship-run-log, ship-space-switcher, ship-event-feed, ship-collection-grid, ship-secret-input
- **Vendored** — ship-code-editor, ship-terminal, ship-markdown-editor, sui-command-bar, sui-launcher

## See also
- /agents · the surface they render
- /docs · the SUI tutorial
`,
  },
  {
    slug: 'security',
    title: 'SHIP — Security',
    description:
      'Four layers. Spaces, sandboxed code, the SUI renderable sandbox, and per-actor permissions.',
    body: `# Security

Four layers. An agent has to clear all of them before it does
anything risky.

## Layers (top to bottom)
1. **Spaces** — context, not a boundary. Two agents in two spaces never see each other's tools, files, or decision history unless they intentionally connect.
2. **Sandboxed code** — every view ships as a webview. The view's reach is exactly the SDK calls the compiler emitted; raw \`fetch\` from view code is a compile error.
3. **SES on SUI** — renderable expressions are an allowlist (slice, map, filter, includes, has, get, …). A view that calls \`window.eval\` or constructs a regex literal is a compile error.
4. **Permissions + gates** — every tool call is checked against the actor's resolved permission preset. A read-only agent cannot edit a file, full stop.

## See also
- /orchestration · gate steps in a pipeline
- /spaces · what contexts the actor lives in
`,
  },
  {
    slug: 'pricing',
    title: 'SHIP — Pricing',
    description:
      'Local stays free. Cloud-side features (sync, hosted spaces, compute) get priced once we lock them down.',
    body: `# Pricing

Local stays free. Cloud-side gets priced once we lock it down.

## Local — today, free
- Local-first storage — your data, your machine
- Bring your own model
- Every app, agent, view + skill
- Unlimited skills

## Pro · cloud-side — coming soon
Plans aren't locked yet. We'd rather wait than guess.
- **Device Sync** — your install follows you across machines (laptop, desktop, the spare in the closet).
- **Team Space** — invite teammates to a hosted space; agent activity is shared, your local-first guarantees hold on the space's event log.
- **Cloud compute** — optional; pay only for what you actually use.

Your local Ship keeps working either way.

## See also
- /download · install (public beta)
- /legal · terms
`,
  },
  {
    slug: 'docs',
    title: 'SHIP — Docs',
    description:
      'How Ship works — agents, pipelines, SUI views, collections, MCP. Curated guides; full reference lives with the SDK.',
    body: `# Docs

User-facing guides for Ship. Reference material lives separately.

## Get started
- [Getting started](https://getship.dev/docs/getting-started/) — install + first agent in 5 minutes.
- [SUI tutorial](https://getship.dev/docs/ship-ui/tutorial/) — render a real surface, step by step.

## Concepts
- [Agents](https://getship.dev/docs/agents/) — Ship's first-party agent and its actor contract.
- [Spaces](https://getship.dev/docs/concepts/spaces/) — a space is a context, not a boundary.
- [Security](https://getship.dev/docs/architecture/security/) — the four layers.
- [Local-first](https://getship.dev/docs/concepts/local-first/) — runtime, daemon, and data on your machine.
- [Runtime model](https://getship.dev/docs/architecture/runtime-model/) — events, projections, the four primitives.

## Operations
- [Pipelines](https://getship.dev/docs/agents/) — compose, trigger, replay.
- [Collections + artifacts](https://getship.dev/docs/concepts/data/) — schemaful rows + content-addressed blobs.
- [MCP integration](https://getship.dev/docs/agents/) — plug in external tools via connectors.

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
