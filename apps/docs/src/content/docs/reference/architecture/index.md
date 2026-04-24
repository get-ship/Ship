---
title: "Architecture Overview"
description: "How Ship is built — transport, daemon, runtime, kernel, compiler — and how they interleave."
sidebar:
  label: "Architecture Overview"
  order: 1
---
<figure style="margin: 2rem 0;">
<svg viewBox="0 0 880 560" xmlns="http://www.w3.org/2000/svg" role="img" aria-labelledby="arch-diagram-title" style="width: 100%; height: auto; font-family: var(--font-body); color: var(--foreground);">
  <title id="arch-diagram-title">Ship architecture — transport forwards requests to the daemon or calls the compiler directly; the daemon hosts the runtime; the runtime wraps the kernel; events persist to SQLite.</title>
  <defs>
    <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor" opacity="0.7"/>
    </marker>
    <marker id="arrow-accent" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--accent)" opacity="0.85"/>
    </marker>
    <marker id="arrow-success" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--success)" opacity="0.85"/>
    </marker>
    <style>
      .region       { stroke-width: 1; rx: 14; ry: 14; transition: filter 200ms var(--ease-out, ease-out); }
      .region:hover { filter: brightness(1.04) drop-shadow(0 2px 8px color-mix(in oklch, var(--foreground) 15%, transparent)); }
      .region-transport { fill: color-mix(in oklch, var(--accent) 10%, var(--card));  stroke: color-mix(in oklch, var(--accent) 45%, var(--border)); }
      .region-daemon    { fill: color-mix(in oklch, var(--info) 9%, var(--card));     stroke: color-mix(in oklch, var(--info) 40%, var(--border)); }
      .region-runtime   { fill: color-mix(in oklch, var(--muted) 55%, var(--card));   stroke: var(--border); }
      .region-kernel    { fill: var(--background); stroke: color-mix(in oklch, var(--primary) 55%, var(--border)); stroke-dasharray: 4 3; }
      .region-compiler  { fill: color-mix(in oklch, var(--success) 10%, var(--card)); stroke: color-mix(in oklch, var(--success) 45%, var(--border)); }
      .region-storage   { fill: var(--card); stroke: var(--border); }
      .label-title  { font-family: var(--font-display); font-weight: 700; font-size: 15px; letter-spacing: -0.01em; fill: var(--foreground); }
      .label-sub    { font-size: 11.5px; fill: var(--muted-foreground); font-family: var(--font-body); }
      .chip         { font-size: 11px; fill: var(--foreground); font-family: var(--font-mono, ui-monospace, monospace); }
      .chip-bg      { fill: var(--background); stroke: var(--border); stroke-width: 1; rx: 5; ry: 5; }
      .flow         { fill: none; stroke-width: 1.5; opacity: 0.85; }
      .flow-http    { stroke: color-mix(in oklch, var(--info)    70%, var(--foreground)); }
      .flow-compile { stroke: color-mix(in oklch, var(--success) 70%, var(--foreground)); }
      .flow-persist { stroke: var(--foreground); opacity: 0.55; stroke-dasharray: 3 3; }
      .flow-label   { font-size: 10.5px; fill: var(--muted-foreground); font-family: var(--font-body); letter-spacing: 0.04em; text-transform: uppercase; }
    </style>
  </defs>
  <!-- Transport -->
  <g>
    <rect class="region region-transport" x="40" y="20" width="800" height="76"/>
    <text class="label-title" x="60" y="52">Transport</text>
    <text class="label-sub"   x="60" y="72">Thin dispatchers. Parse input, forward to the daemon or call the compiler directly.</text>
    <g transform="translate(460 32)">
      <rect class="chip-bg" x="0"   y="0" width="88" height="22"/><text class="chip" x="44"  y="15" text-anchor="middle">ship-cli</text>
      <rect class="chip-bg" x="96"  y="0" width="72" height="22"/><text class="chip" x="132" y="15" text-anchor="middle">mcp</text>
      <rect class="chip-bg" x="176" y="0" width="72" height="22"/><text class="chip" x="212" y="15" text-anchor="middle">shell</text>
      <rect class="chip-bg" x="256" y="0" width="88" height="22"/><text class="chip" x="300" y="15" text-anchor="middle">studio</text>
    </g>
  </g>
  <!-- Flow: transport -> daemon (HTTP) -->
  <path class="flow flow-http" d="M 230 96 L 230 136" marker-end="url(#arrow)"/>
  <text class="flow-label" x="240" y="122" fill="color-mix(in oklch, var(--info) 70%, var(--foreground))">HTTP · :9315</text>
  <!-- Flow: transport -> compiler (direct) -->
  <path class="flow flow-compile" d="M 720 96 L 720 352" marker-end="url(#arrow-success)"/>
  <text class="flow-label" x="730" y="170" fill="color-mix(in oklch, var(--success) 70%, var(--foreground))">compile · in-process</text>
  <!-- Daemon (contains runtime) -->
  <g>
    <rect class="region region-daemon" x="40" y="136" width="600" height="356"/>
    <text class="label-title" x="60" y="166">shipd <tspan class="label-sub">— Axum HTTP server, stays up across invocations</tspan></text>
    <text class="label-sub"   x="60" y="186">Coordinates multi-agent work. Hosts service actors that share the kernel.</text>
    <g transform="translate(60 198)">
      <rect class="chip-bg" x="0"   y="0" width="96"  height="22"/><text class="chip" x="48"  y="15" text-anchor="middle">supervisor</text>
      <rect class="chip-bg" x="104" y="0" width="108" height="22"/><text class="chip" x="158" y="15" text-anchor="middle">orchestrator</text>
      <rect class="chip-bg" x="220" y="0" width="64"  height="22"/><text class="chip" x="252" y="15" text-anchor="middle">mesh</text>
      <rect class="chip-bg" x="292" y="0" width="120" height="22"/><text class="chip" x="352" y="15" text-anchor="middle">human gateway</text>
      <rect class="chip-bg" x="420" y="0" width="96"  height="22"/><text class="chip" x="468" y="15" text-anchor="middle">PTY bridge</text>
    </g>
  </g>
  <!-- Runtime (contains kernel) -->
  <g>
    <rect class="region region-runtime" x="60" y="238" width="560" height="236"/>
    <text class="label-title" x="80" y="266">Runtime</text>
    <text class="label-sub"   x="80" y="286">Persistent state. Workspaces are the unit of agent work; pipelines drive them forward.</text>
    <g transform="translate(80 298)">
      <rect class="chip-bg" x="0"   y="0" width="96"  height="22"/><text class="chip" x="48"  y="15" text-anchor="middle">workspaces</text>
      <rect class="chip-bg" x="104" y="0" width="72"  height="22"/><text class="chip" x="140" y="15" text-anchor="middle">sessions</text>
      <rect class="chip-bg" x="184" y="0" width="80"  height="22"/><text class="chip" x="224" y="15" text-anchor="middle">pipelines</text>
      <rect class="chip-bg" x="272" y="0" width="96"  height="22"/><text class="chip" x="320" y="15" text-anchor="middle">collections</text>
      <rect class="chip-bg" x="376" y="0" width="96"  height="22"/><text class="chip" x="424" y="15" text-anchor="middle">projections</text>
    </g>
  </g>
  <!-- Kernel -->
  <g>
    <rect class="region region-kernel" x="80" y="336" width="520" height="124"/>
    <text class="label-title" x="100" y="364">Kernel <tspan class="label-sub">— pure event routing</tspan></text>
    <text class="label-sub"   x="100" y="384">Validate → persist → deliver. Namespace-scoped. No filesystem. No database.</text>
    <g transform="translate(100 398)">
      <rect class="chip-bg" x="0"   y="0" width="104" height="22"/><text class="chip" x="52"  y="15" text-anchor="middle">KernelRouter</text>
      <rect class="chip-bg" x="112" y="0" width="84"  height="22"/><text class="chip" x="154" y="15" text-anchor="middle">EventSink</text>
      <rect class="chip-bg" x="204" y="0" width="72"  height="22"/><text class="chip" x="240" y="15" text-anchor="middle">Mailbox</text>
      <rect class="chip-bg" x="284" y="0" width="108" height="22"/><text class="chip" x="338" y="15" text-anchor="middle">ActorConfig</text>
    </g>
    <text class="chip" x="100" y="445" font-size="10.5" fill="var(--muted-foreground)">persist-before-deliver: no actor reacts to an event that isn't durable first.</text>
  </g>
  <!-- Compiler -->
  <g>
    <rect class="region region-compiler" x="670" y="352" width="170" height="140"/>
    <text class="label-title" x="690" y="380">Compiler</text>
    <text class="label-sub"   x="690" y="400" font-size="10.5">Pure transform.</text>
    <text class="label-sub"   x="690" y="414" font-size="10.5">ProjectLibrary →</text>
    <text class="label-sub"   x="690" y="428" font-size="10.5">ResolvedConfig →</text>
    <text class="label-sub"   x="690" y="442" font-size="10.5">CompileOutput.</text>
    <g transform="translate(690 454)">
      <rect class="chip-bg" x="0"   y="0" width="68" height="22"/><text class="chip" x="34"  y="15" text-anchor="middle">vars</text>
      <rect class="chip-bg" x="76"  y="0" width="68" height="22"/><text class="chip" x="110" y="15" text-anchor="middle">providers</text>
    </g>
  </g>
  <!-- Persistence leg -->
  <path class="flow flow-persist" d="M 340 474 L 340 510" marker-end="url(#arrow)"/>
  <g>
    <rect class="region region-storage" x="220" y="510" width="240" height="36"/>
    <text class="chip" x="340" y="533" text-anchor="middle" font-size="12">platform.db · events + projections</text>
  </g>
  <!-- Compile output leg -->
  <path class="flow flow-compile" d="M 755 492 L 755 520" marker-end="url(#arrow-success)"/>
  <g>
    <rect class="region region-storage" x="600" y="520" width="240" height="32"/>
    <text class="chip" x="720" y="541" text-anchor="middle" font-size="11">.claude/ · .agents/ · .cursor/ · .gemini/ · .codex/</text>
  </g>
