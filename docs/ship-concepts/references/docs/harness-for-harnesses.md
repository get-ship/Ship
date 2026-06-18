---
group: Concepts
title: Ship runs your agents
description: Ship's first-party agent reads your project description and scaffolds a real Ship app — no third-party harness in the loop.
order: 2
---

A **Ship agent** is a first-party actor that runs inside the Ship
runtime. It reads your project description, scaffolds a real Ship
view, queries your collections, and writes artifacts. It is not a
wrapper over a third-party CLI; it is the agent.

You author an agent profile once in `.ship/agents/<name>.jsonc` —
the skills it knows, the tools it can reach, the gates it has to
clear, the model it talks to, the space it lives in. Ship compiles
that profile into a runtime actor the kernel supervises alongside
every other actor in your project.

## Why this matters

- **One source of truth.** Skills, MCP servers, permissions, model,
  rules, and gates live once in `.ship/agents/<name>.jsonc`. The
  runtime reads from there — no parallel config to maintain.
- **Auditable by default.** Every prompt, tool call, decision, and
  token lands on the event bus. Scrub back, replay, ship a fix.
- **Scoped, not isolated.** An agent's reach is its actor scope
  (`cwd`, `space_id`, permissions, transport, MCP tool gate). One
  agent per space, scoped to that space's data and tools; multiple
  agents can run in the same project, each with its own scope.
- **Same actor, every view.** The same agent profile powers the chat
  pane, the editor's inline agent, headless runs, and the CLI.
  Skills compose; the runtime doesn't fork.

## Pass-through settings

Some configuration is universal — skills, MCP servers, basic
permissions, the rules of a given space. Other settings only mean
something to a specific runtime layer: provider-specific model
variants, sandbox modes, an external integration. Ship distinguishes
the two. Shared configuration applies everywhere; pass-through
settings stay scoped to the layer that understands them. Nothing
leaks across layers; nothing gets dropped.

## Reach

When you need an external integration — an API token, a third-party
service, a model provider that doesn't ship with Ship — the agent
calls it through a connector. Connectors are declared in
`.ship/connectors/`, gated by the actor's MCP tool gate, and audited
on the event bus like every other tool call.

The list of available integrations lives in
`/docs/configuration/connectors` and grows as the project grows.
Ship's first-party agent is the host; connectors are how it reaches
out.
