---
name: ship-schema-reference
stable-id: ship-schema-reference
description: Use when the user asks about Ship configuration format, available fields, valid values, or how to structure their .ship/ directory. Covers ship.jsonc manifest, agent profile JSONC, permissions.jsonc, mcp.jsonc, skill frontmatter, vars.json, pipeline YAML, and ship.lock schemas.
tags: [reference, schema, configuration, documentation]
authors: [ship]
---

# Ship Configuration Schema Reference

All Ship config files use JSONC (JSON with comments). JSON Schemas are available at `https://raw.githubusercontent.com/madvib/ship/main/schemas/` for editor autocompletion.

For detailed field-level documentation, read the reference pages in `references/docs/`:

- [Overview](references/docs/index.md) — directory layout and schema URLs
- [Project Manifest](references/docs/manifest.md) — `.ship/ship.jsonc`
- [Agent Profile](references/docs/agent.md) — `.ship/agents/<id>.jsonc`
- [Permissions](references/docs/permissions.md) — `.ship/permissions.jsonc`
- [MCP Servers](references/docs/mcp.md) — `.ship/mcp.jsonc`
- [Skill Frontmatter](references/docs/skill.md) — `SKILL.md` YAML frontmatter
- [Skill Variables](references/docs/vars.md) — `assets/vars.json`
- [Pipelines](references/docs/pipeline.md) — `.ship/pipelines/<name>.yaml`
- [Lockfile](references/docs/lockfile.md) — `ship.lock`
