---
title: "Your First Project"
sidebar:
  label: "Your First Project"
  order: 3
---
## Initialize

Navigate to your project and run:

```bash
cd your-project
ship init
```

This creates a `.ship/` directory with a project manifest, a `dev` agent, and compiles immediately. Your project is ready to use with your AI tools.

## What just happened

`ship init` did three things:

1. **Scaffolded `.ship/`** — project manifest, default agent (`dev.jsonc`), and directory structure
2. **Compiled** — generated provider-native config (CLAUDE.md, .mcp.json, etc.) from the dev agent
3. **Gitignored** — compiled output is added to `.gitignore` automatically

## Open Studio

For a visual interface:

```bash
ship studio --open
```

Studio opens in your browser. From there you can:

- Browse and install skills from the registry
- Configure agent profiles
- Set skill variables (preferences that personalize behavior)
- See everything update in real time

## Edit your agent

Your agent lives at `.ship/agents/dev.jsonc`. Open it and add skills, change permissions, or add more providers:

```jsonc
{
  "agent": {
    "id": "dev",
    "name": "Dev",
    "providers": ["claude", "gemini"]
  },
  "skills": {
    "refs": ["tdd", "code-review"]
  },
  "permissions": {
    "preset": "ship-autonomous"
  }
}
```

After editing, recompile:

```bash
ship compile
```

## Add more agents

Copy your dev agent and customize:

```bash
cp .ship/agents/dev.jsonc .ship/agents/backend.jsonc
```

Edit `backend.jsonc` — change the `id` to `"backend"`, update the name, skills, and permissions. Then activate:

```bash
ship use backend
```

## Install packages

Add skills from the registry:

```bash
ship add github.com/owner/skill-pack
```

This updates `ship.jsonc`, resolves dependencies, writes `ship.lock`, and recompiles.

## Next steps

- [Schema Reference](/reference/schema/) — full field reference for all config files
- [Smart Skills](/reference/smart-skills/) — understand how skills and variables work
- [Registry](/reference/registry/) — browse and publish packages