</svg>
<figcaption style="text-align: center; font-size: 0.85rem; color: var(--muted-foreground); margin-top: 0.75rem;">
Two paths out of transport: HTTP to the daemon, or a direct compile call. The daemon hosts the runtime; the runtime wraps the kernel. Events persist to <code>platform.db</code>; compile writes provider-native files.
</figcaption>
</figure>

<figure style="margin: 2.5rem 0;">
<svg viewBox="0 0 900 660" xmlns="http://www.w3.org/2000/svg" role="img" aria-labelledby="seq-title" style="width: 100%; height: auto; font-family: var(--font-body); color: var(--foreground);">
  <title id="seq-title">Sequence diagram: ship run end-to-end. CLI POSTs a job, daemon emits job.created through the kernel, sink persists, orchestrator reacts off its mailbox, executor spawns the agent in a worktree, and completion events flow back through the same loop.</title>
  <defs>
    <marker id="seq-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor" opacity="0.75"/>
    </marker>
    <marker id="seq-arrow-accent" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--accent)"/>
    </marker>
    <marker id="seq-arrow-info" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--info)"/>
    </marker>
    <marker id="seq-arrow-return" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--muted-foreground)"/>
    </marker>
    <style>
      .p-head     { fill: var(--card); stroke: var(--border); stroke-width: 1; rx: 8; ry: 8; }
      .p-label    { font-family: var(--font-display); font-weight: 600; font-size: 12px; fill: var(--foreground); }
      .p-sublabel { font-size: 10px; fill: var(--muted-foreground); font-family: var(--font-mono, ui-monospace, monospace); }
      .lifeline   { stroke: var(--border); stroke-width: 1; stroke-dasharray: 2 4; }
      .msg        { stroke: var(--foreground); stroke-width: 1.25; fill: none; opacity: 0.8; }
      .msg-event  { stroke: var(--accent); stroke-width: 1.5; fill: none; }
      .msg-http   { stroke: var(--info); stroke-width: 1.5; fill: none; }
      .msg-return { stroke: var(--muted-foreground); stroke-width: 1; stroke-dasharray: 4 3; fill: none; }
      .msg-self   { stroke: var(--foreground); stroke-width: 1; fill: none; opacity: 0.65; }
      .msg-label  { font-size: 11px; fill: var(--foreground); font-family: var(--font-mono, ui-monospace, monospace); }
      .msg-note   { font-size: 10.5px; fill: var(--muted-foreground); font-family: var(--font-body); }
      .phase-bar  { fill: color-mix(in oklch, var(--muted) 45%, var(--card)); stroke: var(--border); stroke-width: 1; rx: 4; ry: 4; }
      .phase-text { font-size: 10.5px; font-weight: 600; fill: var(--muted-foreground); font-family: var(--font-body); letter-spacing: 0.05em; text-transform: uppercase; }
    </style>
  </defs>
  <!-- Participant headers at x: 80, 220, 360, 500, 640, 780 -->
  <g>
    <rect class="p-head" x="30"  y="30" width="100" height="42"/>
    <text class="p-label"    x="80" y="50" text-anchor="middle">CLI</text>
    <text class="p-sublabel" x="80" y="64" text-anchor="middle">ship-cli</text>
    <rect class="p-head" x="170" y="30" width="100" height="42"/>
    <text class="p-label"    x="220" y="50" text-anchor="middle">Daemon</text>
    <text class="p-sublabel" x="220" y="64" text-anchor="middle">runtime_api</text>
    <rect class="p-head" x="310" y="30" width="100" height="42"/>
    <text class="p-label"    x="360" y="50" text-anchor="middle">Kernel</text>
    <text class="p-sublabel" x="360" y="64" text-anchor="middle">KernelRouter</text>
    <rect class="p-head" x="450" y="30" width="100" height="42"/>
    <text class="p-label"    x="500" y="50" text-anchor="middle">Sink</text>
    <text class="p-sublabel" x="500" y="64" text-anchor="middle">platform.db</text>
    <rect class="p-head" x="590" y="30" width="100" height="42"/>
    <text class="p-label"    x="640" y="50" text-anchor="middle">Orchestrator</text>
    <text class="p-sublabel" x="640" y="64" text-anchor="middle">service.job-dispatch</text>
    <rect class="p-head" x="730" y="30" width="140" height="42"/>
    <text class="p-label"    x="800" y="50" text-anchor="middle">Executor + worktree</text>
    <text class="p-sublabel" x="800" y="64" text-anchor="middle">supervisor/executors</text>
  </g>
  <!-- Lifelines -->
  <line class="lifeline" x1="80"  y1="72" x2="80"  y2="635"/>
  <line class="lifeline" x1="220" y1="72" x2="220" y2="635"/>
  <line class="lifeline" x1="360" y1="72" x2="360" y2="635"/>
  <line class="lifeline" x1="500" y1="72" x2="500" y2="635"/>
  <line class="lifeline" x1="640" y1="72" x2="640" y2="635"/>
  <line class="lifeline" x1="800" y1="72" x2="800" y2="635"/>
  <!-- Phase 1: synchronous POST + persist + deliver -->
  <rect class="phase-bar" x="20" y="92" width="860" height="20"/>
  <text class="phase-text" x="32" y="107">1 · synchronous: POST → event durable → mailbox filled</text>
  <!-- 1. CLI → Daemon: POST /api/jobs -->
  <line class="msg msg-http" x1="80" y1="135" x2="215" y2="135" marker-end="url(#seq-arrow-info)"/>
  <text class="msg-label" x="150" y="128" text-anchor="middle" fill="color-mix(in oklch, var(--info) 70%, var(--foreground))">POST /api/jobs</text>
  <text class="msg-note"  x="150" y="148" text-anchor="middle">slug · branch · agent · pipeline · spec_content</text>
  <!-- 2. Daemon self: ensure_pipeline enrich -->
  <path class="msg msg-self" d="M 220 168 L 260 168 L 260 184 L 225 184" marker-end="url(#seq-arrow)"/>
  <text class="msg-label" x="264" y="180">ensure_pipeline(payload)</text>
  <!-- 3. Daemon → Kernel: persist_and_route / emit(job.created) -->
  <line class="msg msg-event" x1="220" y1="212" x2="355" y2="212" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="287" y="206" text-anchor="middle" fill="color-mix(in oklch, var(--accent) 70%, var(--foreground))">emit(job.created)</text>
  <!-- 4. Kernel → Sink: persist -->
  <line class="msg msg-event" x1="360" y1="244" x2="495" y2="244" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="427" y="238" text-anchor="middle">sink.persist(&amp;event)</text>
  <text class="msg-note"  x="427" y="258" text-anchor="middle">INSERT INTO events (synchronous)</text>
  <!-- 5. Kernel → Orchestrator: try_send (mailbox) -->
  <line class="msg msg-event" x1="360" y1="286" x2="635" y2="286" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="497" y="280" text-anchor="middle">mailbox.try_send(event)</text>
  <text class="msg-note"  x="497" y="300" text-anchor="middle">subscribe_namespaces: ["job.", "step."]</text>
  <!-- 6. Daemon → CLI: 200 OK -->
  <line class="msg msg-return" x1="215" y1="324" x2="85" y2="324" marker-end="url(#seq-arrow-return)"/>
  <text class="msg-label" x="150" y="318" text-anchor="middle" fill="var(--muted-foreground)">200 { job_id }</text>
  <text class="msg-note"  x="150" y="340" text-anchor="middle">CLI exits. Orchestrator takes over.</text>
  <!-- Phase 2: async reaction -->
  <rect class="phase-bar" x="20" y="356" width="860" height="20"/>
  <text class="phase-text" x="32" y="371">2 · asynchronous: orchestrator processes the event</text>
  <!-- 7. Orchestrator mailbox.recv → handle_job_event → handle_job_created -->
  <path class="msg msg-self" d="M 640 395 L 685 395 L 685 411 L 645 411" marker-end="url(#seq-arrow)"/>
  <text class="msg-label" x="689" y="408">mailbox.recv() → handle_job_created</text>
  <!-- 8. Orchestrator → Executor: isolation.prepare + executor.spawn -->
  <line class="msg" x1="640" y1="442" x2="795" y2="442" marker-end="url(#seq-arrow)"/>
  <text class="msg-label" x="718" y="436" text-anchor="middle">isolation.prepare → executor.spawn</text>
  <text class="msg-note"  x="718" y="456" text-anchor="middle">git worktree add job/&lt;slug&gt; → spawn process</text>
  <!-- 9. Executor self: pipeline specs + dispatch prompt -->
  <path class="msg msg-self" d="M 800 475 L 855 475 L 855 491 L 805 491" marker-end="url(#seq-arrow)"/>
  <text class="msg-label" x="800" y="508" text-anchor="middle" font-size="10.5">write .artifacts/pipeline/* · dispatch prompt</text>
  <!-- Phase 3: completion feedback loop -->
  <rect class="phase-bar" x="20" y="524" width="860" height="20"/>
  <text class="phase-text" x="32" y="539">3 · feedback: process exit → step_completed → orchestrator advances</text>
  <!-- 10. Executor → Kernel: emit(job.step_completed) -->
  <line class="msg msg-event" x1="800" y1="563" x2="365" y2="563" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="582" y="557" text-anchor="middle" fill="color-mix(in oklch, var(--accent) 70%, var(--foreground))">emit(job.step_completed)</text>
  <!-- 11. Kernel → Sink -->
  <line class="msg msg-event" x1="360" y1="590" x2="495" y2="590" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="427" y="584" text-anchor="middle">persist</text>
  <!-- 12. Kernel → Orchestrator again -->
  <line class="msg msg-event" x1="360" y1="618" x2="635" y2="618" marker-end="url(#seq-arrow-accent)"/>
  <text class="msg-label" x="497" y="612" text-anchor="middle">mailbox.try_send → advance_pipeline</text>
</svg>
<figcaption style="text-align: center; font-size: 0.85rem; color: var(--muted-foreground); margin-top: 0.75rem;">
<code>ship run</code> end-to-end. Phase 1 is synchronous — the POST doesn't return until the event is on disk and in the orchestrator's mailbox. Phase 2 is asynchronous — the orchestrator consumes from its mailbox and dispatches. Phase 3 closes the loop — the executor emits completion through the same kernel path, triggering <code>advance_pipeline</code> for the next step.
</figcaption>
</figure>

## Code shape

**Crates** (`crates/core/`):

| Crate | Role |
|---|---|
| `ship-types` | Shared type primitives. Zero internal deps. |
| `ship-jobs` | Pipeline and job types. YAML normalization. |
| `ship-kernel` | Pure event routing. Actor model. |
| `runtime` | State management. Persistence, services, projections, registry. |
| `compiler` | Pure transformation. Native + WASM. |
| `cli-framework` | Shared CLI metadata and lifecycle. |
| `mcp-framework` | MCP protocol transport abstraction. |

**Apps** (`apps/`):

| App | Role |
|---|---|
| `shipd` | The daemon. Hosts runtime, services, supervisor. |
| `ship-cli` | CLI binary. Clap commands. |
| `mcp` | MCP server. Stdio and HTTP transports. |
| `shell` | Daemon-served terminal UI. |
| `web` | Ship Studio. |
| `docs` | This site (Astro + Starlight). |

**Packages** (`packages/`):

| Package | Role |
|---|---|
| `@ship/compiler` | WASM-bindgen npm wrapper around the compiler crate. |
| `@ship/set` | Web components + tokens. The `ship-elements` bundle. |
| `@ship/primitives` | Shared UI primitives. |
| `@ship/sdk` | TypeScript SDK for talking to the daemon. |
| `@ship/assets` | Shared static assets. |

## Flows

**Compile.** `ship use` → compiler reads `.ship/` → writes `.claude/`, `.agents/`, `.cursor/`, `.gemini/`, `.codex/`. In-process, no daemon. Deterministic — same input, same bytes. Output paths are gitignored by default.

**Agent session.** Transport hits `shipd` over HTTP → runtime emits an event → kernel validates → sink persists to `platform.db` → subscribed actors (session projection, mesh, supervisors) receive it → read models update. Persist-before-deliver: nothing reacts to an event that isn't durable first.

**Pipeline.** Orchestrator finds an unblocked step → dispatches a job event → supervisor allocates a worktree and spawns the agent → agent emits progress events → orchestrator sees completion → finds the next unblocked step. Runs until the pipeline drains or hits a `human:` gate.

## Design invariants

**Determinism before cleverness.** The compiler rejects anything non-reproducible. No `Instant::now()`, no random IDs, no insertion-order-dependent iteration.

**Events are the record. Projections are a cache.** You may not mutate an event. You may drop and rebuild any projection table. A feature that needs a new read model adds a projection, not a new mutable table.

**The kernel owns no storage.** Storage decisions live in `runtime`, injected through `EventSink`.

**Transport forwards, daemon decides.** One hop: transport → compiler, or transport → daemon.

## Deeper reading

- [The Ship Stack](/reference/ship-stack/) — product-layer view.
- [Runtime internals](/reference/runtime/) — shipd, services, persistence.
- [Kernel](/reference/architecture/kernel/) — routing contract, limitations, future work.
- [Provider Outputs](/reference/compiler/providers/) — what the compiler writes per provider.
- [Daemon authentication model](/reference/architecture/auth-model/) — tokens and session scopes.

## Services inside the daemon

`shipd` hosts several service actors. Each has a single responsibility and communicates via the kernel.

**Orchestrator** (`runtime/services/orchestrator/`). Advances pipelines step-by-step. When a job completes, `advance_pipeline` finds newly unblocked steps and dispatches them. Handles gate retries (up to `max_retries`), step monitoring (watches process exit), `run` shell commands, and `human` pauses. `replay.rs` re-dispatches incomplete pipelines on daemon startup.

**Mesh** (`runtime/services/mesh.rs`). Inter-agent message routing via `SharedMeshRegistry`. Agents publish to namespaces; subscribers filter by capability scope. Namespaces are enforced at the `ActorStore` write boundary — an actor cannot emit events into a namespace it lacks write permission for.

**Supervisor** (`apps/shipd/src/supervisor/`). Process dispatch, worktree setup, terminal launch, job lifecycle. Spawns agent processes in git worktrees, wires stdio to the kernel, monitors exit codes, and surfaces results to the orchestrator.

**Human gateway** (`runtime/services/human_gateway.rs`). Pauses pipelines at `human:` steps until a human acknowledges. The web UI renders the pending prompt; acknowledgment advances the pipeline.

**PTY bridge** (`apps/shipd/pty_handler.rs`). WebSocket-to-tmux bridge. Studio opens a terminal session; the daemon proxies bytes between the browser WebSocket and a tmux pane.

## Runtime module map

Top-level modules in `crates/core/runtime/src/lib.rs`:

| Module | Owns |
|---|---|
| `actor` | Actor trait, supervisor, restart policies. |
| `agents` | Agent profiles, export/import, permissions, rules, skills, skill paths, skill vars. |
| `artifacts` | Artifact metadata, session artifact persistence. |
| `collections` | Collection types and persistence. |
| `config` | `.ship/` parsing — catalog, hooks, security, providers. |
| `data` | Shared data types. |
| `db` | SQLite connection pool, KV store, workspace/session/artifact persistence, schema migrations. |
| `events` | Facade over `ship-kernel` + `ship-types` event types. |
| `hooks` | `RuntimeHooks` trait for lifecycle events. |
| `log` | File-based project action log. |
| `persistence` | `EventStore` trait, `SqliteEventStore`, snapshots, cursors. |
| `project` | Project init, directory resolution, worktree-aware path lookup. |
| `projections` | Read models rebuilt from events (jobs, sessions, workspaces). |
| `registry` | Package install, fetch, resolve, verify, tracking. |
| `services` | Orchestrator, mesh, human gateway, provider adapters, dispatch ports. |
| `workspace` | Workspace lifecycle, sessions, trash, artifact events, reconciliation. |

## Platform database

Path: `~/.ship/platform.db`. Relational state. Opened via `sqlx` with compile-time query checks. WAL journal mode, foreign keys enabled.

| Table family | Concern |
|---|---|
| `workspaces`, `workspace_events` | Workspace records + event projection. |
| `sessions`, `session_events` | Session records + event projection. |
| `jobs`, `actor_events` | Job and actor lifecycle events. |
| `events` | Shared event log. All actor events. `actor_id` column enforces logical isolation. |
| `file_claims` | Per-file exclusive write locks across parallel agents. |
| `artifact_registry` | Artifact metadata store. |
| `kv`, `managed_state` | Generic key-value + state blobs. |
| `adrs`, `branch_context` | Architecture decision records, branch-scoped docs. |

Tests get automatic isolation: `get_global_dir()` detects test binaries and returns a per-thread temp directory.

## Actor protocol

Every actor has: a `namespace`, permitted write/read/subscribe scopes, and a mailbox. Actors are a logical boundary — `actor_id` column filters reads; `ActorStore` rejects writes outside the actor's `write_namespaces`. No per-actor SQLite files.

| Actor type | Has UI | Has MCP | Examples |
|---|---|---|---|
| Agent | no | yes (client) | Code agent, design agent |
| App | yes | yes (server) | Ship Studio |
| Service | no | no | Orchestrator, mesh, supervisor |

Routing contract: validate → persist → deliver. Direct messages short-circuit namespace fanout via `target_actor_id`. Broadcasts match by prefix on `event_type` against each subscriber's `subscribe_namespaces`.

## Workspaces

A workspace is an isolated unit of agent work, keyed by git branch today. The supervisor allocates a git worktree per workspace so parallel agents don't collide on the same files. File claims in the platform DB give per-file exclusive locks across agents sharing a workspace.

Sessions track agent time within a workspace — start, progress, end with summary. One active session per workspace.

## Event flow internals

`KernelRouter::route(event)`:

1. Run each registered validator. Fail = error, event discarded.
2. `sink.persist(&event)` — synchronous. Durable before any delivery.
3. Deliver:
   - If `event.target_actor_id` set → direct to that mailbox only.
   - Otherwise broadcast: deliver to every actor whose `subscribe_namespaces` contains a prefix of `event.event_type`.

Delivery uses `try_send` — mailbox full = silent drop. See [Kernel](/reference/architecture/kernel/) for the full contract, current limitations, and future work.
