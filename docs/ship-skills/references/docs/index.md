---
group: Smart Skills
title: Smart Skills
description: An extension of the open agentskills.io format with runtime-layer features and stricter publishing.
order: 1
---

A **skill** is a markdown file that tells an agent how to do something — naming conventions, code patterns, review checklists, any kind of consistent guidance you want an agent to follow. Skills follow the open [agentskills.io](https://agentskills.io) format.

**Smart Skills** is Ship's extension of that format. We add features at the runtime layer and tighten a few rules for publishing.

## At the runtime layer

<sui-card padding="md">
<strong slot="header">Typed variables</strong>
Declare variables in the skill. The runtime resolves them per user, per project, or per machine at compile time. Same skill, different output for whoever's using it.
</sui-card>

<sui-card padding="md">
<strong slot="header">Runtime-injected context</strong>
When the runtime renders a skill, it can supply context from the system or from the surrounding pack — values the skill author doesn't have to ask for explicitly.
</sui-card>

The variables story is the headline. See [Variables](./variables/) for what you can do with them.

## At publish time

To publish a skill on the Ship Store, the format is a little stricter than the open spec:

- A stable id is required, so installs are deterministic.
- Versions are supported.
- Frontmatter is validated against a schema before the skill is accepted.

You don't need any of this for skills that live in your own project. The strictness applies to packages that travel.
