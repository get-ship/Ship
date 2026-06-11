---
group: Concepts
title: Local-first
description: The runtime, your data, and the app all run on your machine. Cloud is opt-in.
order: 3
---

Local-first is the default, not a feature. The runtime, the database, and the app all run on your machine. Your projects, agents, skills, and data live where your code lives.

This isn't "self-host if you want" — it's the architecture. There's no Ship server you depend on. The runtime, the daemon, and the browser app are all local; the agent runs as a supervised actor in your active space, calling connectors over the local MCP socket.

## Cloud is optional

The Ship Store and share links are adjacent services you can opt into. Pair with `ship login` if you want to publish or share a package. Skip it entirely and Ship works the same.

## What this gets you

- **Offline by default.** No round-trips for normal work.
- **Data ownership.** Your projects, collections, and artifacts stay on your machine.
- **Privacy.** What you do in Ship doesn't leave your laptop unless you publish or sync.
