---
name: ship-schema-reference
stable-id: ship-schema-reference
description: Use when the user asks about Ship configuration — file schemas, KV settings, env vars, defaults resolution chain. Covers ship.jsonc manifest, agent profile JSONC, permissions.jsonc, mcp.jsonc, skill frontmatter, vars.json, pipeline JSONC, ship.lock schemas, the `daemon_settings` KV store, every `SHIP_*` env var, and how the four configuration tiers resolve.
tags: [reference, schema, configuration, documentation, settings, env-vars]
authors: [ship]
---

# Ship Configuration Reference

Ship configuration spans four tiers: per-launch env vars / CLI flags, the project file (`.ship/ship.jsonc` and friends), the user KV store (`daemon_settings`), and compile-time defaults. The pages below document each surface and the resolution chain that picks a value at runtime.

All Ship config files use JSONC (JSON with comments). JSON Schemas are available at `https://raw.githubusercontent.com/madvib/ship/main/schemas/` for editor autocompletion.

For detailed field-level documentation, read the reference pages in `references/docs/`:

- [Overview](references/docs/index.md) — directory layout and schema URLs
- [Project Manifest](references/docs/manifest.md) — `.ship/ship.jsonc`
- [Agent Profile](references/docs/agent.md) — `.ship/agents/<id>.jsonc`
- [Permissions](references/docs/permissions.md) — `.ship/permissions.jsonc`
- [MCP Servers](references/docs/mcp.md) — `.ship/mcp.jsonc`
- [Skill Frontmatter](references/docs/skill.md) — `SKILL.md` YAML frontmatter
- [Skill Variables](references/docs/vars.md) — `assets/vars.json`
- [Pipelines](references/docs/pipeline.md) — `.ship/pipelines/<name>.jsonc`
- [Lockfile](references/docs/lockfile.md) — `ship.lock`
- [Environment Variables](references/docs/env-vars.md) — every `SHIP_*` env var Ship reads
- [KV Settings](references/docs/settings.md) — daemon-managed preferences in `daemon_settings`, plus the four-tier resolution chain
