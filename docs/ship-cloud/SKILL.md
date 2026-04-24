---
name: ship-cloud
stable-id: ship-cloud
description: Use when working with Ship Cloud — the Cloudflare Workers backend under `cloud/` that syncs runtime events across workspaces, plus the aspirational hosted platform (auth, billing, registry, GitHub app) that will land on top.
tags: [ship, cloud, platform, cloudflare, durable-objects]
authors: [ship]
---

# Ship Cloud

Ship Cloud is the hosted platform layer. Today it lives under `cloud/`
in this repo as a Cloudflare Workers deployment target: a Hono gateway
and two Durable Objects that project Rust-runtime events into durable
SQLite state.

## Current state (in `cloud/`)

### Apps

- `cloud/apps/gateway` — Hono Worker. Auth, routing, CORS. **Zero
  business logic.** Validates input, enforces auth, forwards to a DO
  via RPC, returns the result.
- `cloud/apps/actors/platform` — **PlatformDO**, one per project.
  Projects the workspace + job lifecycle events. Owns the project-scoped
  event log.
- `cloud/apps/actors/workspace` — **WorkspaceDO**, one per workspace.
  Projects actor + session lifecycle events. Created on demand by the
  platform DO.

### Packages

- `cloud/packages/types` — TS transport types. `EventEnvelope` and every
  event-type string constant are **codegen** from Rust via
  `cargo xtask gen-types` into `cloud/packages/types/src/generated.ts`.
  Cloud is a sync target for the runtime, not a parallel domain model —
  never invent event names.

### Tools

- `cloud/tools/drizzle` — Drizzle config factory used by each DO to
  generate migrations.

## Patterns

- **Hono** for gateway routing; middleware chain `onError` → `auth` →
  route handler.
- **Drizzle ORM** over DO SQLite storage. Schema in
  `src/data/schema/**/*.ts`; migrations in `migrations/` (regen via
  `pnpm --filter @ship-cloud/platform schema:generate`).
- **RPC facades** for DO method exposure — each DO class exports
  facade classes (SyncFacade, JobsFacade, etc.) that the gateway calls
  across the DO boundary.
- **Repository pattern** for data access. `src/repositories/*.repository.ts`
  wrap Drizzle queries and return `Result<T>`.
- **Projection repositories** translate inbound events into row updates
  (`session-projection`, `actor-projection`, `workspace-projection`,
  `job-projection`).
- **`Result<T>`** for error handling — no thrown errors across RPC.
- **Vitest with `@cloudflare/vitest-pool-workers`** for testing; use
  `runInDurableObject` for DO isolation.

## Projection → Rust event map

Cloud projects Rust events, nothing more. To add a new projection case,
find the emitter in `crates/core/ship-types/src/event_types.rs`, import
the constant in the projection repo, add a `switch` branch.

| Projection | Events handled | Status written |
|---|---|---|
| `WorkspaceProjection` (platform) | `workspace.{created,activated,archived,deleted}` | same |
| `JobProjection` (platform) | `job.{created,dispatched,completed,failed}` | same |
| `SessionProjection` (workspace) | `session.{started,ended}` | same |
| `ActorProjection` (workspace) | `actor.{created,woke,slept,stopped,crashed}` | same |

## Rules

- Gateway has zero business logic. Validate, authenticate, forward.
- DOs own their data. No shared state between DOs except through events.
- `EventEnvelope` shape and event-type string constants are Rust-owned.
  Regenerate via `cargo xtask gen-types` after schema changes; never
  hand-edit `cloud/packages/types/src/generated.ts`.
- Tests use `runInDurableObject` for DO isolation.
- Local-dev only today. Wrangler configs are present; production
  bindings + secrets are not wired.

## Aspirational (what "hosted platform" will also own)

Not yet in `cloud/`; all slated for the hosted tier:

- **Authentication** — Better Auth login, OAuth providers, user/org
  management. Stub in gateway middleware today.
- **Billing** — Stripe subscriptions, usage metering, plan limits.
- **Registry API** — package hosting, publish endpoints, search,
  security review queue. Studio talks to this via `/api/registry/*`.
- **GitHub App** — auto-publish webhooks, PR integration, repo linking.
- **Landing page** — marketing site at getship.dev.

## What Cloud does NOT own

- Local CLI (`ship` binary)
- Local daemon (`shipd`)
- Studio config IDE (workspace/agent/skill editing)
- Compiler (`.ship/` → provider configs)
- Registry browser UI (client-side, reads from registry API)

## Regen flow

When adding a field to Rust's `EventEnvelope` or a new `event_type` constant:

```
1. Edit crates/core/ship-types/src/{envelope.rs, event_types.rs}
2. cargo run -p xtask -- gen-types
3. Verify cloud/packages/types/src/generated.ts picked it up
4. Wire the new field through cloud schemas + repos if cloud stores it
5. cargo run -p xtask -- gen-types --check  # fails if stale
```

CI should run `cargo xtask gen-types --check` on every commit.
