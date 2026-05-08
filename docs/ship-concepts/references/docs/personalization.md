---
group: Concepts
title: Personalization
description: Typed variables and themes. The same skill or app produces different output per user, per project, per team.
order: 5
---

The same skill, agent, or app can produce different output for different users and different projects. That isn't done by forking code — it's done by typed variables and themes.

## Variables

Every skill can declare typed variables — strings, enums, lists, structured objects. Each variable has a scope:

- **Global.** Set once for your machine.
- **Local.** Set for a project on this machine.
- **Project.** Committed to `.ship/`, applies to a project across machines.

Values merge in that order. The project value wins; the local override fills in; the global default is the floor. The same `code-review` skill can know your house style globally, your team's conventions per project, and your in-progress override locally — without three forks.

## Themes

Themes apply to the app and to everything that renders inside it: views, dashboards, artifact viewers. Six built-in themes; switch at runtime; author your own. Custom apps you build with Ship inherit the active theme automatically.

Together, variables and themes mean the runtime delivers an experience tuned to who's using it and what they're working on — without anyone shipping a custom build.
