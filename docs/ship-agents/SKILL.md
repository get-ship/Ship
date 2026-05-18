---
name: ship-agents
stable-id: ship-agents
description: Use when creating, configuring, or managing Ship agents — profiles, skills, permissions, providers, rules.
tags: [ship, agents, guide]
authors: [ship]
---

# Ship Agents

Use this skill when configuring or understanding Ship agents. Agents are declarative profiles that compile to provider-specific config files.

See `references/docs/` for the full reference:

- **Agents** -- What agents are and the agent.jsonc schema overview.
- **Agent Configuration** -- Full profile reference: skills, MCP servers, permissions, rules, plugins, provider settings.
- **Permissions** -- The four permission presets and per-agent overrides.

## Launching an agent

`ship use <agent>` activates and compiles the profile. Add a launch flag to dispatch a session in the same step:

```
ship use <agent>                              # config only (compile + activate)
ship use <agent> --launch                     # interactive session
ship use <agent> --auto                       # autonomous (headless)
ship use <agent> --resume                     # continue last session here
ship use <agent> --launch --prompt "fix X"    # seed the first turn
ship use <agent> --auto --prompt "fix X"      # one-shot autonomous
ship use <agent> --launch --spec ./job.md     # spec-driven launch
ship use <agent> --launch --harness codex --model sonnet
                                              # per-launch overrides
```

`--auto`, `--prompt`, `--spec`, and `--resume` each imply `--launch`. `--prompt` and `--spec` are mutually exclusive. The workspace label is derived from the directory basename; tail with `ship procs --slug <slug>`.
