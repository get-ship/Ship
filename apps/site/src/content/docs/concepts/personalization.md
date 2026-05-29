---
title: Personalization
description: Typed variables and themes — the same skill or app produces different output per user, per space, per team.
section: Concepts
order: 5
kicker: The mental model
tags: [concepts, variables, themes]
related: [what-is-ship]
---

The same skill, agent, or app can produce different output for different users and spaces — not by forking code, but through typed variables and themes.

## Variables

Every skill can declare typed variables — strings, enums, lists, structured objects — and each has a scope:

- **Global** — set once for your machine.
- **Local** — set for a space on this machine, not synced.
- **Space** — committed to the space's `.ship/`, travels with it across machines.

Values merge in that order: the space value wins, the local override fills in, the global default is the floor. So one `code-review` skill can know your house style globally, your team's conventions per space, and an in-progress override locally — without three forks.

## Themes

Themes apply to the app and everything that renders inside it — views, dashboards, artifact viewers. There are seven built-in themes; switch at runtime, or author your own. Custom apps you build with Ship inherit the active theme automatically.

Together, variables and themes mean the runtime delivers an experience tuned to who's using it and what they're working on — without anyone shipping a custom build.
