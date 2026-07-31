---
title: FAQ
description: The questions everyone asks before they install — short answers, pointing to the detail.
section: FAQ
order: 1
kicker: FAQ
tags: [faq]
status: ready
---

## Is Ship free?

Ship runs on your machine and is free to use. An optional paid cloud tier — sync and teams — is coming; solo, fully local, costs nothing.

## What happens to my data?

It stays on your machine. The runtime, your databases, and your data all live locally — nothing leaves unless you publish or sync. → [Local-first](/docs/local-first/)

## Does it work with Claude Code, Cursor, Gemini, Codex, OpenCode?

Yes. You keep the harness you already use; Ship sits above it and keeps its config in sync. → [Harnesses](/docs/harnesses/)

## What runs my UI?

Sail, Ship's view framework — declarative templates compiled ahead of time, server-rendered, hydrated client-side. → [What is Sail](/docs/what-is-sui/)

## Can agents see my whole machine?

Only what their permission preset allows. Every agent runs under `ship-readonly`, `ship-standard`, `ship-autonomous`, or `ship-elevated`, and you can tighten further. → [Permissions](/docs/permissions/) · [Security](/docs/security/)

## Can my team share an agent or a space?

Sharing rides on the optional cloud tier (coming). Solo and offline, everything works the same minus sharing.

## What platforms?

macOS, Windows, and Linux. → [Get Ship](/docs/quick-start/)
