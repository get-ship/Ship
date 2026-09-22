---
title: Local-first
description: The runtime, your data, and the app all run on your machine. Cloud is opt-in.
section: Concepts
order: 3
kicker: The mental model
tags: [concepts, local-first]
related: [what-is-ship]
---

Local-first is the default, not a feature. The runtime, the database, and the app all run on your machine. Your spaces, agents, skills, and data live where your code lives.

This isn't "self-host if you want" — it's the architecture. There's no Ship server you depend on: the CLI, the daemon, and the app are all local. The coding agents you already use — Claude Code, Cursor, Gemini, Codex, OpenCode — talk to Ship over MCP on `localhost`.

## Cloud is optional

The Store and share links are adjacent services you can opt into. Sign in if you want to publish or share a package; skip it entirely and Ship works the same.

## What this gets you

- **Offline by default** — no round-trips for normal work.
- **Data ownership** — your spaces, collections, and artifacts stay on your machine.
- **Privacy** — what you do in Ship doesn't leave your laptop unless you publish or sync.
