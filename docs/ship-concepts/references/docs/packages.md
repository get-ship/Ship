---
group: Concepts
title: Packages
description: Skills, agents, and pipelines distribute as Git repos. Pinned by commit and content hash. Becoming the Ship Store.
order: 6
---

Skills, agents, and other authorable pieces are packaged as Git repositories with a `.ship/` directory inside. Anyone can install someone else's package by pointing Ship at the repo URL.

```bash
ship add github.com/owner/skill-pack
```

A `ship.lock` file pins each dependency to a specific commit and a content hash. Installs are reproducible — the same `ship install --frozen` produces the same configuration on every machine.

## Where this is going

The story is becoming the **Ship Store** — a curated home for skills, agents, pipelines, themes, and views. The package system works today; the Store experience around it is what's being built next.
