---
title: "ADR Index"
description: "Architecture Decision Records — durable record of choices that constrain future work."
sidebar:
  label: "ADR Index"
  order: 1
---
One file per decision. Filenames are kebab-title; metadata lives in YAML frontmatter (`status`, `date`, `deciders`, `supersedes`, `superseded-by`, `tags`).

See `SKILL.md` for authoring conventions and lifecycle.

## Current ADRs

Listed alphabetically. Filter by `status` in frontmatter to see active decisions.

- [`collections-as-data-layer`](./collections-as-data-layer.md) — all user-authored content goes through collections in `data.db`; `platform.db` holds only runtime state.
- [`home-layout-and-scope-convention`](./home-layout-and-scope-convention.md) — `~/.ship/` splits into `library/` (portable user scope) and `machine/` (runtime state); canonical customization-vector list applied uniformly at project and user scopes.
- [`api-url-shape-and-versioning`](./api-url-shape-and-versioning.md) — all HTTP API routes live under `/api/v1/`; tenancy is a `t/{tenant}/` path segment; `/auth/*`, `/assets/*`, `/webhooks/*`, `/health` are exempt.
- [`artifact-storage-key-opacity`](./artifact-storage-key-opacity.md) — `ArtifactRecord.path` is an opaque storage key; a `StorageBackend` trait abstracts read/write so cloud can swap filesystem for object storage without caller changes.
