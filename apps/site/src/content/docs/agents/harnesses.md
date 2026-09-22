---
title: Harnesses
description: Ship wraps the CLI agents you already use — Claude Code, Cursor, Gemini, Codex, OpenCode — and keeps their config in sync from one source.
section: Agents
order: 4
kicker: Agents
tags: [agents, harnesses, mcp]
related: [agents]
---

Ship wraps the CLI agent harnesses you already use. You don't switch agent platforms when you adopt Ship — you keep whichever one you have, and Ship sits above it.

The job it does for you is **configuration management.** Every harness has its own format for context files, MCP servers, permission rules, model preferences, sandbox settings. Without Ship that's parallel configs to hand-maintain every time something changes. With Ship you author one declarative profile and the right files appear in the right places, every time you compile.

## Why it matters

- **One source of truth.** Skills, MCP servers, and permissions live once in `.ship/`. The harness files (`CLAUDE.md`, `.cursor/rules`, and the rest) are build output, not authored by hand.
- **Switch harnesses without rewriting.** Point the same profile at a different harness; the config regenerates and you keep going.
- **Mix harnesses in one space.** Different agents can target different harnesses — no divergent config to maintain.

## Shared vs pass-through

Some config is universal — skills, MCP servers, basic permissions — and applies everywhere. Other settings only mean something to a specific harness — sandbox modes, model variants, harness-specific rule formats — and stay scoped to the one that understands them. Nothing leaks across harnesses; nothing gets dropped.

## Supported today

Claude Code, Cursor, Gemini CLI, Codex, OpenCode. The list grows as new harnesses become viable targets.
