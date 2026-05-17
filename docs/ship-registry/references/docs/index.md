---
group: Registry
title: Registry overview
order: 1
description: How Ship packages are distributed — Git-based today, the Ship Store curating the same surface later.
---

Ship packages are Git repositories with a `.ship/` directory at the root. You install one by pointing `ship add` at a Git URL; you publish one by pushing the repo. The resolver pins every dependency in `ship.lock` to an exact commit and SHA-256 tree hash, so a `ship install` is reproducible.

The curated **[Ship Store](#ship-store)** wraps the same surface with browse, search, and a quality bar. It's in development; the underlying Git-backed registry works today.

## What's in this section

- **[Installing Packages](./installing/)** — `ship add` syntax, version constraints, lockfile reproducibility, and how transitive dependencies resolve.
- **[Publishing Packages](./publishing/)** — package shape, `manifest.json`, semver versioning, and how to publish a package others can install.

## Package shape

Every Ship package is a Git repository with this minimum layout:

```
your-package/
├── .ship/
│   ├── skills/      # Optional: shared skills
│   ├── agents/      # Optional: shared agent profiles
│   ├── pipelines/   # Optional: shared pipelines
│   └── manifest.json  # Required: declares what the package exports
└── README.md
```

`manifest.json` declares the package id, semver version, and which `.ship/` subtrees are public. See [Manifest schema](/schema-reference/manifest/) for the full shape.

## Ship Store

The Ship Store is a curated browsing surface on top of the Git-based registry — a place to find first-party and community packages without knowing the repo URL up front. Packages live in the same Git repos they always did; the Store adds discovery, versioning policies, and a quality bar.

Install and publish work today via `ship add` and `ship publish` against any Git URL. The Store ships next.
