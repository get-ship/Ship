---
title: Red Phase
description: Writing the failing test that defines the next behavior.
section: phases
order: 1
---

# Red Phase

You are writing a failing test. This test defines a behavior that does not exist yet.

## Rules

- Write ONE test at a time
- Run it immediately — it MUST fail
- If it passes, the test is wrong or the behavior already exists. Delete it and try again.
- Edit test files only. Do NOT touch implementation files.
- You may read any file to understand existing behavior.

## What makes a good failing test

- Tests BEHAVIOR, not implementation
- Uses realistic inputs
- Has a descriptive name that explains the expected behavior
- Fails because the behavior doesn't exist, not because of a typo or import error

## Process

1. Check `${EVIDENCE_DIR:-$(git rev-parse --show-toplevel)/.artifacts}/spec-from-tests.md` (or its `SHIP_MISSION_SCRATCH` equivalent under a mission) for the next planned test if it exists
## Phase lock

To constrain an agent to Red only:

```
ship vars set tdd phase red
```

The agent will then only see Red phase instructions when `ship use` compiles the skill.
