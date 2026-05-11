---
group: Concepts
title: A harness for harnesses
description: Ship wraps the CLI agents you already use and keeps their configuration in sync.
order: 2
---

Ship wraps the CLI agent harnesses you already use. You don't switch agent platforms when you adopt Ship — you keep whichever one you have, and Ship sits above it.

The job Ship does for you is **configuration management**. Every provider has its own format for context files, MCP server lists, permission rules, model preferences, sandbox settings. Without Ship, that means hand-maintaining parallel configs every time something changes. With Ship, you author one declarative profile and the right files appear in the right places — automatically, every time you compile.

## Why this matters

- **One source of truth.** Skills, MCP servers, permissions live once in `.ship/`. The provider files (`CLAUDE.md`, `.cursor/rules`, and the rest) are build output, not authored by hand.
- **Switch providers without rewriting.** Point the same profile at a different provider; the configuration is regenerated and you keep going.
- **Mix providers across a project.** Different agents in the same project can target different providers — no divergent config to maintain.

## Pass-through settings

Some configuration is universal — skills, MCP servers, basic permissions. Other settings only mean something to a specific provider: sandbox modes, model variants, provider-specific rule formats. Ship distinguishes the two. Shared configuration applies everywhere; pass-through settings stay scoped to the provider that understands them. Nothing leaks across providers; nothing gets dropped.

## Currently supported

Claude Code, Cursor, Gemini CLI, Codex, OpenCode. The list grows as new agent harnesses become viable targets.
