---
title: Skills
description: A skill is a markdown instruction set an agent follows — with typed variables so one skill produces tailored output per user and space.
section: Agents
order: 3
kicker: Agents
tags: [agents, skills]
related: [agents, personalization]
---

A **skill** is a markdown file that tells an agent how to do something — naming conventions, code patterns, review checklists, any consistent guidance you want an agent to follow. Skills use the open [agentskills.io](https://agentskills.io) format, so they're portable.

An agent picks up skills by reference:

```jsonc
{ "skills": { "refs": ["tdd", "code-review"] } }
```

## Smart Skills

Ship extends the open format at the runtime layer — what it calls **Smart Skills**:

- **Typed variables.** Declare variables in the skill; the runtime resolves them per user, per space, or per machine at compile time. Same skill, tailored output for whoever's using it. (See [Personalization](/docs/personalization/).)
- **Runtime-injected context.** When the runtime renders a skill it can supply context from the system or the surrounding pack — values the author doesn't have to ask for explicitly.

You don't need any of this for skills that live in your own space. The extra strictness — a stable id, versions, schema-validated frontmatter — only applies to skills published as packages that travel.
