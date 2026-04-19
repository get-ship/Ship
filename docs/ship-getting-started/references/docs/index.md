---
group: Getting Started
title: Introduction
section: guide
order: 1
---

# Introduction

Ship is a package manager and runtime for AI agents. Install skills and agents from the registry, customize them for your projects, and let Ship handle the rest.

## What Ship does

- **Skills** — install, publish, and personalize agent capabilities. Smart skills adapt to each user and project through typed variables.
- **Agents** — compose skills, tools, permissions, and rules into profiles that work across Claude Code, Cursor, Gemini, Codex, and OpenCode.
- **Studio** — a visual IDE connected to your local projects. Browse the registry, configure agents, edit skills, set preferences.
- **Registry** — share skills and agents publicly at getship.dev, or keep them private to your team.

## Getting started

Get started in three commands:

```bash
curl -fsSL https://getship.dev/install | sh
cd your-project
ship init
```

`ship init` scaffolds `.ship/`, creates a `dev` agent, and compiles provider-native config files automatically. Your project is ready to use with Claude Code, Gemini, Codex, or OpenCode immediately.

For a visual interface, run `ship studio --open` to browse the registry, configure agents, and edit skills in the browser.

Continue to [Installation](./installation).
