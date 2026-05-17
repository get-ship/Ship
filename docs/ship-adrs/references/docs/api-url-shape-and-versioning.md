---
title: "API URL Shape and Versioning"
description: "ADR: every HTTP API route on the local daemon lives under /api/v1/; auth, assets, and the dev webhook ingress are exempt; OpenAPI is published and CI-guarded."
group: ADRs
order: 30
audience: internal
status: accepted
date: 2026-04-25
supersedes: []
tags: [http-api, versioning, contract-freeze]
---

# API URL Shape and Versioning

## Context

Audit Item 9 — the local daemon's HTTP API today exposes 137+ routes across `apps/shipd/src/router.rs` with no version prefix, no published OpenAPI, and no CI guard against drift. The daemon serves the single local user (CLI, MCP, Studio web shell). It is not a multi-tenant service.

This ADR locks in the daemon's URL contract for v1. Cloud has its own architecture and its own URL surface — that's a separate decision when cloud lands, not something this ADR speaks to.

Three facts force the decision now:

1. **Versioning is retroactive or it is impossible.** Adding `/api/v1/` after external callers (the CLI, MCP, published views) pin today's shape means carrying both forever.
2. **OpenAPI is the same problem.** Without a committed `openapi.json` and a CI guard, every new route is a silent contract change. Generated SDKs (`packages/sdk/`) drift from reality.
3. **Local-first stays local-first.** The daemon serves one user on one machine. Adding tenancy or multi-user shapes "for cloud" at the daemon layer would be premature and wrong — cloud is an adjacent service, not a different runtime mode.

## Decision

### 1. `/api/v1/` is the prefix for every versioned route

Every HTTP route the daemon serves belongs to the Ship public contract and lives under `/api/v1/`. Examples:

```
/api/v1/workspaces
/api/v1/workspaces/{id}
/api/v1/workspaces/{id}/sessions
/api/v1/agent_sessions/{id}
/api/v1/agent_sessions/{id}/log               (SSE)
/api/v1/collections/{id}/items
/api/v1/artifacts/{id}
/api/v1/views
/api/v1/view/{name}
/api/v1/view/{name}/sse              (Datastar signal stream)
/api/v1/ship/schema
/api/v1/openapi.json
```

The prefix is unconditional. There is no "convenience prefix-less" escape hatch. Unversioned routes either belong to an exempt family (below) or they are bugs.

### 2. Exempt families

Three categories sit outside `/api/v1/` because they are not versioned application contract:

- **`/auth/*`** — pairing, attestation, session establishment. The pairing protocol is a stable handshake, not a versioned contract.
- **`/assets/*`** — static UI bundles for the local Studio shell. Cache-controlled, content-addressed.
- **`/health`** — liveness/readiness probe.

The previous `/api/hooks/webhook/{*path}` (legacy webhook ingress) is **not** part of this v1 contract. It exists today as a development convenience for testing trigger sources locally; in production deployments webhooks come through cloud (separate architecture decision). Either it stays as a dev-only ingress (gated explicitly) or it is removed. That call belongs to whoever designs the trigger-source/cloud-integration story; it is out of scope here.

### 3. Published OpenAPI and CI guard

`apps/docs/openapi.json` is generated from the daemon's route registrations and is the canonical contract. CI runs an `openapi-check` target that fails when:
- A route is added or modified without a corresponding OpenAPI entry.
- A route is removed without a `removed-in: vNext` annotation.

Generated SDK code (`packages/sdk/`) regenerates from this file. Drift is impossible by construction.

### 4. Cloud is out of scope

Cloud's URL contract is a separate decision. Cloud is not a hosted daemon. Cloud is adjacent services (storage, sync, compute, integrations, auth) that the local daemon talks *to* over published cloud APIs. Whatever shape those cloud APIs take, they do not retroactively constrain the daemon's `/api/v1/` namespace.

## Consequences

### Positive

- One clean local API contract. Versioned, documented, drift-protected.
- Generated SDK matches reality; no hand-maintained type drift.
- Cloud's design space is not pre-constrained by daemon shape, and vice versa.
- New routes are documented or they don't merge. Reviewer cost goes to zero on "is this on the contract."

### Negative

- Every existing route prefix changes once. Mechanical migration: CLI, MCP, view fetches, hardcoded test URLs.
- The `openapi-check` CI step has to actually fail loud or it's worthless. Requires up-front rigor.

## Alternatives Considered

### Unversioned (`/api/...`)

Rejected. Without a version prefix, breaking-change communication has nowhere to live. SDKs can't pin a contract. Future v2 routes either coexist messily under `/api/...` or require a global migration.

### Tenancy in URL (`/api/v1/t/{tenant}/...`)

Rejected as out of scope. Daemon serves one local user. Adding tenancy at the daemon layer "for cloud parity" assumes cloud is a hosted daemon, which it isn't. When cloud's URL surface is designed, it can adopt whatever tenancy model fits cloud's architecture without constraining daemon URLs.

### Header-based versioning (`X-API-Version: 1`)

Rejected. Less discoverable, harder to debug in browser dev tools, no caching benefit, no copy-paste-able URLs. Path versioning is boring and right.

## Migration

1. Add `/api/v1/` prefix to every existing daemon route.
2. Update CLI HTTP clients (`apps/ship-cli/src/`) and MCP transport (`apps/mcp/src/network_client.rs`) to include the prefix.
3. Update view fetch paths (`.ship/views/**` Jinja templates) and Datastar signal sources.
4. Generate `apps/docs/openapi.json` from the new routes.
5. Land `openapi-check` CI script and wire into `just check-raw` or a sibling target.

## How to Measure Success

- `rg '"/api/(?!v1/)' apps/ packages/ .ship/` returns zero matches outside the three exempt families.
- `apps/docs/openapi.json` exists and is regenerated by `just gen-openapi` (or equivalent).
- A new route added without an OpenAPI entry fails CI.
- Generated SDK in `packages/sdk/` matches the daemon's route table.
